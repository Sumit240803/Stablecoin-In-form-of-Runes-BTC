# REE Compatible Canister for  BTC-collateralized runes 

# Architecture

# What’s BTC-collateralized Runes?

Runes are digital assets built natively on Blockchain. Due to limitations of BTC in NFTs or DeFi because of its protocol, there comes the need for runes which are designed to be efficient and simple which uses Bitcoin’s UTXO model and OP\_RETURN opcode.  
A **BTC-collateralized Rune** is a fungible or semi-fungible on-chain token that is **minted only upon verified Bitcoin deposits** and can be **redeemed or burned** when the backing BTC is withdrawn, ensuring **trustless 1:1 value representation** between the token and Bitcoin.

# Minting Runes 

To mint the runes, users first of all need to stake the BTC from its wallet. Our price-oracle-canister will fetch the real time BTC price from Binance API which ensures the valid price feed. The price data will be used to calculate the amount of runes to be issued to the user.

Our system will also verify the BTC is locked securely and its value is locked. This can be done with off-chain verification or a bridge canister that listens for BTC transactions.

Using the BTC amount and current BTC price our system will calculate the number of runes so that the ratio will be 1:1 similar to the stablecoin. The value of total runes must be equal to the BTC collateralized.

The canister then calls the mint function to issue the runes to the user. The canister keeps track of how much BTC collateral backs how many runes to maintain a **collateralization ratio**.  
Important for audits, redemptions, and ensuring no over-minting happens.

# Burning Runes 

The process of burning the runes is not very complicated but it contains some safety checks to ensure smooth burning of runes.  
The user requests the amount of runes to burn but before burning the system will check if the user has enough tokens to burn.   
The canister **burns** the specified amount of runes from the user’s balance via your token ledger.  
The canister prepares a Bitcoin transaction to send the corresponding BTC amount back to the user’s BTC address. 

Since ICP cannot directly sign Bitcoin transactions, you must:  
Use threshold ECDSA to sign BTC transactions on-chain or, emit a withdrawal event your off-chain service listens to, which signs & broadcasts BTC txns.

