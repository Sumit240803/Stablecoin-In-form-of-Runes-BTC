use ic_cdk::{
    bitcoin_canister::{
        bitcoin_get_current_fee_percentiles, GetCurrentFeePercentilesRequest, MillisatoshiPerByte,
    },
    update,
};

use crate::BTC_CONTEXT;

#[update]
pub async fn get_current_fee_percentiles() -> Vec<MillisatoshiPerByte> {
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());
    let response = bitcoin_get_current_fee_percentiles(&GetCurrentFeePercentilesRequest {
        network: ctx.network,
    })
    .await
    .unwrap();
    response
}
