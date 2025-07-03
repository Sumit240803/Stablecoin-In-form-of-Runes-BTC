use bitcoin::{
    consensus::serialize,
    secp256k1::{PublicKey, Secp256k1},
    Address, XOnlyPublicKey,
};
use candid::CandidType;
use ic_cdk::{
    bitcoin_canister::{
        bitcoin_get_utxos, bitcoin_send_transaction, GetUtxosRequest, SendTransactionRequest,
    },
    trap, update,
};
use serde::{Deserialize, Serialize};


use crate::{
    common::{get_fee_per_byte, DerivationPath, PrimaryOutput},
    p2tr,
    runes::{build_etching_script, Etching},
    schnorr_api::{get_schnorr_public_key, schnorr_sign},
    BTC_CONTEXT,
};
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct RuneArgument {
    name: String,
    divisibility: u8,
    premine: u128,
    symbol: Option<String>, // Use String instead of char for Candid compatibility
    turbo: bool
}
#[update]
pub async fn etch_rune(args: RuneArgument) -> String {
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());

    // Validate rune name according to protocol rules.
    // Runes use strict naming conventions for consistency.
    if args.name.is_empty() {
        trap("Rune name cannot be empty");
    }

    if args.name.len() > 28 {
        trap("Rune name cannot exceed 28 characters");
    }

    if !args.name.chars().all(|c| c.is_ascii_uppercase()) {
        trap("Rune name must contain only uppercase letters A-Z");
    }

    // Derive the internal key for our Taproot address.
    // Since rune data goes in OP_RETURN (not script), we use simple key-path spending.
    let internal_key_path = DerivationPath::p2tr(0, 0);
    let internal_key = get_schnorr_public_key(&ctx, internal_key_path.to_vec_u8_path()).await;
    let internal_key = XOnlyPublicKey::from(PublicKey::from_slice(&internal_key).unwrap());

    // Create our Taproot address for funding the rune etching.
    // No script commitments needed since rune data goes in OP_RETURN output.
    let secp256k1_engine = Secp256k1::new();
    let own_address = Address::p2tr(&secp256k1_engine, internal_key, None, ctx.bitcoin_network);

    // Query for available funds (UTXOs) to pay for the rune etching.
    // We need existing bitcoin to cover transaction fees and any change.
    let own_utxos = bitcoin_get_utxos(&GetUtxosRequest {
        address: own_address.to_string(),
        network: ctx.network,
        filter: None,
    })
    .await
    .unwrap()
    .utxos;

    // Create the rune etching configuration with fixed parameters.
    // This defines all the token properties that will be permanently recorded.
    let etching = Etching {
        divisibility: args.divisibility,    // No decimal places (whole units only)
        premine: args.premine, // Mint 1M units to the etcher (fixed supply)
        rune_name: args.name.clone(),
        symbol: args.symbol.as_ref().and_then(|s| s.chars().next()), // Convert Option<String> to Option<char>
        terms: None,        // No open minting allowed
        turbo: args.turbo,       // Standard etching mode
        spacers: 0,         // No visual spacers in the name
    };

    // Build the runestone script containing the rune metadata.
    // This creates the OP_RETURN output that defines the new token.
    let runestone_script = build_etching_script(&etching)
        .unwrap_or_else(|e| trap(&format!("Failed to build runestone: {}", e)));

    // Build the rune etching transaction.
    // The transaction includes an OP_RETURN output with the encoded runestone.
    let fee_per_byte = get_fee_per_byte(&ctx).await;
    let (transaction, prevouts) = p2tr::build_transaction(
        &ctx,
        &own_address,
        &own_utxos,
        p2tr::SelectUtxosMode::Single,
        &PrimaryOutput::OpReturn(runestone_script),
        fee_per_byte,
    )
    .await;

    // Sign the rune etching transaction using key-path spending.
    // Simple signature since we're not using any script commitments.
    let signed_transaction = p2tr::sign_transaction_key_spend(
        &ctx,
        &own_address,
        transaction,
        prevouts.as_slice(),
        internal_key_path.to_vec_u8_path(),
        vec![],
        schnorr_sign,
    )
    .await;

    // Broadcast the transaction to the Bitcoin network.
    // Once confirmed, the rune is permanently etched and the tokens are minted.
    bitcoin_send_transaction(&SendTransactionRequest {
        network: ctx.network,
        transaction: serialize(&signed_transaction),
    })
    .await
    .unwrap();

    // Return the transaction ID so users can track their rune etching.
    signed_transaction.compute_txid().to_string()
}
