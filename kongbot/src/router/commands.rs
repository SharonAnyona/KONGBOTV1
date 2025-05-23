use crate::state;
use echo::Echo;
use coin_price::CoinPrice;
use alert::Alert;
use settrade::{SetTrade, Wallet};
use bal_trades::Trades;
use oc_bots_sdk::api::command::CommandHandlerRegistry;
use oc_bots_sdk::api::definition::BotCommandDefinition;
use oc_bots_sdk_canister::env::now;
use oc_bots_sdk_canister::http_command_handler;
use oc_bots_sdk_canister::CanisterRuntime;
use oc_bots_sdk_canister::OPENCHAT_CLIENT_FACTORY;
use oc_bots_sdk_canister::{HttpRequest, HttpResponse};
use std::sync::LazyLock;

mod echo;
pub mod bal_trades;
mod coin_price;
pub mod alert; 
pub mod settrade;
pub mod trade_engine;
pub mod common_types;
pub mod wallet;



static COMMANDS: LazyLock<CommandHandlerRegistry<CanisterRuntime>> =
    LazyLock::new(|| CommandHandlerRegistry::new(OPENCHAT_CLIENT_FACTORY.clone())
        .register(Echo)
        .register(CoinPrice)
        .register(Alert)
        .register(SetTrade)
        .register(Trades)
        .register(Wallet));

pub fn definitions() -> Vec<BotCommandDefinition> {
    COMMANDS.definitions()
}

pub async fn execute(request: HttpRequest) -> HttpResponse {
    let public_key = state::read(|state| state.oc_public_key().to_string());
    let now = now();
    http_command_handler::execute(request, &COMMANDS, &public_key, now).await
}

// Initialize alert monitoring when the canister starts
pub fn init_alert_monitoring() {
    alert::init_alert_monitoring();
}
