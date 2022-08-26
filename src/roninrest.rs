use std::collections::HashMap;
use std::time::Duration;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web3::contract::{Contract, Options};
use web3::transports::Http;
use web3::types::{Address};
use web3::Web3;

use crate::{ERC1155, NFT};

const DEFAULT_USER_AGENT: &str = "ronin/wallet-export0.1.0 See: https://github.com/wehmoen/wallet-export";

const WEB3_RPC: &str = "http://localhost:8545";

#[derive(Serialize, Deserialize)]
pub struct NFTIdList {
    pub address: String,
    pub contract: String,
    pub items: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    erc: ERC1155,
    token_id: u64,
    name: String,
    id: String,
    category: String,
    rarity: String,
    description: String,
    image_url: String,
    pub balance: i64,
}

impl TokenInfo {
    pub fn minimal(&self) -> [String; 2] {
        [self.id.clone(), self.balance.to_string()]
    }
}

fn normalize_address(input: &str) -> String {
    input.replace("ronin:", "0x")
}

pub struct Adapter {
    pub host: String,
    web3_client: Web3<Http>,
    client: ClientWithMiddleware,
}

impl Adapter {
    pub fn new() -> Adapter {
        Adapter {
            host: "https://ronin.rest".into(),
            web3_client: Web3::new(
                Http::new(WEB3_RPC).unwrap()
            ),
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

    pub async fn list_erc1155(&self, erc1155: ERC1155, address: String) -> HashMap<u64, TokenInfo> {
        let address = normalize_address(&address).as_str().parse::<Address>().unwrap();

        let contract = match erc1155 {
            ERC1155::Rune => "0xc25970724f032af21d801978c73653c440cf787c",
            ERC1155::Charm => "0x814a9c959a3ef6ca44b5e2349e3bba9845393947"
        };

        let token_data_url = match erc1155 {
            ERC1155::Rune => "https://ronin.rest/origin/game/listRunes",
            ERC1155::Charm => "https://ronin.rest/origin/game/listCharms"
        };

        let token_data: Value = serde_json::from_str(
            &self.client.get(token_data_url).header("user-agent", DEFAULT_USER_AGENT).send().await.unwrap().text().await.unwrap()
        ).unwrap();

        let items: Vec<Value> = token_data.get("_items").unwrap().as_array().unwrap().iter().map(|i| -> Value {
            i.to_owned()
        }).collect();

        let mut token: HashMap<u64, TokenInfo> = HashMap::new();

        for item in items {
            let inner = item["item"].to_owned();

            if inner["tokenStandard"] == *"ERC1155" && inner["tokenAddress"].as_str().unwrap().to_lowercase() == contract {
                let token_id: u64 = match inner["tokenId"].as_str() {
                    None => 0u64,
                    Some(id) => id.parse::<u64>().unwrap()
                };

                if token_id > 0 {
                    token.insert(token_id, TokenInfo {
                        erc: erc1155,
                        token_id,
                        name: inner["name"].as_str().unwrap().into(),
                        id: inner["id"].as_str().unwrap().into(),
                        category: inner["category"].as_str().unwrap().into(),
                        rarity: inner["rarity"].as_str().unwrap().into(),
                        description: inner["description"].as_str().unwrap().into(),
                        image_url: inner["imageUrl"].as_str().unwrap().into(),
                        balance: 0i64,
                    });
                }
            }
        }


        let contract = Contract::from_json(self.web3_client.eth(), contract.parse().unwrap(), include_bytes!("abi.json")).unwrap();

        let token_ids = token.keys().map(|t| -> u64 {
            t.to_owned()
        }).collect::<Vec<u64>>();

        for id in token_ids {
            let balance: i64 = contract.query("balanceOf",
                                              (
                                                  address,
                                                  id
                                              )
                                              ,
                                              None,
                                              Options::default(),
                                              None)
                .await.unwrap();

            match token.get(&id) {
                None => {}
                Some(t) => {
                    let mut info = t.clone();
                    info.balance = balance;
                    token.insert(id, info);
                }
            };
        }


        token
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
                i.to_string().replace('"', "")
            }).collect::<Vec<String>>();

            let try_more = items.len() == 25;

            ids.append(&mut items);

            if !try_more {
                break;
            }

            offset += 25;
        }

        ids
    }
}