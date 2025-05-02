use async_trait::async_trait;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::oc_api::client::Client;
use oc_bots_sdk::types::BotCommandContext;
// Change to CanisterRuntime
use oc_bots_sdk_canister::CanisterRuntime;
// Remove reqwest client
// use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

// Change the LazyLock initialization to match the function signature
static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(|| Price::definition());

#[derive(Deserialize, Debug)]
pub struct PriceResponse {
    #[serde(flatten)]
    pub prices: HashMap<String, HashMap<String, f64>>,
    #[serde(skip)] // Add skip for non-deserializable field
    pub mapped_id: String, // Store the ID used in the response
}

pub struct Price;

#[async_trait]
// Change to CanisterRuntime
impl CommandHandler<CanisterRuntime> for Price {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        // Change to CanisterRuntime
        client: Client<CanisterRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        let coin_ids_str = client
            .context()
            .command
            .arg::<String>("coins")
            .to_lowercase();
        let currency = client
            .context()
            .command
            .arg::<String>("currency")
            .to_lowercase();
        let currency_upper = currency.to_uppercase();

        // FIXME: HTTP calls need to be implemented using IC HTTPS Outcalls
        // let price_data = self.fetch_crypto_price(&coin_ids_str, &currency).await?;
        // let response = self.format_price_data(&price_data, &currency_upper);

        let response = format!(
            "Price fetching feature (for coins: {}, currency: {}) is not yet implemented for on-chain execution.",
            coin_ids_str, currency_upper
        );

        let message = client
            .send_text_message(response)
            .with_block_level_markdown(true)
            .execute_then_return_message(|_, _| ());

        Ok(SuccessResult { message })
    }
}

impl Price {
    // FIXME: HTTP calls need to be implemented using IC HTTPS Outcalls
    // pub(crate) async fn fetch_crypto_price(
    //     &self,
    //     coin_ids: &str,
    //     currency: &str,
    // ) -> Result<PriceResponse, String> {
    //     let client = ReqwestClient::new();
    //     let url = format!(
    //         "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}",
    //         coin_ids,
    //         currency
    //     );
    //
    //     let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    //
    //     if !response.status().is_success() {
    //         return Err(format!(
    //             "Failed to fetch price data: Status {}",
    //             response.status()
    //         ));
    //     }
    //
    //     let mut price_response = response
    //         .json::<PriceResponse>()
    //         .await
    //         .map_err(|e| e.to_string())?;
    //
    //     // Store the first key found as the mapped_id (assuming single coin for now)
    //     if let Some(key) = price_response.prices.keys().next() {
    //         price_response.mapped_id = key.clone();
    //     }
    //
    //     Ok(price_response)
    // }
    //
    // fn format_price_data(&self, data: &PriceResponse, currency_upper: &str) -> String {
    //     let mut response = String::from("**Current Prices**\n\n");
    //     for (coin_id, prices) in &data.prices {
    //         if let Some(price) = prices.get(currency_upper.to_lowercase().as_str()) {
    //             response.push_str(&format!(
    //                 "**{}**: {:.2} {}\n",
    //                 coin_id.to_uppercase(),
    //                 price,
    //                 currency_upper
    //             ));
    //         } else {
    //             response.push_str(&format!(
    //                 "**{}**: Price not available in {}\n",
    //                 coin_id.to_uppercase(),
    //                 currency_upper
    //             ));
    //         }
    //     }
    //     response
    // }

    pub fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "price".to_string(),
            description: Some("Get current price for cryptocurrency".to_string()),
            placeholder: Some("Fetching price data...".to_string()),
            params: vec![
                BotCommandParam {
                    name: "coins".to_string(),
                    description: Some("Cryptocurrency IDs (e.g., bitcoin,ethereum)".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 100,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("Enter coin ID(s)".to_string()),
                },
                BotCommandParam {
                    name: "currency".to_string(),
                    description: Some("Currency to display price in".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 3,
                        max_length: 3,
                        choices: vec![
                            BotCommandOptionChoice {
                                name: "USD (US Dollar)".to_string(),
                                value: "usd".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "EUR (Euro)".to_string(),
                                value: "eur".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "KSH (Kenyan Shilling)".to_string(),
                                value: "kes".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "BTC (Bitcoin)".to_string(),
                                value: "btc".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "ETH (Ethereum)".to_string(),
                                value: "eth".to_string(),
                            },
                        ],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("Select currency".to_string()),
                },
            ],
            permissions: BotPermissions::from_message_permission(MessagePermission::Text),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}
