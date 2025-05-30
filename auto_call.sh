#!/bin/bash

while true;do
    echo "Fetching BTC price at $(date)"
    dfx canister call price-oracle-canister update_btc_price
    sleep 60
done