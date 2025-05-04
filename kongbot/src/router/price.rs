// src/router/logic/price.rs
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, http_request
};

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct CoinPrice {
    usd: f64,
}

// Calculate required cycles based on request and max response size
// This follows IC's pricing model for HTTP outcalls
fn calculate_cycles(request_size: usize, max_response_bytes: usize) -> u128 {
    // Base cost for any HTTP request
    let base_cost: u128 = 400_000_000;
    
    // Cost per byte of request data
    let request_cost_per_byte: u128 = 100_000;
    
    // Cost per byte of response data (reserved)
    let response_cost_per_byte: u128 = 800_000;
    
    // Calculate total cost
    let request_cycles = request_size as u128 * request_cost_per_byte;
    let response_cycles = max_response_bytes as u128 * response_cost_per_byte;
    
    // Add a 20% buffer for safety
    let total = (base_cost + request_cycles + response_cycles) * 120 / 100;
    
    total
}

pub async fn fetch_price(coin: &str) -> Result<f64, String> {
    let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", coin);
    
    // Set appropriate headers for the request
    let headers = vec![
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "OpenChat-Bot/1.0".to_string(),
        },
        HttpHeader {
            name: "Accept".to_string(),
            value: "application/json".to_string(),
        }
    ];
    
    // Define max response size - 100KB should be more than enough for price data
    let max_response_bytes: usize = 100_000;
    
    // Create the HTTP request argument
    let request = CanisterHttpRequestArgument {
        url: url.clone(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(max_response_bytes as u64),
        transform: None,
        headers,
    };
    
    // Calculate required cycles based on request size and max response size
    // URL length + headers size (approximate)
    let request_size = url.len() + 200;
    let cycles = calculate_cycles(request_size, max_response_bytes);
    
    ic_cdk::println!("Making HTTP request to {} with {} cycles", url, cycles);
    
    // Make the HTTP request with calculated cycles - following the example approach
    let response = http_request(request, cycles)
        .await
        .map_err(|(code, message)| format!("HTTP request failed: code {:?}, message: {}", code, message))?;
    
    // Unpack the response tuple
    let response = response.0;
    
    // Parse the response body without explicitly checking status code
    let response_body = String::from_utf8(response.body)
        .map_err(|e| format!("Failed to decode response body: {}", e))?;
    
    // Parse JSON response
    let json: HashMap<String, CoinPrice> = serde_json::from_str(&response_body)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Extract the price for the requested coin
    json.get(coin)
        .map(|price| {
            ic_cdk::println!("Successfully fetched price for {}: ${}", coin, price.usd);
            price.usd
        })
        .ok_or_else(|| {
            let error_msg = format!("Price not found for coin: {}", coin);
            ic_cdk::println!("{}", error_msg);
            error_msg
        })
}
