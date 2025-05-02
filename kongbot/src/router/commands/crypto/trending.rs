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
use std::sync::LazyLock;

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(Trending::definition);

#[derive(Deserialize, Debug)]
struct TrendingCoin {
    id: String,
    name: String,
    symbol: String,
    market_cap_rank: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct TrendingItem {
    item: TrendingCoin,
}

#[derive(Deserialize, Debug)]
struct TrendingResponse {
    coins: Vec<TrendingItem>,
}

pub struct Trending;

#[async_trait]
// Change to CanisterRuntime
impl CommandHandler<CanisterRuntime> for Trending {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        // Change to CanisterRuntime
        client: Client<CanisterRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        // FIXME: HTTP calls need to be implemented using IC HTTPS Outcalls
        // let trending_data = self.fetch_trending_coins().await?;
        // let response = self.format_trending_data(&trending_data);

        let response =
            "Trending coins feature is not yet implemented for on-chain execution.".to_string();

        let message = client
            .send_text_message(response)
            .with_block_level_markdown(true)
            .execute_then_return_message(|_, _| ());

        Ok(SuccessResult { message })
    }
}

impl Trending {
    // FIXME: HTTP calls need to be implemented using IC HTTPS Outcalls
    // async fn fetch_trending_coins(&self) -> Result<TrendingResponse, String> {
    //     let client = ReqwestClient::new();
    //     let url = "https://api.coingecko.com/api/v3/search/trending";
    //
    //     let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    //
    //     if !response.status().is_success() {
    //         return Err(format!(
    //             "Failed to fetch trending data: Status {}",
    //             response.status()
    //         ));
    //     }
    //
    //     response
    //         .json::<TrendingResponse>()
    //         .await
    //         .map_err(|e| e.to_string())
    // }
    //
    // fn format_trending_data(&self, data: &TrendingResponse) -> String {
    //     let mut response = String::from("**🔥 Top-7 Trending Coins on CoinGecko**\n\n");
    //     for (index, item) in data.coins.iter().enumerate() {
    //         let coin = &item.item;
    //         let rank = coin.market_cap_rank.map_or_else(|| "N/A".to_string(), |r| r.to_string());
    //         response.push_str(&format!(
    //             "**{}. {} ({})**\n",
    //             index + 1,
    //             coin.name,
    //             coin.symbol.to_uppercase()
    //         ));
    //         response.push_str(&format!("   Rank: {}\n\n", rank));
    //     }
    //     response
    // }

    pub fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "trending".to_string(),
            description: Some("Show trending cryptocurrencies".to_string()),
            placeholder: Some("Fetching trending coins...".to_string()),
            params: vec![],
            permissions: BotPermissions::from_message_permission(MessagePermission::Text),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}
