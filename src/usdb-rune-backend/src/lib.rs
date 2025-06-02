
use ic_cdk::{api::{caller}, query, update};
use serde::{Deserialize, Serialize}; 
use std::cell::RefCell;

type UsdbAmount = u64;


#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserBalance {
    pub principal: candid::Principal,
    pub amount: UsdbAmount,
}



thread_local! {
    static TOTAL_SUPPLY: RefCell<UsdbAmount> = RefCell::new(0);
    static USER_BALANCES: RefCell<Vec<UserBalance>> = RefCell::new(Vec::new());
}


#[query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

/// Returns the current total supply of USDB.
#[query]
fn get_total_supply() -> UsdbAmount {
    TOTAL_SUPPLY.with(|supply| *supply.borrow())
}

/// A placeholder mint function. In reality, this would involve BTC collateral.
/// For now, it just increases total supply and a user's balance.
#[update]
fn mint_usdb(amount: UsdbAmount) -> UsdbAmount {
    let minter = caller();

    TOTAL_SUPPLY.with(|supply| {
        *supply.borrow_mut() += amount;
    });

    USER_BALANCES.with(|balances| {
        let mut balances = balances.borrow_mut();
        if let Some(entry) = balances.iter_mut().find(|b| b.principal == minter) {
            entry.amount += amount;
        } else {
            balances.push(UserBalance {
                principal: minter,
                amount,
            });
        }
    });

    get_total_supply()
}


/// A placeholder burn function. In reality, this would involve releasing BTC collateral.
/// For now, it just decreases total supply and a user's balance.
#[update]
fn burn_usdb(amount: UsdbAmount) -> UsdbAmount {
    let burner = caller();

    USER_BALANCES.with(|balances| {
        let mut balances = balances.borrow_mut();
        if let Some(entry) = balances.iter_mut().find(|b| b.principal == burner) {
            if entry.amount >= amount {
                entry.amount -= amount;
                TOTAL_SUPPLY.with(|supply| {
                    *supply.borrow_mut() -= amount;
                });
            } else {
                ic_cdk::trap("Insufficient USDB balance to burn.");
            }
        } else {
            ic_cdk::trap("No USDB balance found for caller.");
        }
    });

    get_total_supply()
}


#[query]
fn get_my_balance() -> UsdbAmount {
    let current = caller();
    USER_BALANCES.with(|balances| {
        balances
            .borrow()
            .iter()
            .find(|b| b.principal == current)
            .map_or(0, |b| b.amount)
    })
}
