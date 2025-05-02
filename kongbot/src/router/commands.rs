use oc_bots_sdk::api::command::CommandHandlerRegistry;
use oc_bots_sdk::api::definition::BotCommandDefinition;
use oc_bots_sdk_canister::{
    CanisterRuntime, HttpRequest as SdkHttpRequest, HttpResponse as SdkHttpResponse,
    OPENCHAT_CLIENT_FACTORY,
};
use std::sync::LazyLock;

mod crypto;

static COMMANDS: LazyLock<CommandHandlerRegistry<CanisterRuntime>> = LazyLock::new(|| {
    CommandHandlerRegistry::new(OPENCHAT_CLIENT_FACTORY.clone())
        .register(crypto::Alert)
        .register(crypto::Market)
        .register(crypto::Price)
        .register(crypto::SetTrade)
        .register(crypto::Trending)
});

pub fn definitions() -> Vec<BotCommandDefinition> {
    vec![
        crypto::Alert::definition().clone(),
        crypto::Market::definition().clone(),
        crypto::Price::definition().clone(),
        crypto::SetTrade::definition().clone(),
        crypto::Trending::definition().clone(),
    ]
}

pub async fn execute(req: SdkHttpRequest) -> SdkHttpResponse {
    let body_str = String::from_utf8(req.body).unwrap_or_default();

    // Parse command request
    let command_request = match serde_json::from_str(&body_str) {
        Ok(cr) => cr,
        Err(e) => {
            return SdkHttpResponse {
                status_code: 400,
                headers: vec![("Content-Type".to_string(), "application/json".to_string())],
                body: format!(
                    r#"{{"error": "Invalid command request: {}", "success": false}}"#,
                    e
                )
                .into_bytes(),
            }
        }
    };

    // Execute command using the registry with default context and time
    let context = Default::default();
    let now = 0; // Use current time or pass from caller if needed
    let response = COMMANDS.execute(command_request, context, now).await;

    SdkHttpResponse {
        status_code: 200,
        headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        body: serde_json::to_vec(&response).unwrap_or_default(),
    }
}
