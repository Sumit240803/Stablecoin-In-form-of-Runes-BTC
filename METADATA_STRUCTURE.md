# Metadata Structure and Storage format for each Rune

## RuneMetadata structure

rune\_id: String  
*Unique identifier for the rune token (e.g., UUID or hash).*

*owner: Principal*  
*The owner’s principal ID (ICP identity).*

*amount: u64*  
 *The amount of runes (token quantity).*

*collateral\_btc: f64*  
 *The amount of BTC collateral backing this rune.*

*minted\_at: u64*  
 *Timestamp (Unix epoch seconds) when the rune was minted.*

*last\_updated: u64*  
 *Timestamp of the last metadata update (e.g., burn or transfer).*

*price\_per\_rune\_usd: f64*  
 *Price of one rune in USD at minting time.*

*btc\_price\_usd: f64*  
 *BTC price in USD at the time of minting (for reference).*

*status: RuneStatus*  
 *Current state of the rune (Active, Burned, Pending).*

***RuneStatus enum values:***

*Active*

*Burned*

*Pending*

**Storage format in the canister:**

* Use a map/dictionary with `rune_id` as key and `RuneMetadata` as value.  
   (e.g., `HashMap<String, RuneMetadata>` or stable structures like `BTreeMap`).

* Optionally, maintain an index mapping from `owner` to a list of `rune_id`s they own, for efficient querying.  
   (e.g., `HashMap<Principal, Vec<String>>`)

We can also use rune indexer provided by Omnity Network