/*use candid::CandidType;
use ic_cdk::update;
use serde::Deserialize;

use crate::{common::DerivationPath, ecdsa::get_ecdsa_public_key, schnorr_api::get_schnorr_public_key, user_service::util::principal_to_account, BTC_CONTEXT};

#[derive(CandidType, Deserialize, Debug)]
pub struct EtchingArgs {
    pub divisibility: u8,
    pub symbol: u32,
    pub rune: String,
    pub amount: u128,
    pub cap: u128,
    pub turbo: bool,
    pub premine: u128,
    pub height: Option<(u64, u64)>,
    pub offset: Option<(u64, u64)>,
    pub fee_rate: Option<u64>,
}

*/