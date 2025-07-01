use std::{
    cell::{Cell},
   
};

use candid::{CandidType};
use ic_cdk::{init, post_upgrade};

use serde::Deserialize;


use ic_cdk::bitcoin_canister::Network;

mod common;
mod ecdsa;
mod p2tr;
mod schnorr_api;
mod service;
mod runes;

#[derive(Clone, Copy)]
pub struct BitcoinContext {
    pub network: Network,
    pub bitcoin_network: bitcoin::Network,
    pub key_name: &'static str,
}
#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub network: Network,
}

thread_local! {
    static BTC_CONTEXT: Cell<BitcoinContext> = const {
        Cell::new(BitcoinContext {
            network: Network::Testnet,
            bitcoin_network: bitcoin::Network::Testnet,
            key_name: "dfx_test_key",
        })
    };
}

fn init_upgrade(network: Network) {
    let key_name = match network {
        Network::Regtest => "dfx_test_key",
        Network::Mainnet | Network::Testnet => "test_key_1",
    };

    let bitcoin_network = match network {
        Network::Mainnet => bitcoin::Network::Bitcoin,
        Network::Testnet => bitcoin::Network::Testnet,
        Network::Regtest => bitcoin::Network::Regtest,
    };

    BTC_CONTEXT.with(|ctx| {
        ctx.set(BitcoinContext {
            network,
            bitcoin_network,
            key_name,
        })
    });
}

#[init]
pub fn init(network: Network) {
    init_upgrade(network);
}

/// Post-upgrade hook.
/// Reinitializes the BitcoinContext with the same logic as `init`.
#[post_upgrade]
fn upgrade(network: Network) {
    init_upgrade(network);
}

/// Input structure for sending Bitcoin.
/// Used across P2PKH, P2WPKH, and P2TR transfer endpoints.
#[derive(candid::CandidType, candid::Deserialize)]
pub struct SendRequest {
    pub destination_address: String,
    pub amount_in_satoshi: u64,
}
