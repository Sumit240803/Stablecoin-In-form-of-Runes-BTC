use crate::BitcoinContext;
use bitcoin::{
    self, absolute::LockTime, blockdata::witness::Witness, hashes::Hash, transaction::Version,
    Address, Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
};
use candid::Principal;
//use candid::Principal;
//use ic_cdk::api::management_canister::bitcoin::{bitcoin_get_current_fee_percentiles, GetCurrentFeePercentilesRequest, Utxo};
use tiny_keccak::{Hasher, Sha3};
//use tiny_keccak::{Hasher, Sha3};
use ic_cdk::bitcoin_canister::{
    bitcoin_get_current_fee_percentiles, GetCurrentFeePercentilesRequest, Utxo,
};
use std::fmt;

/// Purpose field for a BIP-43/44-style derivation path.
/// Determines the address type. Values are defined by:
/// - BIP-44 for P2PKH (legacy): 44'
/// - BIP-84 for P2WPKH (native SegWit): 84'
/// - BIP-86 for P2TR (Taproot): 86'
pub enum Purpose {
    P2PKH,  // BIP-44
    P2WPKH, // BIP-84
    P2TR,   // BIP-86
}

impl Purpose {
    fn to_u32(&self) -> u32 {
        match self {
            Purpose::P2PKH => 44,
            Purpose::P2WPKH => 84,
            Purpose::P2TR => 86,
        }
    }
}

/// Represents a full BIP-32-compatible derivation path:
/// m / purpose' / coin_type' / account' / change / address_index
///
/// This abstraction is suitable for BIP-44 (legacy), BIP-84 (SegWit), and BIP-86 (Taproot),
/// and provides convenience constructors and binary serialization for use with ECDSA/Schnorr
/// key derivation APIs.
pub struct DerivationPath {
    /// Purpose according to BIP-43 (e.g., 44 for legacy, 84 for SegWit, 86 for Taproot)
    purpose: Purpose,

    /// Coin type (0 = Bitcoin mainnet/testnet). Can be extended for altcoins.
    coin_type: u32,

    /// Logical account identifier. Use this to separate multiple user accounts or roles.
    account: u32,

    /// Chain: 0 = external (receive), 1 = internal (change)
    change: u32,

    /// Address index: used to derive multiple addresses within the same account.
    address_index: u32,
}

impl DerivationPath {
    /// Constructs a new derivation path using the given purpose, account, and address index.
    ///
    /// - `purpose`: Determines the address type (BIP-44 for P2PKH, BIP-84 for P2WPKH, BIP-86 for P2TR).
    /// - `account`: Use to separate logical users or wallets. For multi-user wallets, assign each user a unique account number.
    /// - `address_index`: Used to derive multiple addresses within the same account.
    ///
    /// The coin type is set to 0 (Bitcoin), and change is set to 0 (external chain).
    fn new(purpose: Purpose, account: u32, address_index: u32) -> Self {
        Self {
            purpose,
            coin_type: 0,
            account,
            change: 0,
            address_index,
        }
    }

    /// Convenience constructor for P2PKH (legacy) addresses.
    pub fn p2pkh(account: u32, address_index: u32) -> Self {
        Self::new(Purpose::P2PKH, account, address_index)
    }

    /// Convenience constructor for P2WPKH (native SegWit) addresses.
    pub fn p2wpkh(account: u32, address_index: u32) -> Self {
        Self::new(Purpose::P2WPKH, account, address_index)
    }

    /// Convenience constructor for P2TR (Taproot) addresses.
    pub fn p2tr(account: u32, address_index: u32) -> Self {
        Self::new(Purpose::P2TR, account, address_index)
    }

    const HARDENED_OFFSET: u32 = 0x8000_0000;

    /// Returns the derivation path as a Vec<Vec<u8>> (one 4-byte big-endian element per level),
    /// suitable for use with the Internet Computer's ECDSA/Schnorr APIs.
    pub fn to_vec_u8_path(&self) -> Vec<Vec<u8>> {
        vec![
            (self.purpose.to_u32() | Self::HARDENED_OFFSET)
                .to_be_bytes()
                .to_vec(),
            (self.coin_type | Self::HARDENED_OFFSET)
                .to_be_bytes()
                .to_vec(),
            (self.account | Self::HARDENED_OFFSET)
                .to_be_bytes()
                .to_vec(),
            self.change.to_be_bytes().to_vec(),
            self.address_index.to_be_bytes().to_vec(),
        ]
    }
}

impl fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "m/{}'/{}'/{}'/{}/{}",
            self.purpose.to_u32(),
            self.coin_type,
            self.account,
            self.change,
            self.address_index
        )
    }
}

pub fn generate_derivation_path(principal: &Principal) -> Vec<Vec<u8>> {
    let mut hash = [0u8; 32];
    let mut hasher = Sha3::v256();
    hasher.update(principal.as_slice());
    hasher.finalize(&mut hash);
    vec![hash.to_vec()]
}

pub enum PrimaryOutput {
    /// Pay someone (spendable output).
    Address(Address, u64), // destination address, amount in satoshis
    /// Embed data (unspendable OP_RETURN output).
    OpReturn(ScriptBuf), // script already starts with OP_RETURN
}

pub fn select_utxos_greedy(
    own_utxos: &[Utxo],
    amount: u64,
    fee: u64,
) -> Result<Vec<&Utxo>, String> {
    // Greedily select UTXOs in reverse order (oldest last) until we cover amount + fee.
    let mut utxos_to_spend = vec![];
    let mut total_spent = 0;
    for utxo in own_utxos.iter().rev() {
        total_spent += utxo.value;
        utxos_to_spend.push(utxo);
        if total_spent >= amount + fee {
            break;
        }
    }

    // Abort if we can't cover the payment + fee.
    if total_spent < amount + fee {
        return Err(format!(
            "Insufficient balance: {}, trying to transfer {} satoshi with fee {}",
            total_spent, amount, fee
        ));
    }

    Ok(utxos_to_spend)
}

pub fn select_one_utxo(own_utxos: &[Utxo], amount: u64, fee: u64) -> Result<Vec<&Utxo>, String> {
    for utxo in own_utxos.iter().rev() {
        if utxo.value >= amount + fee {
            return Ok(vec![&utxo]);
        }
    }

    Err(format!(
        "No sufficiently large utxo found: amount {} satoshi, fee {}",
        amount, fee
    ))
}

pub fn build_transaction_with_fee(
    utxos_to_spend: Vec<&Utxo>,
    own_address: &Address,
    primary_output: &PrimaryOutput,
    fee: u64,
) -> Result<(Transaction, Vec<TxOut>), String> {
    // Define a dust threshold below which change outputs are discarded.
    // This prevents creating outputs that cost more to spend than they're worth.
    const DUST_THRESHOLD: u64 = 1_000;

    // --- Build Inputs ---
    // Convert UTXOs into transaction inputs, preparing them for signing.
    let inputs: Vec<TxIn> = utxos_to_spend
        .iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint {
                txid: Txid::from_raw_hash(Hash::from_slice(&utxo.outpoint.txid).unwrap()),
                vout: utxo.outpoint.vout,
            },
            sequence: Sequence::MAX,      // No relative timelock constraints
            witness: Witness::new(),      // Will be filled in during signing
            script_sig: ScriptBuf::new(), // Empty for SegWit and Taproot (uses witness)
        })
        .collect();

    // --- Create Previous Outputs ---
    // Each TxOut represents an output from previous transactions being spent.
    // This data is required for signing P2WPKH and P2TR transactions.
    let prevouts = utxos_to_spend
        .clone()
        .into_iter()
        .map(|utxo| TxOut {
            value: Amount::from_sat(utxo.value),
            script_pubkey: own_address.script_pubkey(),
        })
        .collect();

    // --- Build Outputs ---
    // Create the primary output based on the operation type.
    let mut outputs = Vec::<TxOut>::new();

    match primary_output {
        PrimaryOutput::Address(addr, amt) => outputs.push(TxOut {
            script_pubkey: addr.script_pubkey(),
            value: Amount::from_sat(*amt),
        }),
        PrimaryOutput::OpReturn(script) => outputs.push(TxOut {
            script_pubkey: script.clone(),
            value: Amount::from_sat(0), // OP_RETURN outputs carry no bitcoin value
        }),
    }

    // Calculate change and add change output if above dust threshold.
    // This prevents value loss while avoiding uneconomical outputs.
    let total_in: u64 = utxos_to_spend.iter().map(|u| u.value).sum();
    let change = total_in
        .checked_sub(outputs.iter().map(|o| o.value.to_sat()).sum::<u64>() + fee)
        .ok_or("fee exceeds inputs")?;

    if change >= DUST_THRESHOLD {
        outputs.push(TxOut {
            script_pubkey: own_address.script_pubkey(),
            value: Amount::from_sat(change),
        });
    }

    // --- Assemble Transaction ---
    // Create the final unsigned transaction with version 2 for modern features.
    Ok((
        Transaction {
            input: inputs,
            output: outputs,
            lock_time: LockTime::ZERO, // No absolute timelock
            version: Version::TWO,     // Standard for modern Bitcoin transactions
        },
        prevouts,
    ))
}
pub async fn get_fee_per_byte(ctx: &BitcoinContext) -> u64 {
    // Query recent fee percentiles from the Bitcoin network.
    // This gives us real-time fee data based on recent transaction activity.
    let fee_percentiles = bitcoin_get_current_fee_percentiles(&GetCurrentFeePercentilesRequest {
        network: ctx.network,
    })
    .await
    .unwrap();

    if fee_percentiles.is_empty() {
        // Empty percentiles indicate that we're likely on regtest with no standard transactions.
        // Use a reasonable fallback that works for development and testing.
        2000 // 2 sat/vB in millisatoshis
    } else {
        // Use the 50th percentile (median) for balanced confirmation time and cost.
        // This avoids both overpaying (high percentiles) and slow confirmation (low percentiles).
        fee_percentiles[50]
    }
}
