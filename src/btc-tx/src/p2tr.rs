use bitcoin::{key::Secp256k1, taproot::{TaprootBuilder, TaprootSpendInfo}, PublicKey, ScriptBuf, XOnlyPublicKey};




pub fn create_taproot_spend_info(
    internal_key_bytes : &[u8],
    script_key_bytes : &[u8],

)-> TaprootSpendInfo{
    let internal_key = XOnlyPublicKey::from_slice(internal_key_bytes).unwrap();

    let spend_script = create_spend_script(script_key_bytes);

    let secp256k1_engine = Secp256k1::new();
    TaprootBuilder::new()
    .add_leaf(0, spend_script.clone())
    .expect("adding leaf should work")
    .finalize(&secp256k1_engine, internal_key)
    .expect("finalizing taproot builder should work")
}

pub fn create_spend_script(script_key_bytes : &[u8])->ScriptBuf{
    let script_key = XOnlyPublicKey::from(PublicKey::from_slice(script_key_bytes).unwrap());
    bitcoin::blockdata::script::Builder::new()
    .push_x_only_key(&script_key)
    .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKSIG)
    .into_script()
}