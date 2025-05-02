use crate::router::commands::crypto::icrc::icrc1::icrc1_transfer;
use anyhow::Result;
use async_trait::async_trait;
use candid::Principal;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::oc_api::client::Client;
use oc_bots_sdk::types::BotCommandContext;
use oc_bots_sdk_canister::CanisterRuntime;
use std::str::FromStr;
use std::sync::LazyLock;

// Define the command as a static definition
static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(|| Icrc1Transfer::definition());

pub struct Icrc1Transfer;

#[async_trait]
impl CommandHandler<CanisterRuntime> for Icrc1Transfer {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        client: Client<CanisterRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        // Extract parameters from command
        let canister_id = client.context().command.arg::<String>("canister_id");

        let from = client.context().command.arg::<String>("from");

        let to = client.context().command.arg::<String>("to");

        // Parse the amount as a f64 from a string
        let amount_str = client.context().command.arg::<String>("amount");

        let amount = amount_str
            .parse::<f64>()
            .map_err(|e| format!("Invalid amount format: {}", e))?;

        // Execute the transfer
        let result = icrc1_transfer_token(&canister_id, &from, &to, amount)
            .await
            .map_err(|e| e.to_string());

        // Return success or error message
        let response = match result {
            Ok(_) => format!(
                "✅ Successfully transferred {} tokens from {} to {} on ledger canister {}",
                amount, from, to, canister_id
            ),
            Err(e) => format!("❌ Transfer failed: {}", e),
        };

        let message = client
            .send_text_message(response)
            .execute_then_return_message(|_, _| ());

        Ok(SuccessResult { message })
    }
}

impl Icrc1Transfer {
    pub fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "icrc1_transfer".to_string(),
            description: Some("Transfer ICRC-1 tokens between accounts".to_string()),
            placeholder: Some("Processing ICRC-1 transfer...".to_string()),
            params: vec![
                BotCommandParam {
                    name: "canister_id".to_string(),
                    description: Some("The canister ID of the ICRC-1 token ledger".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 100,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("Enter the canister ID".to_string()),
                },
                BotCommandParam {
                    name: "from".to_string(),
                    description: Some("The principal sending the tokens".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 100,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("Enter the sender's principal ID".to_string()),
                },
                BotCommandParam {
                    name: "to".to_string(),
                    description: Some("The principal receiving the tokens".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 100,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("Enter the recipient's principal ID".to_string()),
                },
                BotCommandParam {
                    name: "amount".to_string(),
                    description: Some("The amount of tokens to transfer".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 20,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("Enter the amount to transfer (e.g., 1.23)".to_string()),
                },
            ],
            permissions: BotPermissions::from_message_permission(MessagePermission::Text),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}

pub async fn icrc1_transfer_token(
    canister_id: &str,
    from: &str,
    to: &str,
    amount: f64,
) -> Result<()> {
    // Validate principals
    let _from = Principal::from_str(from)
        .map_err(|e| anyhow::anyhow!("Invalid 'from' principal: {}", e))?;
    let _to =
        Principal::from_str(to).map_err(|e| anyhow::anyhow!("Invalid 'to' principal: {}", e))?;

    // Execute transfer through ICRC-1 interface
    icrc1_transfer(canister_id, from, to, amount).await?;
    Ok(())
}
