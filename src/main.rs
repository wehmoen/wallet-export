use std::io::{BufWriter, stdout};

use dialoguer::Input;
use roninrest::Adapter;
use serde::{Deserialize, Serialize};
use web3::types::Address;

mod roninrest;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum ERC1155 {
    Rune,
    Charm,
}

pub enum NFT {
    Axie,
    Land,
    Item,
}

#[derive(Serialize, Deserialize)]
struct Wallet {
    axies: Vec<String>,
    lands: Vec<String>,
    items: Vec<String>,
    runes: Vec<[String; 2]>,
    charms: Vec<[String; 2]>,
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
                    return Some(kv[1].to_string());
                }
            }
        }

        None
    }
}

fn normalize_address(input: &str) -> String {
    input.replace("ronin:", "0x")
}

#[tokio::main]
async fn main() {
    let silent: bool = match ArgParser::split(&"--silent".to_string()) {
        None => false,
        Some(_) => true
    };

    if !silent {
        ferris_says::say(b"Ronin Wallet Export v0.1.0\n\nby: wehmoen#0001", 24, &mut BufWriter::new(stdout())).unwrap();
    }

    let wallet: String = match ArgParser::split(&"--address".to_string()) {
        None => {

            if silent {
                panic!("No address provided!");
            }

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
        }
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

    let rr = Adapter::new();
    if !silent {
        println!("Loading runes...");
    }
    let runes = rr.list_erc1155(ERC1155::Rune, wallet.clone()).await;
    if !silent {
        println!("Loading charms...");
    }
    let charms = rr.list_erc1155(ERC1155::Charm, wallet.clone()).await;
    if !silent {
        println!("Loading axies...");
    }
    let axies = rr.list_nft(NFT::Axie, wallet.clone()).await;
    if !silent {
        println!("Loading lands...");
    }
    let lands = rr.list_nft(NFT::Land, wallet.clone()).await;
    if !silent {
        println!("Loading items...");
    }
    let items = rr.list_nft(NFT::Item, wallet.clone()).await;

    let mut runes_vec: Vec<[String; 2]> = vec![];
    let mut total_runes: i64 = 0;

    for rune in runes {
        if rune.1.balance > 0 {
            total_runes += rune.1.balance;
            runes_vec.push(rune.1.minimal());
        }
    }

    let mut charms_vec: Vec<[String; 2]> = vec![];
    let mut total_charms: i64 = 0;

    for charm in charms {
        if charm.1.balance > 0 {
            total_charms += charm.1.balance;
            charms_vec.push(charm.1.minimal());
        }
    }

    if !silent {
        println!("Result:\n\nAxies: {}\nLands: {}\nItems: {}\nRunes: {}\nCharms: {}", axies.len(), lands.len(), items.len(), total_runes, total_charms);
    }
    let serialized = serde_json::to_string(&Wallet {
        axies,
        lands,
        items,
        runes: runes_vec,
        charms: charms_vec,
    }
    ).unwrap();

    let output_format = match ArgParser::split(&"--output".to_string()) {
        None => "stdout".to_string(),
        Some(value) => {

            match value.as_str() {
                "file" => value,
                "stdout" => value,
                _ => {
                    panic!("Invalid --output format. Supported: file,stdout");
                }
            }
        }
    };

    if output_format == "file" {
        let file_name = format!("{}.json", wallet);

        std::fs::write(&file_name, &serialized).unwrap();
        if !silent {
            println!("Wallet stored as: {}", file_name);
        }
    }

    if output_format == "stdout" {
        println!("{}", serialized);
    }
}
