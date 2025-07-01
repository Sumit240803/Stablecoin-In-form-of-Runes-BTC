use bitcoin::{key::Secp256k1, Address, PublicKey, XOnlyPublicKey};
use ic_cdk::update;

use crate::{common::DerivationPath, schnorr_api::get_schnorr_public_key, BTC_CONTEXT};

#[update]
pub async fn get_p2tr_key_path_only_address()->String{
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());
    let internal_key_path = DerivationPath::p2tr(0, 0);
    let internal_key = get_schnorr_public_key(&ctx, internal_key_path.to_vec_u8_path()).await;
    let internal_key = XOnlyPublicKey::from(PublicKey::from_slice(&internal_key).unwrap());
    let secp256k1_engine = Secp256k1::new();
    Address::p2tr(&secp256k1_engine, internal_key, None, ctx.bitcoin_network).to_string()
}