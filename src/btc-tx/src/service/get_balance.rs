use ic_cdk::{bitcoin_canister::{bitcoin_get_balance, GetBalanceRequest}, update};

//use ic_cdk::{api::management_canister::bitcoin::{bitcoin_get_balance, GetBalanceRequest}, update};
use crate::BTC_CONTEXT;

#[update]
pub async fn get_balance(address: String) -> u64 {
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());

    match bitcoin_get_balance(&GetBalanceRequest {
        address,
        network: ctx.network,
        min_confirmations: None,
    }).await {
        Ok(balance) => balance,
        Err(e) => {
            ic_cdk::println!("get_balance failed: {:?}", e);
            0 // Or any fallback default
        }
    }
}
