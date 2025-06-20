
## BTC backed Runes - ICP & Rust

This project is based on staking BTC and get runes in exchange of it. These runes could be traded or stored. The ratio of Rune to that of BTC will be 1:1. 
## Prerequisites

Before starting development, ensure you have:

- [Git](https://git-scm.com/downloads)
- [Rust and Cargo](https://www.geeksforgeeks.org/rust/how-to-install-rust-on-windows-and-linux-operating-system/)
- [dfx (Internet Computer SDK)](https://internetcomputer.org/docs/building-apps/getting-started/install)
- [Bitcoin core ](https://bitcoincore.org/bin/bitcoin-core-27.0/)
    - For bitcoin core you need to find your specific system version. For example : if you are on Linux you need to download (bitcoin-27.0-aarch64-linux-gnu.tar.gz from the above link).
- Linux (Preferred) - Could be WSL for windows or a linux operating system.
            
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
You also need to add schnorr canister's id which can be found using this in another terminal
```bash
 dfx canister id schnorr_canister
```
After all the above steps you must see the links to the canisters.

## Documentation

[Runes](https://internetcomputer.org/docs/build-on-btc/runes)

[Rust](https://doc.rust-lang.org/stable/)

[ICP](https://internetcomputer.org/docs/building-apps/getting-started/quickstart)


