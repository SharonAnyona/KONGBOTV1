use async_trait::async_trait;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::oc_api::actions::send_message;
use oc_bots_sdk::oc_api::client::Client;
use oc_bots_sdk::types::BotCommandContext;
use oc_bots_sdk_canister::CanisterRuntime;
use std::sync::LazyLock;

use crate::router::price;

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(CoinPrice::definition);

pub struct CoinPrice;

#[async_trait]
impl CommandHandler<CanisterRuntime> for CoinPrice {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        oc_client: Client<CanisterRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        // Get the coin argument with explicit type annotation
        let coin = oc_client.context().command.arg::<String>("coin").to_string();
        
        // Fetch the price using the price module
        let price_result = price::fetch_price(&coin).await;
        
        let response_text = match price_result {
            Ok(price) => format!("Current price of {} is ${:.2}", coin, price),
            Err(e) => format!("Failed to fetch price for {}: {}", coin, e),
        };

        // Send the message to OpenChat
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

impl CoinPrice {
    fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "coin_price".to_string(),
            description: Some("Get the current price of a cryptocurrency".to_string()),
            placeholder: None,
            params: vec![BotCommandParam {
                name: "coin".to_string(),
                description: Some("The cryptocurrency to check (e.g., bitcoin, ethereum)".to_string()),
                param_type: BotCommandParamType::StringParam(StringParam {
                    min_length: 1,
                    max_length: 50,
                    choices: Vec::new(),
                    multi_line: false,
                }),
                required: true,
                placeholder: Some("bitcoin".to_string()),
            }],
            permissions: BotPermissions::text_only(),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}
