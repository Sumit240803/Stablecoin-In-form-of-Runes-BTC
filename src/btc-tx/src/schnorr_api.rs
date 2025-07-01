

use std::{cell::RefCell, collections::HashMap};



use ic_cdk::{ management_canister::{schnorr_public_key, sign_with_schnorr, Bip341, SchnorrAlgorithm, SchnorrAux, SchnorrKeyId, SchnorrPublicKeyArgs, SignWithSchnorrArgs}};

use crate::{BitcoinContext};
type DerivationPath = Vec<Vec<u8>>;
type SchnorrKey = Vec<u8>;

thread_local! {
    static SCHNORR_KEY_CACHE : RefCell<HashMap<DerivationPath,SchnorrKey>> = RefCell::new(HashMap::new());
}

pub async fn get_schnorr_public_key(
    ctx: &BitcoinContext,
    derivation_path: Vec<Vec<u8>>,
) -> Vec<u8> {
    // Retrieve and return already stored public key
    if let Some(key) = SCHNORR_KEY_CACHE.with_borrow(|map| map.get(&derivation_path).cloned()) {
        return key;
    }

    let public_key =schnorr_public_key(&SchnorrPublicKeyArgs {
        canister_id: None,
        derivation_path: derivation_path.clone(),
        key_id: SchnorrKeyId {
            name: ctx.key_name.to_string(),
            algorithm: SchnorrAlgorithm::Bip340secp256k1,
        },
    })
    .await
    .unwrap()
    .public_key;

    // Cache the public key
    SCHNORR_KEY_CACHE.with_borrow_mut(|map| {
        map.insert(derivation_path, public_key.clone());
    });

    public_key
}




pub async fn schnorr_sign(
    key_name : String,
    derivation_path : Vec<Vec<u8>>,
    merkle_root_hash : Option<Vec<u8>>,
    message : Vec<u8>
)-> Vec<u8>{
    let aux = merkle_root_hash.map(|bytes|{
        SchnorrAux::Bip341(Bip341{
            merkle_root_hash : bytes
        })
    });
    sign_with_schnorr(&SignWithSchnorrArgs{
        message,
        derivation_path,
        key_id : SchnorrKeyId{
            name : key_name,
            algorithm : SchnorrAlgorithm::Bip340secp256k1
        },
        aux
    })
    .await
    .unwrap()
    .signature
}

pub async fn mock_sign_with_schnorr(
    _key_name: String,
    _derivation_path: Vec<Vec<u8>>,
    _merkle_root_hash: Option<Vec<u8>>,
    _message_hash: Vec<u8>,
) -> Vec<u8> {
    vec![255; 64]
}