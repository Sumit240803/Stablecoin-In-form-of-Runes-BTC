use bitcoin::{key::Secp256k1, taproot::{TaprootBuilder, TaprootSpendInfo}, PublicKey, ScriptBuf, XOnlyPublicKey};






pub fn create_taproot_spend_info(
    internal_key_bytes: &[u8],
    script_key_bytes: &[u8],
) -> Result<TaprootSpendInfo, String> {
    let internal_key = XOnlyPublicKey::from(PublicKey::from_slice(internal_key_bytes).unwrap());
    

    let spend_script = create_spend_script(script_key_bytes)?;

    let secp256k1_engine = Secp256k1::new();

    TaprootBuilder::new()
        .add_leaf(0, spend_script.clone())
        .map_err(|e| format!("Failed to add leaf: {:?}", e))?
        .finalize(&secp256k1_engine, internal_key)
        .map_err(|e| format!("Failed to finalize taproot builder: {:?}", e))
}

pub fn create_spend_script(script_key_bytes: &[u8]) -> Result<ScriptBuf, String> {
    let script_key = XOnlyPublicKey::from(PublicKey::from_slice(script_key_bytes).unwrap());

    Ok(
        bitcoin::blockdata::script::Builder::new()
            .push_x_only_key(&script_key)
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKSIG)
            .into_script(),
    )
}
