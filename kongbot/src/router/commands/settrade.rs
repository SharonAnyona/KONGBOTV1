use async_trait::async_trait;
use candid::Principal;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::oc_api::actions::send_message;
use oc_bots_sdk::oc_api::client::Client;
use oc_bots_sdk::types::{BotCommandContext, BotCommandScope};
use oc_bots_sdk_canister::CanisterRuntime;
use std::sync::LazyLock;

use crate::router::commands::common_types::{TradeRequest, TradeResponse, TradeType, TradeStatus, TradeError};
use crate::router::commands::trade_engine::execute_trade;

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(SetTrade::definition);

pub struct SetTrade;

// Helper to parse optional f64 args
fn opt_arg_f64(ctx: &BotCommandContext, name: &str) -> Result<Option<f64>, String> {
    let raw: String = ctx.command.arg(name);
    if raw.is_empty() {
        Ok(None)
    } else {
        raw.parse::<f64>()
            .map(Some)
            .map_err(|_| format!("Invalid {}. Please enter a valid number.", name))
    }
}

#[async_trait]
impl CommandHandler<CanisterRuntime> for SetTrade {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        oc_client: Client<CanisterRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        let context = oc_client.context();

        // Convert bot_id to Principal
        let user_principal = Principal::from_text(context.bot_id.to_string())
            .map_err(|_| "Failed to convert user ID to Principal".to_string())?;

        // Required parameters
        let action = context.command.arg::<String>("action").to_lowercase();
        let pair = context.command.arg::<String>("pair").to_uppercase();
        let amount = context.command.arg::<String>("amount")
            .parse::<f64>()
            .map_err(|_| "Invalid amount. Please enter a valid number.".to_string())?;

        // Optional numeric parameters
        let limit_price = opt_arg_f64(&context, "limit_price")?;
        let stop_loss   = opt_arg_f64(&context, "stop_loss")?;
        let take_profit = opt_arg_f64(&context, "take_profit")?;

        // Determine trade type
        let trade_type = match (action.as_str(), limit_price.is_some()) {
            ("buy", false) => TradeType::MarketBuy,
            ("buy", true)  => TradeType::LimitBuy,
            ("sell", false) => TradeType::MarketSell,
            ("sell", true)  => TradeType::LimitSell,
            _ => return Err("Invalid action. Use 'buy' or 'sell'.".to_string()),
        };

        // Extract chat ID
        let chat_id = if let BotCommandScope::Chat(details) = &context.scope {
            format!("{:?}", details.chat)
        } else {
            return Err("Trades can only be set in chats, not communities".to_string());
        };

        // Build request
        let request = TradeRequest {
            pair,
            trade_type,
            amount,
            limit_price,
            stop_loss,
            take_profit,
            expiry: None,
            user_id: user_principal,
            chat_id,
        };

        // Execute trade
        match execute_trade(request).await {
            Ok(response) => {
                let message_text = format_trade_response(&response);
                let message = oc_client
                    .send_text_message(message_text)
                    .with_block_level_markdown(true)
                    .execute_then_return_message(|args, resp| {
                        if let Ok(send_message::Response::Success(_)) = resp {
                            // sent
                        } else {
                            ic_cdk::println!("send_text_message error: {:?}, {:?}", args, resp);
                        }
                    });
                Ok(SuccessResult { message })
            }
            Err(err) => Err(format_trade_error(err)),
        }
    }
}

fn format_trade_response(response: &TradeResponse) -> String {
    match response.status {
        TradeStatus::Filled => format!(
            "✅ **Trade Executed!**\n\n**ID:** {}\n**Price:** ${:.2}\n**Amount:** {:.4}\n**Total:** ${:.2}",
            response.trade_id,
            response.executed_price.unwrap_or(0.0),
            response.amount,
            response.executed_price.unwrap_or(0.0) * response.amount
        ),
        TradeStatus::Pending => {
            // Use limit_price from response if available
            let target = response.executed_price.or(response.limit_price).unwrap_or(0.0);
            format!(
                "🔄 **Limit Order Placed!**\n\n**ID:** {}\n**Amount:** {:.4}\n**Target Price:** ${:.2}\n\nWaiting for price target...",
                response.trade_id,
                response.amount,
                target
            )
        }
        TradeStatus::Cancelled => format!(
            "❌ **Order Cancelled!**\n\n**ID:** {}",
            response.trade_id
        ),
        TradeStatus::Rejected => format!(
            "⛔ **Order Rejected!**\n\n**ID:** {}",
            response.trade_id
        ),
    }
}

fn format_trade_error(error: TradeError) -> String {
    match error {
        TradeError::InsufficientBalance => "❌ Insufficient balance".to_string(),
        TradeError::InvalidTradePair    => "❌ Invalid trading pair".to_string(),
        TradeError::InvalidTradeAmount  => "❌ Invalid trade amount".to_string(),
        TradeError::InvalidTradePrice   => "❌ Invalid price".to_string(),
        TradeError::InvalidOrderType    => "❌ Invalid order type".to_string(),
        TradeError::PriceDataUnavailable=> "❌ Price data unavailable".to_string(),
        TradeError::ExecutionFailed     => "❌ Trade execution failed".to_string(),
        TradeError::Other(msg)          => format!("❌ Error: {}", msg),
    }
}

impl SetTrade {
    fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "settrade".to_string(),
            description: Some("Set up a cryptocurrency trade".to_string()),
            placeholder: None,
            params: vec![
                BotCommandParam {
                    name: "action".to_string(),
                    description: Some("Buy or sell".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 4,
                        choices: vec![
                            BotCommandOptionChoice { name: "buy".to_string(), value: "buy".to_string() },
                            BotCommandOptionChoice { name: "sell".to_string(), value: "sell".to_string() },
                        ],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("buy".to_string()),
                },
                BotCommandParam {
                    name: "pair".to_string(),
                    description: Some("Trading pair (e.g., BTC/USD)".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 3,
                        max_length: 10,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("BTC/USD".to_string()),
                },
                BotCommandParam {
                    name: "amount".to_string(),
                    description: Some("Amount to trade".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 20,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("0.1".to_string()),
                },
                BotCommandParam {
                    name: "limit_price".to_string(),
                    description: Some("Limit price (leave empty for market orders)".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 20,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: false,
                    placeholder: Some("20000".to_string()),
                },
                BotCommandParam {
                    name: "stop_loss".to_string(),
                    description: Some("Stop loss price".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 20,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: false,
                    placeholder: Some("19000".to_string()),
                },
                BotCommandParam {
                    name: "take_profit".to_string(),
                    description: Some("Take profit price".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 20,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: false,
                    placeholder: Some("21000".to_string()),
                },
            ],
            permissions: BotPermissions::text_only(),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}

// Wallet command definition
static WALLET_DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(Wallet::definition);

pub struct Wallet;

#[async_trait]
impl CommandHandler<CanisterRuntime> for Wallet {
    fn definition(&self) -> &BotCommandDefinition {
        &WALLET_DEFINITION
    }

    async fn execute(
        &self,
        oc_client: Client<CanisterRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        let context = oc_client.context();
        let user_principal = Principal::from_text(context.bot_id.to_string())
            .map_err(|_| "Failed to convert user ID to Principal".to_string())?;

        // Handle optional token parameter
        let token_arg: String = context.command.arg("token");
        let token = if token_arg.is_empty() { None } else { Some(token_arg) };

        let message_text = crate::router::commands::wallet::format_balances(user_principal, token);

        let message = oc_client
            .send_text_message(message_text)
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

impl Wallet {
    fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "wallet".to_string(),
            description: Some("View your wallet balances".to_string()),
            placeholder: None,
            params: vec![BotCommandParam {
                name: "token".to_string(),
                description: Some("Specific token to check (optional)".to_string()),
                param_type: BotCommandParamType::StringParam(StringParam {
                    min_length: 1,
                    max_length: 10,
                    choices: vec![],
                    multi_line: false,
                }),
                required: false,
                placeholder: Some("BTC".to_string()),
            }],
            permissions: BotPermissions::text_only(),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}
