
# BTC backed Runes as Stablecoin - ICP & Rust
This project enables users to stake BTC and receive Runes, a BTC-backed stablecoin. Runes maintain value stability and can be used across the ecosystem, while the staked BTC remains locked and redeemable, ensuring security and trust.

## Prerequisites

Before starting development, ensure you have:

- [Git](https://git-scm.com/downloads)
- [Rust and Cargo](https://www.geeksforgeeks.org/rust/how-to-install-rust-on-windows-and-linux-operating-system/)
- [dfx (Internet Computer SDK)](https://internetcomputer.org/docs/building-apps/getting-started/install)
- [Bitcoin core ](https://bitcoincore.org/bin/bitcoin-core-28.0/)
    - For bitcoin core you need to find your specific system version. For example : if you are on Linux you need to download (bitcoin-28.0-aarch64-linux-gnu.tar.gz from the above link).
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
4. Etch the Rune:
```
open candid ui after deploying the canister and add values to the etch rune function : 
turbo - true
premine - 1000
edicts (check the box) - output - 1 , amount - 500
name - MYRUNE
amount - 100
divisibility - 6

Run call and wait for some time you will receive the address
```

5. Mine a block to confirm the etching:
```
bitcoin-cli -conf=$(pwd)/bitcoin.conf generatetoaddress 1 <p2tr_key_path_only_address>
```
6. Decode the Runestone to verify the etching:
```
ord --config-dir . decode --txid <transaction_id>
```
The Rune is now etched. The tokens can be transferred using standard Bitcoin transactions with Runestone data.
## Documentation

[Runes](https://internetcomputer.org/docs/build-on-btc/runes)

[Rust](https://doc.rust-lang.org/stable/)

[ICP](https://internetcomputer.org/docs/building-apps/getting-started/quickstart)


