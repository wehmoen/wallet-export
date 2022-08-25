mod roninrest;

use roninrest::Adapter;
use serde::{Serialize, Deserialize};
use web3::types::Address;

enum NFT {
    Axie,
    Land,
    Item
}

#[derive(Serialize, Deserialize)]
struct Wallet {
    axies: Vec<String>,
    lands: Vec<String>,
    items: Vec<String>
}

struct ArgParser {}
impl ArgParser {
    fn parse() -> Vec<String> {
        std::env::args().collect()
    }

    fn split(param: &String) -> Option<String> {

        let args: Vec<String> = ArgParser::parse();

        for arg in args {
            if arg.starts_with(param) {
                let kv: Vec<&str> = arg.split('=').collect();
                if kv.len() == 2 {
                    return Some(kv[1].to_string())
                }
            }
        }

        None
    }
}

#[tokio::main]
async fn main() {

    let address: String = match ArgParser::split(&"--address".to_string()) {
        None => {
            normalize_address(
                &Input::new()
                    .with_prompt("Please enter your Ronin address")
                    .validate_with(|input: &String| -> Result<(), &str> {
                        let address = normalize_address(input).as_str().parse::<Address>();
                        match address {
                            Ok(_) => Ok(()),
                            Err(_) => Err("Failed to parse your address!")
                        }
                    })
                    .interact()
                    .unwrap()
            )
        },
        Some(passed_address) => {
            let address = normalize_address(&passed_address).as_str().parse::<Address>();
            match address {
                Ok(_) => normalize_address(&passed_address),
                Err(_) => {
                    panic!("Could not parse address!");
                }
            }
        }
    };

    let wallet: String = "ronin:3759468f9fd589665c8affbe52414ef77f863f72".to_string();

    let rr = Adapter::new();

    println!("Loading axies...");

    let axies = rr.list_nft(NFT::Axie, wallet.clone()).await;

    println!("Loading lands...");
    let lands = rr.list_nft(NFT::Land, wallet.clone()).await;

    println!("Loading items...");
    let items = rr.list_nft(NFT::Item, wallet.clone()).await;

    println!("Result:");
    println!("AXIES: {}\"Lands: {}\"Items: {}", axies.len(), lands.len(), items.len());

    let serialized = serde_json::to_string(
        &Wallet {
            axies,
            lands,
            items
        }
    ).unwrap();

    let file_name = format!("{}.json", wallet);

    std::fs::write(file_name, serialized).unwrap();

    println!("Wallet stored as: {}", file_name);

}
