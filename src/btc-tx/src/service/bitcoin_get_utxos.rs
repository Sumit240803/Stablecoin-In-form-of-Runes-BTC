use ic_cdk::{api::management_canister::bitcoin::{bitcoin_get_utxos, GetUtxosRequest}, update};

use crate::BTC_CONTEXT;



#[update]
pub async fn get_total_utxos(address : String)->u64{
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());
    let(response,)= bitcoin_get_utxos(GetUtxosRequest{
        address,
        network : ctx.network,
        filter : None
    })
    .await
    .unwrap();
    response.utxos.iter().map(|utxo| utxo.value).sum()

}