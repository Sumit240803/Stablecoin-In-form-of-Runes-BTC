use bitcoin::{Address, CompressedPublicKey};
use ic_cdk::{api::caller, update};

use crate::{common::DerivationPath, ecdsa::get_ecdsa_public_key, user_service::util::principal_to_account, BTC_CONTEXT};
#[update]
pub async fn get_deposit_address()-> String{
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());
    let principal = caller();
    let account = principal_to_account(principal);
    let derivation_path = DerivationPath::p2wpkh(account, 0).to_vec_u8_path();
    let pub_key = get_ecdsa_public_key(&ctx, derivation_path).await;
    let pub_key = CompressedPublicKey::from_slice(&pub_key).unwrap();
    Address::p2wpkh(&pub_key, ctx.bitcoin_network).to_string()
}