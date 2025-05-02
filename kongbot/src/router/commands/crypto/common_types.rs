use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

// Use thread_local! for mutable static state on the IC
thread_local! {
    // Alerts state
    pub static ALERTS: RefCell<AlertsState> = RefCell::new(AlertsState::default());
    // Trade state (if needed, otherwise remove)
    // pub static TRADES: RefCell<TradesState> = RefCell::new(TradesState::default());
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PriceAlert {
    pub user_id: String,
    pub coin_id: String,
    pub target_price: f64,
    pub currency: String,
    pub is_above: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AlertsState {
    // HashMap<coin_id, Vec<PriceAlert>>
    pub alerts: HashMap<String, Vec<PriceAlert>>,
}

// Remove if TRADES state is not used
// #[derive(Serialize, Deserialize, Debug, Default)]
// pub struct TradesState {
//     // Define trade state structure if needed
// }

// Structures for CoinGecko API responses (keep if used by commented-out code or future implementations)
#[derive(Deserialize, Debug, Clone)]
pub struct MarketData {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: HashMap<String, f64>,
    pub market_cap_rank: Option<u32>,
    pub price_change_percentage_24h: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct PriceResponse {
    #[serde(flatten)]
    pub prices: HashMap<String, HashMap<String, f64>>,
}
