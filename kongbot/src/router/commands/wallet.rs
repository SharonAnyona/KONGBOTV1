use candid::{Principal, Nat};
use ic_cdk::api::time;
use std::cell::RefCell;
use std::collections::HashMap;
use super::common_types::{WalletBalance, TradeError};

thread_local! {
    static WALLETS: RefCell<HashMap<Principal, WalletBalance>> = RefCell::new(HashMap::new());
}

/// Initializes a wallet for a user (called automatically on first interaction)
pub fn init_wallet(user: Principal) -> WalletBalance {
    let wallet = WalletBalance {
        owner: user,
        balances: HashMap::new(),
        last_updated: time(),
    };

    WALLETS.with(|w| {
        w.borrow_mut().insert(user, wallet.clone());
    });

    wallet
}

/// Gets or creates a wallet for a user
pub fn get_wallet(user: Principal) -> WalletBalance {
    WALLETS.with(|w| {
        w.borrow()
            .get(&user)
            .cloned()
            .unwrap_or_else(|| init_wallet(user))
    })
}

/// Checks balance for a specific token
pub fn get_balance(user: Principal, token: &str) -> Nat {
    get_wallet(user)
        .balances
        .get(&token.to_lowercase())
        .cloned()
        .unwrap_or(Nat::from(0u64))
}

/// Deposits tokens into a wallet
pub fn deposit(user: Principal, token: String, amount: Nat) -> Nat {
    WALLETS.with(|w| {
        let mut wallets = w.borrow_mut();
        let wallet = wallets
            .entry(user)
            .or_insert_with(|| init_wallet(user));

        let balance = wallet
            .balances
            .entry(token.to_lowercase())
            .or_insert(Nat::from(0u64));

        *balance += amount;
        wallet.last_updated = time();

        balance.clone()
    })
}

/// Withdraws tokens from a wallet
pub fn withdraw(user: Principal, token: String, amount: Nat) -> Result<Nat, TradeError> {
    WALLETS.with(|w| {
        let mut wallets = w.borrow_mut();
        let wallet = wallets
            .entry(user)
            .or_insert_with(|| init_wallet(user));

        let balance = wallet
            .balances
            .entry(token.to_lowercase())
            .or_insert(Nat::from(0u64));

        if *balance < amount {
            return Err(TradeError::InsufficientBalance);
        }

        *balance -= amount;
        wallet.last_updated = time();

        Ok(balance.clone())
    })
}

/// Formats wallet balances for display
pub fn format_balances(user: Principal, token: Option<String>) -> String {
    match token {
        Some(t) => {
            let balance = get_balance(user, &t);
            format!("💰 {} balance: {}", t.to_uppercase(), balance)
        }
        None => {
            let wallet = get_wallet(user);
            let balances: Vec<String> = wallet
                .balances
                .iter()
                .map(|(token, amount)| format!("  {}: {}", token.to_uppercase(), amount))
                .collect();
                
            if balances.is_empty() {
                "👜 Your wallet is empty".to_string()
            } else {
                format!("💼 Your balances:\n{}", balances.join("\n"))
            }
        }
    }
}
