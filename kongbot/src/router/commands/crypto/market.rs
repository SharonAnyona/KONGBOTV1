use async_trait::async_trait;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::oc_api::client::Client;
use oc_bots_sdk::types::BotCommandContext;
// Change to CanisterRuntime
use oc_bots_sdk_canister::CanisterRuntime;
use std::sync::LazyLock;

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(Market::definition);

pub struct Market;

#[async_trait]
// Change to CanisterRuntime
impl CommandHandler<CanisterRuntime> for Market {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        // Change to CanisterRuntime
        client: Client<CanisterRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        let currency = client
            .context()
            .command
            .arg::<String>("currency")
            .to_lowercase();
        let currency_upper = currency.to_uppercase();

        // FIXME: HTTP calls need to be implemented using IC HTTPS Outcalls
        // let market_data = self.fetch_market_data(&currency).await?;
        // let response = self.format_market_data(&market_data, &currency_upper);

        let response = format!(
            "Market data feature (for currency {}) is not yet implemented for on-chain execution.",
            currency_upper
        );

        let message = client
            .send_text_message(response)
            .with_block_level_markdown(true)
            .execute_then_return_message(|_, _| ());

        Ok(SuccessResult { message })
    }
}

impl Market {
    // FIXME: HTTP calls need to be implemented using IC HTTPS Outcalls
    // async fn fetch_market_data(&self, currency: &str) -> Result<Vec<MarketData>, String> {
    //     let client = ReqwestClient::new();
    //     let url = format!(
    //         "https://api.coingecko.com/api/v3/coins/markets?vs_currency={}&order=market_cap_desc&per_page=10&page=1&sparkline=false",
    //         currency
    //     );
    //
    //     let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    //
    //     if !response.status().is_success() {
    //         return Err(format!(
    //             "Failed to fetch market data: Status {}",
    //             response.status()
    //         ));
    //     }
    //
    //     response
    //         .json::<Vec<MarketData>>()
    //         .await
    //         .map_err(|e| e.to_string())
    // }
    //
    // fn format_market_data(&self, data: &[MarketData], currency_upper: &str) -> String {
    //     let mut response = String::from("**Top 10 Cryptocurrencies by Market Cap**\n\n");
    //     for coin in data {
    //         let price = coin.current_price.get(currency_upper.to_lowercase().as_str()).unwrap_or(&0.0);
    //         let change_24h = coin.price_change_percentage_24h.unwrap_or(0.0);
    //         let rank = coin.market_cap_rank.map_or_else(|| "N/A".to_string(), |r| r.to_string());
    //
    //         response.push_str(&format!(
    //             "**{}. {} ({})**\n",
    //             rank,
    //             coin.name,
    //             coin.symbol.to_uppercase()
    //         ));
    //         response.push_str(&format!(
    //             "   Price: {:.2} {}\n",
    //             price,
    //             currency_upper
    //         ));
    //         response.push_str(&format!(
    //             "   24h Change: {:.2}%\n\n",
    //             change_24h
    //         ));
    //     }
    //     response
    // }

    pub fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "market".to_string(),
            description: Some("Get top cryptocurrencies by market cap".to_string()),
            placeholder: Some("Fetching market data...".to_string()),
            params: vec![
                BotCommandParam {
                    name: "count".to_string(),
                    description: Some("Number of coins to display (5, 10, or 25)".to_string()),
                    placeholder: Some("Select number of coins".to_string()),
                    required: true,
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 2,
                        choices: vec![
                            BotCommandOptionChoice {
                                name: "5 coins".to_string(),
                                value: "5".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "10 coins".to_string(),
                                value: "10".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "25 coins".to_string(),
                                value: "25".to_string(),
                            },
                        ],
                        multi_line: false,
                    }),
                },
                BotCommandParam {
                    name: "currency".to_string(),
                    description: Some("Select currency to display prices in".to_string()),
                    placeholder: Some("Choose currency".to_string()),
                    required: true,
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
                },
            ],
            permissions: BotPermissions::from_message_permission(MessagePermission::Text),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}
