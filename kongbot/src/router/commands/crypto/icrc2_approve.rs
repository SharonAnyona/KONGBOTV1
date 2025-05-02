use crate::router::commands::crypto::icrc::icrc2::icrc2_approve;
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
static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(|| Icrc2Approve::definition());

pub struct Icrc2Approve;

#[async_trait]
impl CommandHandler<CanisterRuntime> for Icrc2Approve {
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

        let spender = client.context().command.arg::<String>("spender");

        // Parse the amount as a f64 from a string
        let amount_str = client.context().command.arg::<String>("amount");

        let amount = amount_str
            .parse::<f64>()
            .map_err(|e| format!("Invalid amount format: {}", e))?;

        // Execute the approval
        let result = icrc2_approve_token(&canister_id, &from, &spender, amount)
            .await
            .map_err(|e| e.to_string());

        // Return success or error message
        let response = match result {
            Ok(_) => format!(
                "✅ Successfully approved spending of {} tokens for {} from {} on ledger canister {}",
                amount, spender, from, canister_id
            ),
            Err(e) => format!("❌ Approval failed: {}", e),
        };

        let message = client
            .send_text_message(response)
            .execute_then_return_message(|_, _| ());

        Ok(SuccessResult { message })
    }
}

impl Icrc2Approve {
    pub fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "icrc2_approve".to_string(),
            description: Some("Approve ICRC-2 token spending".to_string()),
            placeholder: Some("Processing ICRC-2 approval...".to_string()),
            params: vec![
                BotCommandParam {
                    name: "canister_id".to_string(),
                    description: Some("The canister ID of the ICRC-2 token ledger".to_string()),
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
                    description: Some("The principal approving the spending".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 100,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("Enter the approver's principal ID".to_string()),
                },
                BotCommandParam {
                    name: "spender".to_string(),
                    description: Some("The principal being authorized to spend".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 100,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("Enter the spender's principal ID".to_string()),
                },
                BotCommandParam {
                    name: "amount".to_string(),
                    description: Some("The amount of tokens to approve for spending".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 20,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("Enter the amount to approve (e.g., 1.23)".to_string()),
                },
            ],
            permissions: BotPermissions::from_message_permission(MessagePermission::Text),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}

pub async fn icrc2_approve_token(
    canister_id: &str,
    from: &str,
    spender: &str,
    amount: f64,
) -> Result<()> {
    // Validate principals
    let _from = Principal::from_str(from)
        .map_err(|e| anyhow::anyhow!("Invalid 'from' principal: {}", e))?;
    let _spender = Principal::from_str(spender)
        .map_err(|e| anyhow::anyhow!("Invalid 'spender' principal: {}", e))?;

    // Execute approval through ICRC-2 interface
    // Pass correct arguments and add None for the optional expires_at
    icrc2_approve(canister_id, spender, amount, None).await?;
    Ok(())
}
