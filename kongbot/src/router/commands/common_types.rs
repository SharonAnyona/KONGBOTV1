use candid::{CandidType, Deserialize, Nat, Principal};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

// ======================
// Trading Types
// ======================
#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum TradeType {
    MarketBuy,
    MarketSell,
    LimitBuy,
    LimitSell,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum TradeStatus {
    Pending,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct TradeRequest {
    pub pair: String,               // e.g., "ICP/USD"
    pub trade_type: TradeType,
    pub amount: f64,
    pub limit_price: Option<f64>,   // None for market orders
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub expiry: Option<u64>,        // Unix timestamp
    pub user_id: Principal,         // User's principal ID
    pub chat_id: String,            // Chat where the trade was requested
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct TradeResponse {
    pub trade_id: u64,
    pub status: TradeStatus,
    pub executed_price: Option<f64>,
    pub timestamp: u64,
    pub amount: f64,
    pub limit_price: Option<f64>,
}

// ======================
// Error Handling
// ======================
#[derive(CandidType, Serialize, Deserialize, Debug)]
pub enum TradeError {
    // Trading Errors
    InvalidTradePair,
    InvalidTradeAmount,
    InvalidTradePrice,
    InvalidOrderType,
    ExecutionFailed,
    PriceDataUnavailable,
    InsufficientBalance,
    // Other errors
    Other(String),
}

impl fmt::Display for TradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeError::InvalidTradePair => write!(f, "Invalid trading pair"),
            TradeError::InvalidTradeAmount => write!(f, "Invalid trade amount"),
            TradeError::InvalidTradePrice => write!(f, "Invalid trade price"),
            TradeError::InvalidOrderType => write!(f, "Invalid order type"),
            TradeError::ExecutionFailed => write!(f, "Trade execution failed"),
            TradeError::PriceDataUnavailable => write!(f, "Price data unavailable"),
            TradeError::InsufficientBalance => write!(f, "Insufficient balance"),
            TradeError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

// ======================
// Wallet Types
// ======================
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct WalletBalance {
    pub owner: Principal,
    pub balances: HashMap<String, Nat>,
    pub last_updated: u64,
}
