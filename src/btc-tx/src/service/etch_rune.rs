use std::str::FromStr;

use bitcoin::{
    absolute::LockTime, consensus, hashes::{sha256, Hash}, opcodes, script::{Builder, PushBytes}, secp256k1::{
        constants::SCHNORR_SIGNATURE_SIZE, schnorr, Message, PublicKey, Secp256k1, XOnlyPublicKey,
    }, sighash::{EcdsaSighashType, Prevouts, SighashCache, TapSighashType}, taproot::{ControlBlock, LeafVersion, Signature, TapLeafHash, TaprootBuilder}, transaction::Version, Address, Amount, CompressedPublicKey, FeeRate, OutPoint, Script, ScriptBuf, Sequence, TapSighash, Transaction, TxIn, TxOut, Txid, Witness
};use candid::CandidType;


use hex::ToHex;
use ic_cdk::{api::management_canister::bitcoin::{BitcoinNetwork, Utxo}, update};
use ordinals::{Artifact, Etching, Rune, Runestone, SpacedRune, Terms};
use serde::Deserialize;

use crate::{common::DerivationPath, ecdsa::{get_ecdsa_public_key, sign_with_ecdsa}, schnorr_api::{self, get_schnorr_public_key}, service::{bitcoin_get_utxos::get_utxos, get_balance::get_balance, get_p2wpkh_address::get_p2wpkh_address}, tags::Tag, BTC_CONTEXT};
pub const SIG_HASH_TYPE: EcdsaSighashType = EcdsaSighashType::All;
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
#[update]
pub async fn etch_rune(mut args : EtchingArgs)->(String,String){
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());
    let _caller = ic_cdk::id();
    args.rune = args.rune.to_ascii_uppercase();
    let derivation_path = DerivationPath::p2wpkh(0, 0);
    let ecdsa_public_key = get_ecdsa_public_key(&ctx, derivation_path.to_vec_u8_path().clone()).await;
    let schnorr_public_key = get_schnorr_public_key(&ctx, derivation_path.to_vec_u8_path().clone()).await.expect("Failed to get schnorr public key");
    let caller_p2wpkh_address = get_p2wpkh_address().await;
    let balance = get_balance(caller_p2wpkh_address.clone()).await;
    if balance < 1000_0000{
        ic_cdk::trap("Not enough Balance")
    }
    let utxos = get_utxos(caller_p2wpkh_address.clone()).await;
    check_etching(utxos.tip_height, &args);
    let (_commit_tx_address, commit_tx, reveal_tx) = build_and_sign_etching_transaction(
        &derivation_path.to_vec_u8_path(),
        &utxos.utxos,
        &ecdsa_public_key,
        &schnorr_public_key,
        caller_p2wpkh_address,
        args,
    )
    .await;
    let commit_txid = send_bitcoin_transaction(commit_tx).await;
    let reveual_txid = send_bitcoin_transaction(reveal_tx).await;


    (commit_txid,reveual_txid)


    
}

pub fn check_etching(height : u32,arg : &EtchingArgs){
   /*  if arg.height.is_none() && arg.offset.is_none(){
        ic_cdk::trap("No mint term selected")
    }*/
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());
    let network =match ctx.network{
        BitcoinNetwork::Mainnet =>bitcoin::Network::Bitcoin,
        BitcoinNetwork::Testnet => bitcoin::Network::Testnet,
        BitcoinNetwork::Regtest => bitcoin::Network::Regtest,
    };
    let minimum = Rune::minimum_at_height(network, ordinals::Height(height));
    let SpacedRune{rune,spacers: _}= SpacedRune::from_str(&arg.rune).unwrap();
    if rune < minimum{
        ic_cdk::trap("Rune is less than Minimum")
    }
    if rune.is_reserved(){
        ic_cdk::trap("Rune is reserved")
    }
    if char::from_u32(arg.symbol).is_none(){
        ic_cdk::trap("Failed to validate symbol")
    }
    if arg.amount == 0 || arg.cap == 0 {
        ic_cdk::trap("Can't be Zero")
    }
    if arg.divisibility > 38 {
        ic_cdk::trap("Exceeds max allowed divisibility")
    }
     if let Some((start, stop)) = arg.height {
        if start >= stop {
            ic_cdk::trap("Height Start must be lower than Height Stop")
        }
    }
    if let Some((start, stop)) = arg.offset {
        if start >= stop {
            ic_cdk::trap("Offset Start must be lower than Offset Stop")
        }
    }
    
}



pub async fn build_and_sign_etching_transaction(
    derivation_path : &Vec<Vec<u8>>,
    owned_utxos : &[Utxo],
    ecdsa_public_key : &[u8],
    schnorr_public_key: &[u8],
    caller_p2wkh_address:String,
    etching_args: EtchingArgs,
)-> (Address,Transaction,Transaction){

    //Rune info from arguments
    let SpacedRune { rune, spacers } = SpacedRune::from_str(&etching_args.rune).unwrap();
    let symbol = char::from_u32(etching_args.symbol).unwrap();
    let secp256k1 = Secp256k1::new();

    let schnorr_public_key: XOnlyPublicKey =
        PublicKey::from_slice(schnorr_public_key).unwrap().into();


    /*This is used in Bitcoin Runes / Ordinals / inscriptions to mark the start of a specific protocol payload 
    (like "ord" protocol for inscriptions or runes), so that when parsing the script later, tools can recognize 
    which protocol it belongs to.*/
    const PROTOCOL_ID: [u8; 3] = *b"ord";

    /*This Rust code is building a Taproot script (called the reveal_script) that will be used in a Bitcoin transaction, 
    specifically for Runes etching or similar operations where data is encoded in a spendable Taproot leaf. */
    /*
    .push_slice(schnorr_public_key.serialize())
This pushes the serialized x-only Schnorr public key (32 bytes) onto the Bitcoin script stack.

This is the public key that will be used to verify the signature in the next line.


    .push_opcode(opcodes::all::OP_CHECKSIG)
This opcode pops the top item (signature) and uses the pushed public key to verify the signature.


 .push_opcode(opcodes::OP_FALSE)
Pushes a 0 (false) onto the stack.

This means: the script will now continue execution down the OP_IF branch only if the stack top is false (i.e., the signature was invalid).

But in the Taproot context, this OP_FALSE OP_IF pattern is a standard trick to allow the script to hold data that will never be executed — 
it’s just committed to and revealed.

.push_opcode(opcodes::all::OP_IF)
This opens an OP_IF block. The next instructions will only be executed if the top stack item is true.

But because you just pushed OP_FALSE, this branch is never executed — it’s purely used as a data-hiding structure.

.push_slice(PROTOCOL_ID)
This pushes the byte sequence [b'o', b'r', b'd'] (i.e., "ord") onto the stack within the OP_IF block.

So you're basically hiding the "ord" protocol ID inside a false IF block to commit to it without having it affect script execution.

Script Structure

<pubkey>
OP_CHECKSIG
OP_FALSE
OP_IF
  "ord"
  ...
OP_ENDIF


 */
    let mut reveal_script = Builder::new()
        .push_slice(schnorr_public_key.serialize())
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(PROTOCOL_ID);

    /*
    What this does:
    Encodes and pushes a "Rune" tag (byte 13) and its associated commitment() data into the Taproot script being built.

    Why it's useful:
    This enables on-chain storage of Rune metadata in a structured, tag-based format so off-chain parsers (like wallets or indexers)
    can read and interpret Runes from the Taproot script without executing it. */

    Tag::Rune.encode(&mut reveal_script, &Some(rune.commitment()));

    //ENDIF opcode which ends the script stack 
    /*Full Script now 
        <schnorr_pubkey> OP_CHECKSIG
        OP_FALSE
        OP_IF
        "ord"         // Protocol identifier
        13 <commitment_bytes> // Rune metadata
        OP_ENDIF
 */
    let reveal_script = reveal_script
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();


    //Creating pay to taproot address from reveal script 

    let taproot_send_info = TaprootBuilder::new()
        .add_leaf(0, reveal_script.clone())
        .unwrap()
        .finalize(&secp256k1, schnorr_public_key)
        .unwrap();
    //Control Block which will be used to prove : 
    //Script was committed inside taproot output
    //Contains merkle path and leaf version
    let control_block = taproot_send_info
        .control_block(&(reveal_script.clone(), LeafVersion::TapScript))
        .unwrap();
    let ctx = BTC_CONTEXT.with(|ctx| ctx.get());
    let network =match ctx.network{
        BitcoinNetwork::Mainnet =>bitcoin::Network::Bitcoin,
        BitcoinNetwork::Testnet => bitcoin::Network::Testnet,
        BitcoinNetwork::Regtest => bitcoin::Network::Regtest,
    };
    let caller_address = Address::from_str(&caller_p2wkh_address)
        .unwrap()
        .assume_checked();
    let commit_tx_address = Address::p2tr_tweaked(taproot_send_info.output_key(), network);

    /*
    reveal_input: starts with a placeholder input (OutPoint::null() is a dummy — probably will be replaced later).

    reveal_output: will be populated if needed.

    pointer: tracks the index of a special output (used for linking to Rune balance).

 */
    let mut reveal_input = vec![OutPoint::null()];
    let mut reveal_output = vec![];
    let mut pointer = None;
    if etching_args.premine > 0 {
        reveal_output.push(TxOut {
            script_pubkey: caller_address.script_pubkey(),
            value: Amount::from_sat(10_000),
        });
        pointer = Some(reveal_output.len() as u32 - 1u32);
    }

    /*
    This extracts optional range values from etching_args:

height: defines which block height range the rune is valid for (e.g., mintable from block 900k to 910k).

offset: defines a transaction offset range within a block (rarely used).

If neither is set, the program halts with a trap: "No Term Set".

 */
    let (height, offset) = match (etching_args.height, etching_args.offset) {
        (Some((start, stop)), None) => {
            let height = (Some(start), Some(stop));
            (height, (None, None))
        }
        (None, Some((start, stop))) => {
            let offset = (Some(start), Some(stop));
            ((None, None), offset)
        }
        (Some((h_start, h_stop)), Some((o_start, o_stop))) => {
            let height = (Some(h_start), Some(h_stop));
            let offset = (Some(o_start), Some(o_stop));
            (height, offset)
        }
        (None, None) => ic_cdk::trap("No Term Set"),
    };

    let runestone = Runestone {
        etching: Some(Etching {
            rune: Some(rune),
            symbol: Some(symbol),
            divisibility: Some(etching_args.divisibility),
            premine: Some(etching_args.premine),
            spacers: Some(spacers),
            turbo: etching_args.turbo,
            terms: Some(Terms {
                cap: Some(etching_args.cap),
                amount: Some(etching_args.amount),
                height,
                offset,
            }),
        }),
        edicts: vec![],
        mint: None,
        pointer,
    };
    /*
    encipher() encodes the Runestone into a Bitcoin script.

    This typically produces a script like:
    OP_RETURN <data>
 */
    let script_pubkey = runestone.encipher();
    if script_pubkey.len() > 82 {
        ic_cdk::trap("Exceeds OP_RETURN size of 82")
    }
    reveal_output.push(TxOut {
        script_pubkey,
        value: Amount::from_sat(0),
    });
    //Sets the fee rate in sats per vbyte, defaulting to 10 if not specified.

//Internally converts to sats per weight unit (kwu) since SegWit transactions are weight-based.


    let fee_rate = FeeRate::from_sat_per_vb(etching_args.fee_rate.unwrap_or(10)).unwrap();


    /*
    This calls the build_reveal_transaction() function with:

commit_input_index: index of the Taproot-spending input (usually 0).

control_block: proof to spend from Taproot leaf.

fee_rate: for estimating the fee.

reveal_output: the OP_RETURN + premine outputs.

reveal_input: dummy inputs (e.g., OutPoint::null() for now).

reveal_script: the full script being revealed (e.g., contains OP_CHECKSIG + rune metadata). */


    let (_, reveal_fee) = build_reveal_transaction(
        0,
        &control_block,
        fee_rate,
        reveal_output.clone(),
        reveal_input.clone(),
        &reveal_script,
    );

    /*Gathering the UTXOS  */
    let mut utxos_to_spend = vec![];
    let mut total_spent = 0;
    owned_utxos.iter().for_each(|utxo| {
        total_spent += utxo.value;
        utxos_to_spend.push(utxo);
    });
    /*Building transaction input with empty witness and script_signature - to be filled when signing
    
     */

    let input = utxos_to_spend
        .iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint::new(
                Txid::from_raw_hash(Hash::from_slice(&utxo.outpoint.txid).unwrap()),
                utxo.outpoint.vout,
            ),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
            script_sig: ScriptBuf::new(),
        })
        .collect::<Vec<TxIn>>();
        /*Committing input transaction using bitcoin transaction */
    let mut commit_tx = Transaction {
        input,
        output: vec![TxOut {
            script_pubkey: commit_tx_address.script_pubkey(),
            value:Amount::from_sat(total_spent),
        }],
        lock_time: LockTime::ZERO,
        version: bitcoin::transaction::Version(2),
    };
    

    let sig_bytes = 73;
    let estimated_vsize = commit_tx.vsize() + utxos_to_spend.len() * sig_bytes / 4;
let commit_fee = fee_rate.fee_vb(estimated_vsize as u64).unwrap();
    ic_cdk::println!("commit fee: {}\nreveal fee: {}", commit_fee, reveal_fee);
    commit_tx.output[0].value = Amount::from_sat(total_spent - commit_fee.to_sat() - reveal_fee.to_sat());
    
    let mut commit_tx_cache = SighashCache::new(commit_tx.clone());
    for (index, input) in commit_tx.input.iter_mut().enumerate() {
        let utxo_value_set = utxos_to_spend[index].value;
        let sighash = commit_tx_cache
            .p2wpkh_signature_hash(
                index,
                &caller_address.script_pubkey(),
                Amount::from_sat(utxo_value_set),
                SIG_HASH_TYPE
                
            )
            .unwrap();
        let signature =match sign_with_ecdsa(ctx.key_name.to_owned(),derivation_path.clone(),sighash.to_byte_array().to_vec()).await{
            Ok(sig)=>sig,
            Err(e)=>{
                ic_cdk::println!("ECDSA signing failed: {}",e);
                return (
    Address::p2wpkh(&CompressedPublicKey::from_slice(ecdsa_public_key).unwrap(),network), // or any fallback address
    Transaction { input: vec![], output: vec![], lock_time: LockTime::ZERO, version: Version(2) },
    Transaction { input: vec![], output: vec![], lock_time: LockTime::ZERO, version: Version(2) }
);

            }
        };
        let compact: [u8; 64] = signature.serialize_compact();
        let signature_bytes = compact.to_vec();

        let der_signature = sec1_to_der(signature_bytes);
        let mut sig_with_hashtype = der_signature;
        sig_with_hashtype.push(SIG_HASH_TYPE.to_u32() as u8);
        input.script_sig = ScriptBuf::builder()
            .push_slice::<&PushBytes>(sig_with_hashtype.as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(ecdsa_public_key.try_into().unwrap())
            .into_script();
        input.witness.clear();

        
    }
    
    let (vout, _) = commit_tx
        .output
        .iter()
        .enumerate()
        .find(|(_vout, output)| output.script_pubkey == commit_tx_address.script_pubkey())
        .unwrap();
    reveal_input[0] = OutPoint {
        txid: commit_tx.compute_txid(),
        vout: vout as u32,
    };
    reveal_output.push(TxOut {
        script_pubkey: caller_address.script_pubkey(),
        value: Amount::from_sat(total_spent - commit_fee.to_sat() - reveal_fee.to_sat()),
    });

    let mut reveal_tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: LockTime::ZERO,
        input: reveal_input
            .iter()
            .map(|outpoint| TxIn {
                previous_output: *outpoint,
                witness: Witness::new(),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::from_height(Runestone::COMMIT_CONFIRMATIONS - 1),
            })
            .collect(),
        output: reveal_output,
    };

    for output in reveal_tx.output.iter() {
        if output.value < Amount::from_sat(output.script_pubkey.minimal_non_dust().to_sat() ){
            ic_cdk::trap("commit txn output would be dust")
        }
    }

    let mut sighash_cache = SighashCache::new(&mut reveal_tx);
    let mut signing_data = vec![];
    let leaf_hash = TapLeafHash::from_script(&reveal_script, LeafVersion::TapScript);
    sighash_cache
        .taproot_encode_signing_data_to(
            &mut signing_data,
            0,
            &Prevouts::All(&[commit_tx.output[vout].clone()]),
            None,
            Some((leaf_hash, 0xFFFFFFFF)),
            TapSighashType::Default,
        )
        .unwrap();
    /*let mut hashed_tag = sha256::Hash::hash(b"TapSighash").to_byte_array().to_vec();
    let mut prefix = hashed_tag.clone();*/
    //prefix.append(&mut hashed_tag);
    //let signing_data: Vec<_> = prefix.iter().chain(signing_data.iter()).cloned().collect();
    let sig_hash = TapSighash::hash(&signing_data).to_byte_array();
    let schnorr_signature =schnorr_api::schnorr_sign(sig_hash.to_vec(), derivation_path.to_vec()).await;
    ic_cdk::println!("sig size: {}", schnorr_signature.len());
    ic_cdk::print("Schnorr Signature successful");
    // Verify the signature to be sure that signing works
    let secp = bitcoin::secp256k1::Secp256k1::verification_only();
    ic_cdk::print("Verification - secp");

    let sig_ = schnorr::Signature::from_slice(&schnorr_signature).unwrap();
    println!("Signautre from slice : {}", sig_);
    //let digest = sha256::Hash::hash(&signing_data).to_byte_array();
    let msg = Message::from_digest_slice(&sig_hash).unwrap();
    ic_cdk::println!("Signature: {:?}", sig_);
    //ic_cdk::println!("Digest: {:?}", digest);
ic_cdk::println!("Digest: {:?}", msg);
ic_cdk::println!("Public Key: {:?}", schnorr_public_key);
let x_pub_key = XOnlyPublicKey::from(schnorr_public_key);
println!("x_only {:?}",x_pub_key.serialize());
ic_cdk::println!("Signature valid? {:?}",match secp.verify_schnorr(&sig_, &msg, &x_pub_key) {
    Ok(_) => ic_cdk::println!("✅ Signature verified successfully."),
    Err(e) => ic_cdk::trap(&format!("❌ Signature verification failed: {}", e)),
});

    assert!(secp
        .verify_schnorr(&sig_, &msg, &x_pub_key)
        .is_ok());

    let witness = sighash_cache.witness_mut(0).unwrap();
    witness.push(
        Signature {
            signature: schnorr::Signature::from_slice(&schnorr_signature).unwrap(),
            sighash_type: TapSighashType::Default,
        }
        .to_vec(),
    );
    witness.push(reveal_script);
    witness.push(&control_block.serialize());
    if Runestone::decipher(&reveal_tx).unwrap() != Artifact::Runestone(runestone) {
        ic_cdk::trap("Runestone mismatched")
    }
    let commit_tx_bytes = consensus::serialize(&commit_tx);
    let reveal_tx_bytes = consensus::serialize(&reveal_tx);
    ic_cdk::println!("Commit tx bytes: {}", hex::encode(commit_tx_bytes));
    ic_cdk::println!("Reveal tx bytes: {}", hex::encode(reveal_tx_bytes));
    (commit_tx_address, commit_tx, reveal_tx)
}


pub fn sec1_to_der(sec1_signature: Vec<u8>) -> Vec<u8> {
    let r: Vec<u8> = if sec1_signature[0] & 0x80 != 0 {
        // r is negative. Prepend a zero byte.
        let mut tmp = vec![0x00];
        tmp.extend(sec1_signature[..32].to_vec());
        tmp
    } else {
        // r is positive.
        sec1_signature[..32].to_vec()
    };

    let s: Vec<u8> = if sec1_signature[32] & 0x80 != 0 {
        // s is negative. Prepend a zero byte.
        let mut tmp = vec![0x00];
        tmp.extend(sec1_signature[32..].to_vec());
        tmp
    } else {
        // s is positive.
        sec1_signature[32..].to_vec()
    };

    // Convert signature to DER.
    vec![
        vec![0x30, 4 + r.len() as u8 + s.len() as u8, 0x02, r.len() as u8],
        r,
        vec![0x02, s.len() as u8],
        s,
    ]
    .into_iter()
    .flatten()
    .collect()
}




pub fn build_reveal_transaction(
    commit_input_index: usize,
    control_block: &ControlBlock,
    fee_rate: FeeRate,
    output: Vec<TxOut>,
    input: Vec<OutPoint>,
    script: &Script,
) -> (Transaction, Amount) {

    /*Creates a transaction with:

The provided inputs (usually a dummy UTXO in reveal_input)

The provided outputs (OP_RETURN + possible premine)

Empty witness fields (initially)

sequence set for RBF & confirmation control

 */
    let reveal_txn = Transaction {
        input: input
            .into_iter()
            .map(|previous_output| TxIn {
                previous_output,
                script_sig: Script::builder().into_script(),
                witness: Witness::new(),
                sequence: Sequence::from_height(Runestone::COMMIT_CONFIRMATIONS - 1),
            })
            .collect(),
        output,
        lock_time: LockTime::ZERO,
        version: bitcoin::transaction::Version(2),
    };
    let fee = {
        let mut reveal_txn_clone = reveal_txn.clone();
        for (current_index, txin) in reveal_txn_clone.input.iter_mut().enumerate() {
            if current_index == commit_input_index {
                txin.witness.push(
                    Signature::from_slice(&[0; SCHNORR_SIGNATURE_SIZE])
                        .unwrap()
                        .to_vec(),
                );
                txin.witness.push(script);
                txin.witness.push(control_block.serialize());
            } else {
                txin.witness = Witness::from_slice(&[&[0; SCHNORR_SIGNATURE_SIZE]]);
            }
        }
        Amount::from_sat(
            (fee_rate.to_sat_per_kwu() as f64 * reveal_txn_clone.vsize() as f64).round() as u64,
        )
    };
    (reveal_txn, fee)
}



pub async fn send_bitcoin_transaction(txn: Transaction) -> String {
    let transaction = bitcoin::consensus::serialize(&txn);
    let ctx = BTC_CONTEXT.with(|cts| cts.get());
    let network = ctx.network;
    ic_cdk::api::management_canister::bitcoin::bitcoin_send_transaction(
        ic_cdk::api::management_canister::bitcoin::SendTransactionRequest {
            transaction,
            network,
        },
    )
    .await
    .unwrap();
    txn.compute_txid().encode_hex()
}