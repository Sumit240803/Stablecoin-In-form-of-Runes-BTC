use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpMethod, HttpHeader,
};
use ic_cdk::{update,query};
use std::cell::RefCell;

thread_local!{
    static BTC_PRICE:RefCell<String> = RefCell::new("0.0".to_string());
}
#[update]
async fn update_btc_price() {
    let url = "https://api.binance.com/api/v3/ticker/price?symbol=BTCUSDT";

    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        max_response_bytes: Some(1024),
        method: HttpMethod::GET,
        headers: vec![HttpHeader {
            name: "X-MBX-APIKEY".to_string(),
            value: "your-api-key-here".to_string(),
        }],
        body: None,
        transform: None,
    };

    let cycles = 5_000_000_000; // 5B cycles (adjust as needed)
    let (response,) = http_request(request, cycles).await.unwrap();
    let price = String::from_utf8(response.body).unwrap();
    BTC_PRICE.with(|p| p.replace(price));
}


#[query]
fn get_btc_price()->String{
    BTC_PRICE.with(|p| p.borrow().clone())
}