use candid::{Decode, Encode};
use ic_cdk::{api::{time, caller}, query, update};
use serde::{Deserialize, Serialize}; 

type UsdbAmount = u64;


#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserBalance {
    pub principal: candid::Principal,
    pub amount: UsdbAmount,
}

static mut TOTAL_SUPPLY: UsdbAmount = 0;
static mut USER_BALANCES: Vec<UserBalance> = Vec::new();



#[query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

/// Returns the current total supply of USDB.
#[query]
fn get_total_supply() -> UsdbAmount {
    unsafe { TOTAL_SUPPLY }
}

/// A placeholder mint function. In reality, this would involve BTC collateral.
/// For now, it just increases total supply and a user's balance.
#[update]
fn mint_usdb(amount: UsdbAmount) -> UsdbAmount {
    // In a real scenario, `caller()` would be verified against BTC deposit.
    let minter = caller();

    unsafe {
        TOTAL_SUPPLY += amount;

        // Find or create user balance entry
        if let Some(balance_entry) = USER_BALANCES.iter_mut().find(|b| b.principal == minter) {
            balance_entry.amount += amount;
        } else {
            USER_BALANCES.push(UserBalance {
                principal: minter,
                amount,
            });
        }

        TOTAL_SUPPLY // Return new total supply
    }
}

/// A placeholder burn function. In reality, this would involve releasing BTC collateral.
/// For now, it just decreases total supply and a user's balance.
#[update]
fn burn_usdb(amount: UsdbAmount) -> UsdbAmount {
    let burner = caller();

    unsafe {
        if let Some(balance_entry) = USER_BALANCES.iter_mut().find(|b| b.principal == burner) {
            if balance_entry.amount >= amount {
                balance_entry.amount -= amount;
                TOTAL_SUPPLY -= amount;
            } else {
                // Handle insufficient balance error (in real world, return Result<T, E>)
                ic_cdk::trap("Insufficient USDB balance to burn.");
            }
        } else {
            ic_cdk::trap("No USDB balance found for caller.");
        }
        TOTAL_SUPPLY // Return new total supply
    }
}

/// Get a user's USDB balance.
#[query]
fn get_my_balance() -> UsdbAmount {
    let current_caller = caller();
    unsafe {
        USER_BALANCES
            .iter()
            .find(|b| b.principal == current_caller)
            .map_or(0, |b| b.amount)
    }
}

