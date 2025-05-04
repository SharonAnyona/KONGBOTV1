use async_trait::async_trait;
use candid::Principal;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::oc_api::actions::send_message;
use oc_bots_sdk::oc_api::client::Client;
use oc_bots_sdk::types::BotCommandContext;
use oc_bots_sdk_canister::CanisterRuntime;
use std::sync::LazyLock;

use crate::router::commands::common_types::{TradeStatus, TradeType};
use crate::router::commands::trade_engine::{get_trade_history, get_active_trades, cancel_trade};

static TRADES_DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(Trades::definition);

pub struct Trades;

#[async_trait]
impl CommandHandler<CanisterRuntime> for Trades {
    fn definition(&self) -> &BotCommandDefinition {
        &TRADES_DEFINITION
    }

    async fn execute(
        &self,
        oc_client: Client<CanisterRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        let context = oc_client.context();
        // Convert user ID to Principal
        let user_principal = Principal::from_text(context.bot_id.to_string())
            .map_err(|_| "Failed to convert user ID to Principal".to_string())?;

        // Handle optional cancel parameter
        let cancel_arg: String = context.command.arg("cancel");
        if !cancel_arg.is_empty() {
            let trade_id = cancel_arg
                .parse::<u64>()
                .map_err(|_| "Invalid trade ID. Please enter a number.".to_string())?;

            match cancel_trade(user_principal, trade_id) {
                Ok(_) => {
                    let msg = format!("✅ Trade #{} has been cancelled.", trade_id);
                    let message = oc_client
                        .send_text_message(msg)
                        .execute_then_return_message(|args, resp| {
                            if let Ok(send_message::Response::Success(_)) = resp {
                                // success
                            } else {
                                ic_cdk::println!("send_text_message error: {:?}, {:?}", args, resp);
                            }
                        });
                    return Ok(SuccessResult { message });
                }
                Err(e) => return Err(format!("Failed to cancel trade: {}", e)),
            }
        }

        // Fetch active trades and history
        let active_trades = get_active_trades(user_principal);
        let trade_history = get_trade_history(user_principal);

        // Build response text
        let mut text = String::new();

        // Active Trades
        if !active_trades.is_empty() {
            text.push_str("🔄 **Active Trades**\n\n");
            for (id, trade) in active_trades {
                let ttype = match trade.trade_type {
                    TradeType::MarketBuy => "Market Buy",
                    TradeType::MarketSell => "Market Sell",
                    TradeType::LimitBuy => "Limit Buy",
                    TradeType::LimitSell => "Limit Sell",
                };
                text.push_str(&format!(
                    "**ID:** {} - **{}**\n**Pair:** {}\n**Amount:** {:.4}\n",
                    id, ttype, trade.pair, trade.amount
                ));
                if let Some(lp) = trade.limit_price {
                    text.push_str(&format!("**Limit Price:** ${:.2}\n", lp));
                }
                if let Some(sl) = trade.stop_loss {
                    text.push_str(&format!("**Stop Loss:** ${:.2}\n", sl));
                }
                if let Some(tp) = trade.take_profit {
                    text.push_str(&format!("**Take Profit:** ${:.2}\n", tp));
                }
                text.push_str("\n");
            }
            text.push_str("To cancel a trade, use `/trades cancel:<trade_id>`\n\n");
        } else {
            text.push_str("🔄 **No Active Trades**\n\n");
        }

        // Trade History
        if !trade_history.is_empty() {
            text.push_str("📜 **Trade History**\n\n");
            for tr in trade_history.iter().rev().take(5) {
                let status = match tr.status {
                    TradeStatus::Filled => "✅ Filled",
                    TradeStatus::Cancelled => "❌ Cancelled",
                    TradeStatus::Rejected => "⛔ Rejected",
                    TradeStatus::Pending => "🔄 Pending",
                };
                text.push_str(&format!(
                    "**ID:** {} - **Status:** {}\n**Amount:** {:.4}\n",
                    tr.trade_id, status, tr.amount
                ));
                if let Some(price) = tr.executed_price {
                    text.push_str(&format!("**Price:** ${:.2}\n", price));
                }
                text.push_str("\n");
            }
        } else {
            text.push_str("📜 **No Trade History**\n\n");
        }

        text.push_str("Use `/settrade` to create a new trade.");

        // Send message
        let message = oc_client
            .send_text_message(text)
            .with_block_level_markdown(true)
            .execute_then_return_message(|args, resp| {
                if let Ok(send_message::Response::Success(_)) = resp {
                    // success
                } else {
                    ic_cdk::println!("send_text_message error: {:?}, {:?}", args, resp);
                }
            });

        Ok(SuccessResult { message })
    }
}

impl Trades {
    fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "trades".to_string(),
            description: Some("View your active trades and trade history".to_string()),
            placeholder: None,
            params: vec![BotCommandParam {
                name: "cancel".to_string(),
                description: Some("Cancel a trade by ID".to_string()),
                param_type: BotCommandParamType::StringParam(StringParam {
                    min_length: 1,
                    max_length: 20,
                    choices: vec![],
                    multi_line: false,
                }),
                required: false,
                placeholder: Some("Trade ID".to_string()),
            }],
            permissions: BotPermissions::text_only(),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}

