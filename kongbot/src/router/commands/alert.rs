use async_trait::async_trait;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::oc_api::actions::send_message;
use oc_bots_sdk::oc_api::client::Client;
use oc_bots_sdk::types::{BotCommandContext, BotCommandScope, Chat};
use oc_bots_sdk_canister::CanisterRuntime;
use std::sync::LazyLock;

use crate::router::price;
use crate::state;

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(Alert::definition);

pub struct Alert;

// Define the alert threshold type
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ThresholdType {
    Above,
    Below,
}

// Define the alert data structure
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PriceAlert {
    pub chat_id: Chat,
    pub coin: String,
    pub threshold: f64,
    pub threshold_type: ThresholdType,
    pub triggered: bool,
}

// Initialize the alert monitoring system
pub fn init_alert_monitoring() {
    ic_cdk::println!("Alert monitoring system initialized");
}

// This function should be called periodically by a heartbeat mechanism
pub async fn check_alerts() {
    ic_cdk::println!("Checking price alerts...");
    
    // Get all active alerts from state
    let alerts = state::read(|state| state.get_price_alerts());
    
    for alert in alerts {
        if alert.triggered {
            continue; // Skip already triggered alerts
        }
        
        // Fetch current price
        match price::fetch_price(&alert.coin).await {
            Ok(current_price) => {
                let threshold_crossed = match alert.threshold_type {
                    ThresholdType::Above => current_price >= alert.threshold,
                    ThresholdType::Below => current_price <= alert.threshold,
                };
                
                if threshold_crossed {
                    // Mark alert as triggered
                    state::mutate(|state| {
                        state.mark_alert_triggered(alert.chat_id, &alert.coin);
                    });
                    
                    // Log the notification
                    let direction = match alert.threshold_type {
                        ThresholdType::Above => "above",
                        ThresholdType::Below => "below",
                    };
                    
                    ic_cdk::println!(
                        "PRICE ALERT: {} price is now ${:.2}, which is {} the threshold of ${:.2} for chat {:?}",
                        alert.coin, current_price, direction, alert.threshold, alert.chat_id
                    );
                    
                    // In a production implementation, you would send a notification to the chat
                    // This would require integration with OpenChat's messaging system
                }
            },
            Err(e) => {
                ic_cdk::println!("Failed to fetch price for {}: {}", alert.coin, e);
            }
        }
    }
}

#[async_trait]
impl CommandHandler<CanisterRuntime> for Alert {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        oc_client: Client<CanisterRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        // Get command arguments
        let coin = oc_client.context().command.arg::<String>("coin").to_string();
        let threshold = oc_client.context().command.arg::<String>("threshold")
            .parse::<f64>()
            .map_err(|_| "Invalid threshold value. Please enter a valid number.".to_string())?;
        let direction = oc_client.context().command.arg::<String>("direction").to_string();
        
        // Validate direction
        let threshold_type = match direction.to_lowercase().as_str() {
            "above" => ThresholdType::Above,
            "below" => ThresholdType::Below,
            _ => return Err("Invalid direction. Please use 'above' or 'below'.".to_string()),
        };
        
        // Fetch current price to validate the coin
        let current_price = price::fetch_price(&coin).await
            .map_err(|e| format!("Failed to fetch price for {}: {}. Please check if the coin exists.", coin, e))?;
        
        // Extract chat information from the command context
        let context = oc_client.context();
        
        // Extract the chat ID from the scope
        let chat_id = match &context.scope {
            BotCommandScope::Chat(details) => details.chat.clone(),
            BotCommandScope::Community(_) => {
                return Err("Alerts can only be set in chats, not communities".to_string());
            }
        };
        
        // Create the alert
        let alert = PriceAlert {
            chat_id,
            coin: coin.clone(),
            threshold,
            threshold_type: threshold_type.clone(),
            triggered: false,
        };
        
        // Save the alert to state
        state::mutate(|state| {
            state.add_price_alert(alert);
        });
        
        // Check alerts immediately after setting one
        check_alerts().await;
        
        // Prepare response message
        let direction_str = match threshold_type {
            ThresholdType::Above => "above",
            ThresholdType::Below => "below",
        };
        
        let response_text = format!(
            "✅ Alert set for {} when price goes {} ${:.2}\n\nCurrent price: ${:.2}",
            coin, direction_str, threshold, current_price
        );

        // Send confirmation message
        let message = oc_client
            .send_text_message(response_text)
            .with_block_level_markdown(true)
            .execute_then_return_message(|args, response| match response {
                Ok(send_message::Response::Success(_)) => {}
                error => {
                    ic_cdk::println!("send_text_message: {args:?}, {error:?}");
                }
            });

        Ok(SuccessResult { message })
    }
}

impl Alert {
    fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "alert".to_string(),
            description: Some("Set a price alert for a cryptocurrency".to_string()),
            placeholder: None,
            params: vec![
                BotCommandParam {
                    name: "coin".to_string(),
                    description: Some("The cryptocurrency to monitor (e.g., bitcoin, ethereum)".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 50,
                        choices: Vec::new(),
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("bitcoin".to_string()),
                },
                BotCommandParam {
                    name: "threshold".to_string(),
                    description: Some("The price threshold to trigger the alert".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 20,
                        choices: Vec::new(),
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("20000".to_string()),
                },
                BotCommandParam {
                    name: "direction".to_string(),
                    description: Some("Whether to alert when price goes above or below the threshold".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 5,
                        choices: vec![
                            BotCommandOptionChoice {
                                name: "above".to_string(),
                                value: "above".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "below".to_string(),
                                value: "below".to_string(),
                            },
                        ],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("above".to_string()),
                },
            ],
            permissions: BotPermissions::text_only(),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}
