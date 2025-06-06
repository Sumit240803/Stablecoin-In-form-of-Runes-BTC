use bitcoin::{Address, XOnlyPublicKey};
use candid::CandidType;
use ic_cdk::update;
use serde::Deserialize;

use crate::{common::DerivationPath, p2tr, schnorr_api::get_schnorr_public_key, BTC_CONTEXT};

#[derive(CandidType, Deserialize)]
pub enum ResultString {
    ok(String),
    err(String),
}

#[update]
pub async fn get_p2tr_script_path_address() -> ResultString {
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());

    let internal_path = DerivationPath::p2tr(0, 1);
    let script_leaf_key_path = DerivationPath::p2tr(0, 2);

    let internal_key = get_schnorr_public_key(&ctx, internal_path.to_vec_u8_path()).await;
    let script_key = get_schnorr_public_key(&ctx, script_leaf_key_path.to_vec_u8_path()).await;

    let internal_key_bytes = match internal_key {
        Ok(ref key) => key,
        Err(e) => return ResultString::err(format!("Internal key error: {}", e)),
    };

    let script_key_bytes = match script_key {
        Ok(ref key) => key,
        Err(e) => return ResultString::err(format!("Script key error: {}", e)),
    };

    let taproot_spend_info = match p2tr::create_taproot_spend_info(internal_key_bytes, script_key_bytes) {
        Ok(info) => info,
        Err(e) => return ResultString::err(format!("Taproot spend info error: {}", e)),
    };

    let address = Address::p2tr_tweaked(taproot_spend_info.output_key(), ctx.bitcoin_network).to_string();

    ResultString::ok(address)
}
