use ic_cdk_macros::{init, query};
use candid::{candid_method, export_service, Principal};
use std::cell::RefCell;

thread_local! {
    static OWNER: RefCell<Option<Principal>> = RefCell::new(None);
}

#[init]
fn init() {
    let caller = ic_cdk::api::caller();
    OWNER.with(|o| *o.borrow_mut() = Some(caller));
}

#[query]
#[candid_method(query)]
fn get_owner() -> Option<Principal> {
    OWNER.with(|o| *o.borrow())
}

// This macro generates the candid interface file (the .did file)
export_service!();
