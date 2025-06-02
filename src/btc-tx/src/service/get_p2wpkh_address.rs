
use bitcoin::{Address, CompressedPublicKey};
use ic_cdk::update;

use crate::{common::DerivationPath, ecdsa::get_ecdsa_public_key, BTC_CONTEXT};

#[update]
pub async fn get_p2wpkh_address() -> String {
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());

  
    let derivation_path = DerivationPath::p2wpkh(0, 0);

    let public_key = get_ecdsa_public_key(&ctx, derivation_path.to_vec_u8_path()).await;

    let public_key = CompressedPublicKey::from_slice(&public_key).unwrap();

  
    Address::p2wpkh(&public_key, ctx.bitcoin_network).to_string()
}