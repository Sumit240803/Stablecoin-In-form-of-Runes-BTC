use std::{cell::{Cell, RefCell}, collections::HashMap};

use candid::{CandidType, Principal};
use ic_cdk::{init, post_upgrade};

use serde::Deserialize;
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;



mod common;
mod ecdsa;
mod service;
mod p2wpkh;
mod schnorr_api;
mod p2tr;
mod user_service;
mod tags;
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
    pub schnorr_canister: Option<Principal>,
  
}
#[derive(CandidType, Deserialize)]
pub struct InitArgs{
    pub network : BitcoinNetwork,
    pub schnorr_canister : Option<Principal>
}

thread_local! {
    static BTC_CONTEXT: Cell<BitcoinContext> = 
        Cell::new(BitcoinContext {
            network: BitcoinNetwork::Regtest,
            bitcoin_network: bitcoin::Network::Regtest,
            key_name: "dfx_test_key",
            schnorr_canister : None,
        });

    
    static INTENTS : RefCell<HashMap<Principal,(String,u64)>> =RefCell::new(HashMap::new());
    
    }
/*thread_local! {
    static INTENTS : RefCell<HashMap<String,(Principal,u64)>> =RefCell::new(HashMap::new());
}*/
fn init_upgrade(network: BitcoinNetwork ,schnorr_canister :Option<Principal>) {
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
            schnorr_canister
        })
    });
}


#[init]
pub fn init(args : InitArgs) {
    init_upgrade(args.network, args.schnorr_canister);
}

#[post_upgrade]
fn upgrade(args : InitArgs) {
    init_upgrade(args.network, args.schnorr_canister);
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct SendRequest {
    pub destination_address: String,
    pub amount_in_satoshi: u64,
}