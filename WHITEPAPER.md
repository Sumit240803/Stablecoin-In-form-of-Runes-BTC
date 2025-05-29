# USDB Token Specification — Detailed Version

## 🔹 Token Overview

- **Name:** USDB (United States Dollar Bitcoin-backed)  
- **Ticker:** USDB  
- **Category:** BTC-Collateralized Stablecoin  
- **Format:** Programmable rune-style fungible tokens (REE-compatible)  
- **Network:** Internet Computer Protocol (ICP) using REE and Chain Key Bitcoin  

---

## 🔹 Collateral Model

- **Backing Asset:** BTC (held via Chain Key Bitcoin)  
- **Collateral Ratio:** 100%+ overcollateralized (configurable)  
- **Custody:** Programmatic via smart contracts using CK-BTC  

### Collateral Locking

- A minimum of **1 USD worth of BTC** must be locked to mint 1 USDB  
- BTC amount calculated via **latest oracle price** (e.g., BTC/USD from HTTPS outcalls)  

---

## 🔹 Issuance Logic

### Minting

1. Users send BTC to a controlled CK-BTC address  
2. REE-compatible canister fetches BTC/USD via HTTPS outcall  
3. Calculates USDB amount and mints tokens to user  

### Burning

1. User sends USDB tokens to `burn()` function  
2. System releases corresponding BTC to user’s BTC address  

### Fees (Optional)

- **Minting Fee:** X%  
- **Burning Fee:** Y%  

---

## 🔹 Token Properties & Metadata

Each rune-style token will have metadata to trace collateral and state:

```json
{
  "rune_id": "usdb-000001a2b4",
  "collateral_amount_btc": "0.000021",
  "btc_usd_price_at_mint": "47123.50",
  "mint_timestamp": "2025-05-28T12:45:03Z",
  "collateral_tx_hash": "abc123...",
  "status": "active" // options: active, burned
}
