use std::io::{BufWriter, stdout};

use dialoguer::Input;
use serde::{Deserialize, Serialize};
use web3::types::Address;
use OperatingMode::Bulk;

use roninrest::Adapter;
use crate::OperatingMode::Single;
use crate::roninrest::ERC20Balance;

mod roninrest;

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

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
struct NonFungibleWallet {
    axies: Vec<String>,
    lands: Vec<String>,
    items: Vec<String>,
    runes: Vec<[String; 2]>,
    charms: Vec<[String; 2]>,
}

#[derive(Serialize, Deserialize)]
struct Wallet {
    wallet: String,
    fungible: Vec<ERC20Balance>,
    non_fungible: NonFungibleWallet
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

async fn process(wallet: String, silent: bool, operation_mode: &OperatingMode) {
    let rr = Adapter::new();

    if operation_mode == &Bulk && !silent {
        println!("Address: {}", wallet);
    }

    if !silent {
        println!("Loading ERC20 balances...");
    }
    let erc20 = rr.list_erc20(wallet.clone()).await;

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
        println!("\nResult:\n=================\nAxies: {}\nLands: {}\nItems: {}\nRunes: {}\nCharms: {}", axies.len(), lands.len(), items.len(), total_runes, total_charms);
        for balance in &erc20 {
            println!("{}", balance);
        }
    }
    let serialized = serde_json::to_string(&Wallet {
        wallet: wallet.clone(),
        fungible: erc20,
        non_fungible: NonFungibleWallet {
            axies,
            lands,
            items,
            runes: runes_vec,
            charms: charms_vec,
        }
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

    if output_format == "file" || operation_mode == &Bulk {
        let file_name = format!("{}.json", wallet);

        std::fs::write(&file_name, &serialized).unwrap();
        if !silent {
            println!("Wallet stored as: {}", file_name);
        }
    }

    if output_format == "stdout" && operation_mode == &Single {
        println!("{}", serialized);
    }

    if operation_mode == &Bulk && !silent {
        println!("\n");
    }
}

fn try_read_address_file(file_name: &String, silent: bool) -> Vec<String> {
    let mut addresses : Vec<String> = vec![];
    let path = std::path::Path::new(file_name);

    if path.exists() {
        let content = std::fs::read_to_string(path).unwrap();
        let content = content.split(LINE_ENDING).collect::<Vec<&str>>();

        for line in content {
            if normalize_address(line).parse::<Address>().is_ok() {
                addresses.push(normalize_address(line))
            } else if !silent {
                println!("Skipping invalid address: {}", line)
            }
        }
    } else {
        panic!("File does not exist: {}", file_name)
    }

    addresses
}

#[derive(Eq, PartialEq)]
enum OperatingMode {
    Single,
    Bulk
}

#[tokio::main]
async fn main() {
    let silent: bool = match ArgParser::split(&"--silent".to_string()) {
        None => false,
        Some(_) => true
    };

    let from_file: String = match ArgParser::split(&"--source-file".to_string()) {
        None => "".to_string(),
        Some(file) => file
    };

    let operating_mode: OperatingMode = match from_file.as_str() {
        "" => Single,
        _ => Bulk
    };

    if !silent {
        ferris_says::say(b"Ronin Wallet Export v0.1.0\n\nby: wehmoen#0001", 24, &mut BufWriter::new(stdout())).unwrap();
    }

    if operating_mode == Single {
        let wallet: String = match ArgParser::split(&"--address".to_string()) {
            None => {
                if silent {
                    panic!("No address provided!");
                }

                normalize_address(
                    &Input::new()
                        .with_prompt("Please enter your Ronin address")
                        .with_initial_text("ronin:3759468f9fd589665c8affbe52414ef77f863f72")
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

        process(wallet, silent, &operating_mode).await
    } else {

        let addresses = try_read_address_file(&from_file, silent);

        if !silent {
            println!("Bulk Mode: {} addresses", addresses.len())
        }

        for wallet in addresses {
            process(wallet, silent, &operating_mode).await
        }
    }
}
