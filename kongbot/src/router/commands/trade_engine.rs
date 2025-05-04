use candid::{Principal, Nat};
use ic_cdk::api::time;
use std::cell::RefCell;
use std::collections::HashMap;
use super::common_types::{TradeRequest, TradeResponse, TradeType, TradeStatus, TradeError};
use super::wallet;
use crate::router::price;

thread_local! {
    static ACTIVE_TRADES: RefCell<HashMap<u64, TradeRequest>> = RefCell::new(HashMap::new());
    static TRADE_HISTORY: RefCell<HashMap<Principal, Vec<TradeResponse>>> = RefCell::new(HashMap::new());
    static TRADE_COUNTER: RefCell<u64> = RefCell::new(0);
}

pub async fn execute_trade(request: TradeRequest) -> Result<TradeResponse, TradeError> {
    // Validate the trade request first
    validate_trade_request(&request)?;
    
    // Check and reserve balances
    reserve_balances(request.user_id, &request).await?;
    
    // Generate a new trade ID
    let trade_id = TRADE_COUNTER.with(|c| {
        let mut counter = c.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Execute based on order type
    let (status, executed_price) = match request.trade_type {
        TradeType::MarketBuy | TradeType::MarketSell => {
            execute_market_order(request.user_id, &request).await?
        }
        TradeType::LimitBuy | TradeType::LimitSell => {
            (TradeStatus::Pending, None)
        }
    };
    
    let response = TradeResponse {
        trade_id,
        status,
        executed_price,
        timestamp: time(),
        amount: request.amount,
        limit_price: request.limit_price,
    };
    
    // Store the active trade
    ACTIVE_TRADES.with(|trades| {
        trades.borrow_mut().insert(trade_id, request.clone());
    });
    
    // Append to user trade history
    TRADE_HISTORY.with(|history| {
        history.borrow_mut()
            .entry(request.user_id)
            .or_default()
            .push(response.clone());
    });
    
    Ok(response)
}

async fn execute_market_order(
    user: Principal,
    request: &TradeRequest,
) -> Result<(TradeStatus, Option<f64>), TradeError> {
    let (base_token, quote_token) = parse_trading_pair(&request.pair)?;
    let price = get_current_price(&request.pair).await?;
    
    match request.trade_type {
        TradeType::MarketBuy => {
            let total_cost = request.amount * price;
            wallet::withdraw(
                user,
                quote_token.to_string(),
                amount_to_nat(total_cost),
            )?;
            wallet::deposit(
                user,
                base_token.to_string(),
                amount_to_nat(request.amount),
            );
        }
        TradeType::MarketSell => {
            wallet::withdraw(
                user,
                base_token.to_string(),
                amount_to_nat(request.amount),
            )?;
            wallet::deposit(
                user,
                quote_token.to_string(),
                amount_to_nat(request.amount * price),
            );
        }
        _ => unreachable!(),
    }
    
    Ok((TradeStatus::Filled, Some(price)))
}

async fn reserve_balances(
    user: Principal,
    request: &TradeRequest,
) -> Result<(), TradeError> {
    let (base_token, quote_token) = parse_trading_pair(&request.pair)?;
    
    match request.trade_type {
        TradeType::MarketBuy | TradeType::LimitBuy => {
            let required = match request.limit_price {
                Some(price) => request.amount * price,
                None => request.amount * get_current_price(&request.pair).await?,
            };
            check_balance(user, &quote_token, required).await?;
        }
        TradeType::MarketSell | TradeType::LimitSell => {
            check_balance(user, &base_token, request.amount).await?;
        }
    }
    
    Ok(())
}

async fn check_balance(
    user: Principal,
    token: &str,
    amount: f64,
) -> Result<(), TradeError> {
    let balance_nat = wallet::get_balance(user, token);
    if balance_nat < amount_to_nat(amount) {
        Err(TradeError::InsufficientBalance)
    } else {
        Ok(())
    }
}

async fn get_current_price(pair: &str) -> Result<f64, TradeError> {
    let (base, _) = parse_trading_pair(pair)?;
    price::fetch_price(base)
        .await
        .map_err(|e| TradeError::Other(format!("Failed to fetch price: {}", e)))
}

fn parse_trading_pair(pair: &str) -> Result<(&str, &str), TradeError> {
    pair
        .split_once('/')
        .ok_or(TradeError::InvalidTradePair)
}

fn amount_to_nat(amount: f64) -> Nat {
    Nat::from((amount * 100.0) as u64)
}

fn validate_trade_request(request: &TradeRequest) -> Result<(), TradeError> {
    // Validate the trading pair
    parse_trading_pair(&request.pair)?;
    
    // Validate amount
    if request.amount <= 0.0 {
        return Err(TradeError::InvalidTradeAmount);
    }
    
    // Validate limit orders require limit price
    if matches!(request.trade_type, TradeType::LimitBuy | TradeType::LimitSell) {
        let price = request.limit_price.ok_or(TradeError::InvalidOrderType)?;
        if price <= 0.0 {
            return Err(TradeError::InvalidTradePrice);
        }
    }
    
    Ok(())
}

// Process pending limit orders
pub async fn check_pending_orders() {
    let trades_to_process: Vec<(u64, TradeRequest)> = ACTIVE_TRADES.with(|trades| {
        trades
            .borrow()
            .iter()
            .filter(|(_, t)| matches!(t.trade_type, TradeType::LimitBuy | TradeType::LimitSell))
            .map(|(id, t)| (*id, t.clone()))
            .collect()
    });

    for (id, trade) in trades_to_process {
        if let Ok(current_price) = get_current_price(&trade.pair).await {
            let execute = match trade.trade_type {
                TradeType::LimitBuy => current_price <= trade.limit_price.unwrap(),
                TradeType::LimitSell => current_price >= trade.limit_price.unwrap(),
                _ => false,
            };
            if execute {
                if let Ok((status, price)) = execute_market_order(trade.user_id, &trade).await {
                    // Update history
                    TRADE_HISTORY.with(|history| {
                        if let Some(entries) = history.borrow_mut().get_mut(&trade.user_id) {
                            if let Some(e) = entries.iter_mut().find(|e| e.trade_id == id) {
                                e.status = status;
                                e.executed_price = price;
                                e.limit_price = trade.limit_price;
                            }
                        }
                    });
                    // Remove from active
                    ACTIVE_TRADES.with(|trades| trades.borrow_mut().remove(&id));
                }
            }
        }
    }
}

// Retrieve user trade history
pub fn get_trade_history(user: Principal) -> Vec<TradeResponse> {
    TRADE_HISTORY.with(|history| history.borrow().get(&user).cloned().unwrap_or_default())
}

// Retrieve user's active trades
pub fn get_active_trades(user: Principal) -> Vec<(u64, TradeRequest)> {
    ACTIVE_TRADES.with(|trades| {
        trades
            .borrow()
            .iter()
            .filter(|(_, t)| t.user_id == user)
            .map(|(id, t)| (*id, t.clone()))
            .collect()
    })
}

// Cancel a trade
pub fn cancel_trade(user: Principal, trade_id: u64) -> Result<TradeResponse, TradeError> {
    let mut result: Result<TradeResponse, TradeError> = Err(TradeError::Other("Trade not found".to_string()));
    
    ACTIVE_TRADES.with(|trades| {
        let mut map = trades.borrow_mut();
        if let Some(req) = map.get(&trade_id) {
            if req.user_id != user {
                result = Err(TradeError::Other("Not authorized to cancel this trade".to_string()));
            } else {
                // Build cancellation response
                let response = TradeResponse {
                    trade_id,
                    status: TradeStatus::Cancelled,
                    executed_price: None,
                    timestamp: time(),
                    amount: req.amount,
                    limit_price: req.limit_price,
                };
                // Append to history
                TRADE_HISTORY.with(|hist| {
                    hist.borrow_mut().entry(user).or_default().push(response.clone());
                });
                // Remove active trade
                map.remove(&trade_id);
                result = Ok(response);
            }
        }
    });
    
    result
}
