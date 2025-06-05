//Assuming BTC transaction successfully sent from user to the deposit address

use candid::Principal;
use ic_cdk::{caller, update};

use crate::INTENTS;

#[update]
pub fn store_intent(address :String, amount : u64){
    let principal: Principal = caller();
    INTENTS.with(|intents| {
        intents.borrow_mut().insert(principal, (address,amount))
    });
}
