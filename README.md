
# BTC backed Runes - ICP & Rust
This project is based on staking BTC and get runes in exchange of it. These runes could be traded or stored. The ratio of Rune to that of BTC will be 1:1.
## Prerequisites

Before starting development, ensure you have:

- [Git](https://git-scm.com/downloads)
- [Rust and Cargo](https://www.geeksforgeeks.org/rust/how-to-install-rust-on-windows-and-linux-operating-system/)
- [dfx (Internet Computer SDK)](https://internetcomputer.org/docs/building-apps/getting-started/install)
- [Bitcoin core ](https://bitcoincore.org/bin/bitcoin-core-27.0/)
    - For bitcoin core you need to find your specific system version. For example : if you are on Linux you need to download (bitcoin-27.0-aarch64-linux-gnu.tar.gz from the above link).
- Linux (Preferred) - Could be WSL for windows or a linux operating system.
- Ord [Link to install](https://github.com/ordinals/ord?tab=readme-ov-file)           
## Deployment

Clone project into local directory

```bash
  git clone https://github.com/DevenDeshmukh/USDB-A-BTC-Backed-Stablecoin-in-the-Form-of-Runes

  cd USDB-A-BTC-Backed-Stablecoin-in-the-Form-of-Runes
```
Start dfx in background
```bash
 dfx start --background --clean
```
In another terminal navigate to the project 
```bash
cargo build
dfx canister create --all
dfx build
dfx deploy
```
The Deployment takes some arguments
```bash
 mainnet/testnet/regtest 
 Select regtest for local testing through arrow keys

```

After all the above steps you must see the links to the canisters.

## Etch Rune

After deploying the canister you need to run the following commands 

First make sure you have installed ord into your system. 
[Link to install ord](https://github.com/ordinals/ord?tab=readme-ov-file).

 Follow these steps

1. Start the ord server to track Rune balances:

 ```
ord --config-dir . server
 ```
2. Get a Taproot address for the Rune etching:
```
dfx canister call btc-tx get_p2tr_key_path_only_address '()'
```
3. Fund the address with bitcoin to pay for the etching:

```
bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 100 <p2tr_key_path_only_address>
```
4. Etch the Rune with an uppercase name (maximum 28 characters):
```
dfx canister call btc-tx etch_rune '("ICPRUNE")'
```

5. Mine a block to confirm the etching:
```
bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 <p2tr_key_path_only_address>
```
6. Decode the Runestone to verify the etching:
```
ord --config-dir . decode --txid <transaction_id>
```
The Rune is now etched with 1_000_000 tokens minted to your address. The tokens can be transferred using standard Bitcoin transactions with Runestone data.
## Documentation

[Runes](https://internetcomputer.org/docs/build-on-btc/runes)

[Rust](https://doc.rust-lang.org/stable/)

[ICP](https://internetcomputer.org/docs/building-apps/getting-started/quickstart)


