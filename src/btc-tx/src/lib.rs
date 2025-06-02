use std::cell::Cell;
use crate::{common::DerivationPath, ecdsa::get_ecdsa_public_key};
use candid::CandidType;
use ic_cdk::{init, post_upgrade, update};
use serde::Deserialize;
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;
use bitcoin::{Address, CompressedPublicKey};

mod common;
mod ecdsa;
mod p2wpkh;
#[derive(Clone, Copy,CandidType,Deserialize)]
pub enum Network {
    Mainnet,
    Testnet,
    Regtest,
}

#[derive(Clone,Copy)]
pub struct BitcoinContext{
    pub network : BitcoinNetwork,
    pub bitcoin_network : bitcoin::Network,
    pub key_name : &'static str,
}

thread_local! {
    static BTC_CONTEXT: Cell<BitcoinContext> = 
        Cell::new(BitcoinContext {
            network: BitcoinNetwork::Testnet,
            bitcoin_network: bitcoin::Network::Testnet,
            key_name: "test_key_1",
        });
    }

fn init_upgrade(network: BitcoinNetwork) {
    let key_name = match network {
        BitcoinNetwork::Regtest => "dfx_test_key",
        BitcoinNetwork::Mainnet | BitcoinNetwork::Testnet => "test_key_1",
    };

    let bitcoin_network = match network {
        BitcoinNetwork::Mainnet => bitcoin::Network::Bitcoin,
        BitcoinNetwork::Testnet => bitcoin::Network::Testnet,
        BitcoinNetwork::Regtest => bitcoin::Network::Regtest,
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
pub fn init(network: BitcoinNetwork) {
    init_upgrade(network);
}

#[post_upgrade]
fn upgrade(network: BitcoinNetwork) {
    init_upgrade(network);
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct SendRequest {
    pub destination_address: String,
    pub amount_in_satoshi: u64,
}

#[update]
pub async fn get_p2wpkh_address() -> String {
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());

  
    let derivation_path = DerivationPath::p2wpkh(0, 0);

    let public_key = get_ecdsa_public_key(&ctx, derivation_path.to_vec_u8_path()).await;

    let public_key = CompressedPublicKey::from_slice(&public_key).unwrap();

  
    Address::p2wpkh(&public_key, ctx.bitcoin_network).to_string()
}