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

//use ic_cdk::api::management_canister::schnorr::{schnorr_public_key, SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgument, SignWithSchnorrArgument, SignWithSchnorrResponse};


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


/*pub async fn schnorr_sign(message: Vec<u8>, derivation_path: Vec<Vec<u8>>) -> Vec<u8> {
    ic_cdk::println!("Received message len: {}", message.len());
ic_cdk::println!("First 32 bytes: {}", hex::encode(&message[..32]));

    let ctx = BTC_CONTEXT.with(|state| state.get());
    let canister_id = *ctx.schnorr_canister.as_ref().unwrap();
    let args = SignWithSchnorrArgs{
        message,
        derivation_path,
        key_id : SchnorrKeyId { algorithm: SchnorrAlgorithm::Bip340secp256k1, name: ctx.key_name.to_string() },
    };
    
    
    ic_cdk::call::<(SignWithSchnorrArgs,), (SignWithSchnorrResult,)>(
        *ctx.schnorr_canister.as_ref().unwrap(),
        "sign_with_schnorr",
        (SignWithSchnorrArgs{
            message,
            derivation_path,
            key_id : SchnorrKeyId { algorithm: SchnorrAlgorithm::Bip340secp256k1, name: ctx.key_name.to_string() },
        },),
    )
    .await
    .unwrap()
    .0
    .signature
}*/

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