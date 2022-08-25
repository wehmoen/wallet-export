use std::time::Duration;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use serde::{Deserialize, Serialize};

use crate::NFT;

const DEFAULT_USER_AGENT: &str = "ronin/wallet-export0.1.0 See: https://github.com/wehmoen/wallet-export";

pub type RRTransactionHash = String;

#[derive(Serialize, Deserialize)]
pub struct NFTIdList {
    pub address: String,
    pub contract: String,
    pub items: Option<Vec<String>>,
}


fn normalize_address(input: &str) -> String {
    input.replace("ronin:", "0x")
}

pub struct Adapter {
    pub host: String,
    client: ClientWithMiddleware,
}

impl Adapter {
    pub fn new() -> Adapter {
        Adapter {
            host: "https://ronin.rest".into(),
            client: ClientBuilder::new(reqwest::Client::new()).with(
                RetryTransientMiddleware::new_with_policy(
                    ExponentialBackoff {
                        max_n_retries: 25,
                        min_retry_interval: Duration::from_secs(1),
                        max_retry_interval: Duration::from_secs(15),
                        backoff_exponent: 2,
                    }
                )
            ).build(),
        }
    }

    pub async fn list_nft(&self, nft: NFT, address: String) -> Vec<String> {
        let mut ids: Vec<String> = vec![];

        let address = normalize_address(&address);
        let mut offset: i64 = 0;

        loop {
            let param: String = match nft {
                NFT::Axie => "axie".to_string(),
                NFT::Land => "land".to_string(),
                NFT::Item => "item".to_string()
            };
            let param = param.to_lowercase();

            let request_url = format!("{}/ronin/nfts/{}/{}?offset={}", self.host, param, address, offset);
            let data: serde_json::Value = serde_json::from_str(
                &self.client.get(request_url).header("user-agent", DEFAULT_USER_AGENT).send().await.unwrap().text().await.unwrap()
            ).unwrap();
            let items = match nft {
                NFT::Axie => data.get("axie"),
                NFT::Land => data.get("land"),
                NFT::Item => data.get("item")
            };

            let mut items = items.unwrap().as_array().to_owned().unwrap().to_owned().iter().map(|i| -> String {
                i.to_string().replace("\"", "")
            }).collect::<Vec<String>>();

            let try_more = items.len() == 25;

            ids.append(&mut items);

            if !try_more {
                break;
            }

            offset = offset + 25;
        }

        ids
    }
}