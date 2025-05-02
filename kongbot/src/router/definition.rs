use super::commands;
use oc_bots_sdk::{
    api::definition::{AutonomousConfig, BotDefinition},
    types::BotPermissions,
};
use oc_bots_sdk_canister::{HttpRequest, HttpResponse};

pub async fn get(_request: HttpRequest) -> HttpResponse {
    HttpResponse::json(
        200,
        &BotDefinition {
            description: "Use this bot to interact with cryptocurrency services.\n\nFeatures:\n- Get real-time crypto prices\n- Set price alerts\n- View market trends\n- Execute token trades\n- ICRC token operations\n\nExample:\n```\n/price \"bitcoin,ethereum\" \"usd\"\n```".to_string(),
            commands: commands::definitions(),
            autonomous_config: Some(AutonomousConfig {
                permissions: BotPermissions::text_only(),
                sync_api_key: true,
            }),
        },
    )
}
