# Ronin Wallet Export
Exports ERC20, ERC721 and ERC1155 balances for any account.

## Installation

```shell
> git clone https://github.com/wehmoen/wallet-export
> cd wallet-export
> cargo build -r
> cargo install --path .
```

## Usage

```shell

# By default the result is written to stdout.
# To write the output to a json file append --output=file to the command

# Interactive mode:
> ronin-wallet-export

# Non-Interactive mode 
> ronin-wallet-export --address=ronin:....

# Non-Interactive bulk mode
# Provide a file with one address per line to process
# For bulk processing the output format is always set to "file"

> ronin-wallet-export --source-file=addresses.txt

# You can supress any logs by adding the --silent=y flag

```

## Output

Each address generates the following output:

```js
let output = {
  "wallet": "0xwallet_address",
  "fungible": [], // Array fo ERC20 Balances
  "non_fungible": {
    "axies": [],
    "lands": [],
    "items": [],
    "runes": [],
    "charms": []
  }
}
```