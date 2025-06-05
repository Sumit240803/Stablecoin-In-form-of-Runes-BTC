use bitcoin::Address;
use ic_cdk::update;

use crate::{common::DerivationPath, p2tr, schnorr_api::get_schnorr_public_key, BTC_CONTEXT};

#[update]
pub async fn get_p2tr_script_path_address()->String{
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());
    let internal_path = DerivationPath::p2tr(0, 1);
    let script_leaf_key_path = DerivationPath::p2tr(0,2);
    let internal_key = get_schnorr_public_key(&ctx, internal_path.to_vec_u8_path()).await;
    let script_key = get_schnorr_public_key(&ctx, script_leaf_key_path.to_vec_u8_path()).await;
    let taproot_spend_info = p2tr::create_taproot_spend_info(&internal_key, &script_key);
    Address::p2tr_tweaked(taproot_spend_info.output_key(), ctx.bitcoin_network).to_string()
}