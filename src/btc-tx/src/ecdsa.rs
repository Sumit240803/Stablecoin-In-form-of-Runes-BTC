use crate::BitcoinContext;
use bitcoin::secp256k1::ecdsa::Signature;
use ic_cdk::api::management_canister::{
    self
};
use ic_cdk::api::management_canister::ecdsa::{
    EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument, SignWithEcdsaArgument,
};

use std::{cell::RefCell, collections::HashMap};

type DerivationPath = Vec<Vec<u8>>;
type EcdsaKey = Vec<u8>;
thread_local! {
    static ECDSA_KEY_CACHE: RefCell<HashMap<DerivationPath, EcdsaKey>> = RefCell::new(HashMap::new());
}


pub async fn get_ecdsa_public_key(ctx: &BitcoinContext, derivation_path: Vec<Vec<u8>>) -> Vec<u8> {
    // Check in-memory cache first.
    if let Some(key) = ECDSA_KEY_CACHE.with_borrow(|map: &HashMap<Vec<Vec<u8>>, Vec<u8>>| map.get(&derivation_path).cloned()) {
        return key;
    }

    // Request the ECDSA public key from the ECDSA API.
    let (response,) = management_canister::ecdsa::ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: derivation_path.clone(),
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: ctx.key_name.to_string(),
        },
    })
    .await
    .unwrap();
    let public_key = response.public_key;

    // Store it in the in-memory cache for future reuse.
    ECDSA_KEY_CACHE.with_borrow_mut(|map| {
        map.insert(derivation_path, public_key.clone());
    });

    public_key
}

pub async fn sign_with_ecdsa(
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
    message_hash: Vec<u8>,
) -> Signature {
    let (response,) = management_canister::ecdsa::sign_with_ecdsa(SignWithEcdsaArgument {
        message_hash,
        derivation_path,
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: key_name,
        },
    })
    .await
    .unwrap();
    let signature = response.signature;

    Signature::from_compact(&signature).unwrap()
}


pub async fn mock_sign_with_ecdsa(
    _key_name: String,
    _derivation_path: Vec<Vec<u8>>,
    _signing_data: Vec<u8>,
) -> Signature {
    let r_s = [1u8; 64];
    Signature::from_compact(&r_s).unwrap()
}