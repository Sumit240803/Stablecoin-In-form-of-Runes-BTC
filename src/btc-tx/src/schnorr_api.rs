/*use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_cdk::{caller, update};
use serde::{Deserialize, Serialize};

use crate::{common::generate_derivation_path};
thread_local! {
    static STATE: RefCell<String> = RefCell::new("aaaaa-aa".to_string());
}
type CanisterId = Principal;
#[derive(CandidType, Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum SchnorrAlgorithm {
    #[serde(rename = "bip340secp256k1")]
    Bip340Secp256k1,
    #[serde(rename = "ed25519")]
    Ed25519,
}

#[derive(CandidType, Serialize, Debug, Clone,Deserialize)]
struct SchnorrKeyId {
    pub algorithm: SchnorrAlgorithm,
    pub name: String,
}
#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct PublicKeyReply {
    pub public_key_hex: String,
}
#[derive(CandidType, Serialize, Debug,Deserialize)]
struct ManagementCanisterSchnorrPublicKeyRequest {
    pub canister_id: Option<CanisterId>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,   
}

#[derive(CandidType, Deserialize, Debug,Serialize)]
struct ManagementCanisterSchnorrPublicKeyReply {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}
#[update]
async fn public_key(algorithm :SchnorrAlgorithm)->Result<PublicKeyReply,String>{
    let principal = caller();
    let request = ManagementCanisterSchnorrPublicKeyRequest {
        canister_id : None,
        derivation_path : generate_derivation_path(&principal),
        key_id :SchnorrKeyIds::TestKeyLocalDevelopment.to_key_id(algorithm),
    };
    let (res,): (ManagementCanisterSchnorrPublicKeyReply,)=
        ic_cdk::call(mgmt_canister_id(), "schnorr_public_key", (request,)).await
        .map_err(|e| format!("schnorr_public_key failed {}",e.1))?;
    Ok(PublicKeyReply { public_key_hex: hex::encode(&res.public_key) })
}


enum SchnorrKeyIds {
    #[allow(unused)]
    ChainkeyTestingCanisterKey1,
    #[allow(unused)]
    TestKeyLocalDevelopment,
    #[allow(unused)]
    TestKey1,
    #[allow(unused)]
    ProductionKey1,
}

impl SchnorrKeyIds{
    fn to_key_id(&self,algorithm :SchnorrAlgorithm)->SchnorrKeyId{
        SchnorrKeyId { algorithm, name: match self {
            Self::ChainkeyTestingCanisterKey1=>"insecure_test_key_1",
            Self::TestKeyLocalDevelopment=>"dfx_test_key",
            Self::TestKey1=>"test_key_1",
            Self::ProductionKey1=>"key_1"
        }.to_string(),
     }
    }
}

fn mgmt_canister_id()->CanisterId{
    STATE.with_borrow(|state| CanisterId::from_text(&state).unwrap())
}*/

use std::{cell::RefCell, collections::HashMap};

use ic_cdk::api::management_canister::schnorr::{schnorr_public_key, SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgument, SignWithSchnorrArgument, SignWithSchnorrResponse};


use crate::{BitcoinContext, BTC_CONTEXT};
type DerivationPath = Vec<Vec<u8>>;
type SchnorrKey = Vec<u8>;

thread_local! {
    static SCHNORR_KEY_CACHE : RefCell<HashMap<DerivationPath,SchnorrKey>> = RefCell::new(HashMap::new());
}

pub async fn get_schnorr_public_key(
    ctx: &BitcoinContext,
    derivation_path: Vec<Vec<u8>>,
) -> Result<Vec<u8>, String> {
    // Return cached key if available
    if let Some(key) = SCHNORR_KEY_CACHE.with_borrow(|map| map.get(&derivation_path).cloned()) {
        ic_cdk::println!("Using cached key: {:?}", key);
        return Ok(key);
    }

    ic_cdk::println!("Fetching new key for path: {:?}", derivation_path);
    
    let arg = SchnorrPublicKeyArgument {
        canister_id: None,
        derivation_path: derivation_path.clone(),
        key_id: SchnorrKeyId {
            name: ctx.key_name.to_string(),
            algorithm: SchnorrAlgorithm::Bip340secp256k1,
        },
    };

    ic_cdk::println!("Sending argument: {:?}", arg);

    let (response,) = schnorr_public_key(arg)
        .await
        .map_err(|e| format!("Schnorr key fetch failed: {:?}", e))?;

    ic_cdk::println!("Raw response: {:?}", response);
    ic_cdk::println!("Public key length: {}", response.public_key.len());
    ic_cdk::println!("Public key bytes: {:?}", response.public_key);

    let public_key = response.public_key;

  

    SCHNORR_KEY_CACHE.with_borrow_mut(|map: &mut HashMap<Vec<Vec<u8>>, Vec<u8>>| {
        map.insert(derivation_path, public_key.clone());
    });

    Ok(public_key)
}




pub async fn schnorr_sign(message: Vec<u8>, derivation_path: Vec<Vec<u8>>) -> Vec<u8> {
    let ctx = BTC_CONTEXT.with(|state| state.get());
    
    ic_cdk::call::<(SignWithSchnorrArgument,), (SignWithSchnorrResponse,)>(
        *ctx.schnorr_canister.as_ref().unwrap(),
        "sign_with_schnorr",
        (SignWithSchnorrArgument {
            message,
            derivation_path,
            key_id : SchnorrKeyId { algorithm: SchnorrAlgorithm::Bip340secp256k1, name: ctx.key_name.to_string() },
        },),
    )
    .await
    .unwrap()
    .0
    .signature
}