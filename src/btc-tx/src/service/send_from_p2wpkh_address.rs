/*use std::str::FromStr;

use bitcoin::{consensus::serialize, Address, CompressedPublicKey, PublicKey};
use ic_cdk::{bitcoin_canister::{bitcoin_get_utxos, bitcoin_send_transaction, GetUtxosRequest, SendTransactionRequest}, trap, update};
//use ic_cdk::{api::management_canister::bitcoin::{bitcoin_get_utxos, bitcoin_send_transaction, GetUtxosRequest, SendTransactionRequest}, trap, update};

use crate::{common::{get_fee_per_byte, DerivationPath}, ecdsa::{get_ecdsa_public_key, sign_with_ecdsa_fn}, p2wpkh, SendRequest, BTC_CONTEXT};

#[update]
pub async fn send_from_p2wpkh_address(request : SendRequest)->String{
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());
    if request.amount_in_satoshi ==0{
        trap("Amount must be grater than 0 satoshi");
    }

    let dst_address = Address::from_str(&request.destination_address)
    .unwrap()
    .require_network(ctx.bitcoin_network)
    .unwrap();
    let derivation_path =DerivationPath::p2wpkh(0, 0);
    let own_public_key = get_ecdsa_public_key(&ctx, derivation_path.to_vec_u8_path()).await;
    let own_compressed_pub_key = CompressedPublicKey::from_slice(&own_public_key).unwrap();

    let own_public_key = PublicKey::from_slice(&own_public_key).unwrap();
    let own_address = Address::p2wpkh(&own_compressed_pub_key, ctx.bitcoin_network);
    let response = bitcoin_get_utxos(&GetUtxosRequest{
        address : own_address.to_string(),
        network : ctx.network,
        filter : None
    })
    .await
    .unwrap();
    let own_utxos = &response.utxos;

    let fee_per_byte = get_fee_per_byte(&ctx).await;
    let (transaction,prevouts) = p2wpkh::build_transaction(&ctx, &own_public_key, &own_address, own_utxos, &dst_address, request.amount_in_satoshi, fee_per_byte).await;
let signed_transactions = p2wpkh::sign_transaction(
    &ctx,
    &own_public_key,
    &own_address,
    transaction,
    &prevouts,
    derivation_path.to_vec_u8_path(),
    |key_name, path, hash| async move {
        sign_with_ecdsa_fn(key_name, path, hash)
            .await
            .expect("Failed to sign with ECDSA")
    },
).await;

    bitcoin_send_transaction(&SendTransactionRequest{
        network : ctx.network,
        transaction:serialize(&signed_transactions),
    })
    .await
    .unwrap();

    signed_transactions.compute_txid().to_string()
   
}*/