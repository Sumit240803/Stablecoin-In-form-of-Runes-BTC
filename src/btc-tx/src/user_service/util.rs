use candid::Principal;
use sha2::Digest;
use sha3::Sha3_256;


pub fn principal_to_account(principal : Principal)-> u32{
    let hash = Sha3_256::digest(principal.as_slice());
    u32::from_be_bytes([hash[0],hash[1],hash[2],hash[3]])
}