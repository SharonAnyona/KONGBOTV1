use async_trait::async_trait;
use oc_bots_sdk::api::command::{CommandHandler, SuccessResult};
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk::oc_api::client::Client;
use oc_bots_sdk::types::BotCommandContext;
// Change to CanisterRuntime
use oc_bots_sdk_canister::CanisterRuntime;
use std::sync::LazyLock;

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(|| SetTrade::definition());

pub struct SetTrade;

#[async_trait]
// Change to CanisterRuntime
impl CommandHandler<CanisterRuntime> for SetTrade {
    fn definition(&self) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        // Change to CanisterRuntime
        client: Client<CanisterRuntime, BotCommandContext>,
    ) -> Result<SuccessResult, String> {
        let action = client.context().command.arg::<String>("action");
        let canister_id = client.context().command.arg::<String>("canister_id");
        let amount = client
            .context()
            .command
            .arg::<String>("amount")
            .parse::<f64>()
            .map_err(|_| "Invalid amount value. Please enter a valid number.".to_string())?;
        let from_principal = client.context().command.initiator.to_string(); // Use initiator as 'from'

        let response_text = match action.as_str() {
            "transfer" => {
                let to_principal = client.context().command.arg::<String>("to_principal");
                // FIXME: Implement icrc1_transfer_token using ic_cdk::call
                // icrc1_transfer_token(&canister_id, &from_principal, &to_principal, amount).await?;
                format!(
                    "✅ Transfer initiated ({} -> {} for {} tokens on canister {}). \n(Note: On-chain transfer execution needs implementation)",
                    from_principal,
                    to_principal,
                    amount,
                    canister_id
                )
            }
            "approve" => {
                let spender_principal = client.context().command.arg::<String>("spender_principal");
                // FIXME: Implement icrc2_approve_token using ic_cdk::call
                // icrc2_approve_token(&canister_id, &from_principal, &spender_principal, amount).await?;
                format!(
                    "✅ Approval initiated ({} approves {} for {} tokens on canister {}). \n(Note: On-chain approval execution needs implementation)",
                    from_principal,
                    spender_principal,
                    amount,
                    canister_id
                )
            }
            _ => return Err("Invalid action specified. Use 'transfer' or 'approve'.".to_string()),
        };

        let message = client
            .send_text_message(response_text)
            .with_block_level_markdown(true)
            .execute_then_return_message(|_, _| ());

        Ok(SuccessResult { message })
    }
}

impl SetTrade {
    pub fn definition() -> BotCommandDefinition {
        BotCommandDefinition {
            name: "settrade".to_string(),
            description: Some("Set up automated trading on DEX".to_string()),
            placeholder: Some("Setting up trade...".to_string()),
            params: vec![
                BotCommandParam {
                    name: "type".to_string(),
                    description: Some("Type of trade to set up".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 3,
                        max_length: 10,
                        choices: vec![
                            BotCommandOptionChoice {
                                name: "Swap tokens".to_string(),
                                value: "swap".to_string(),
                            },
                            BotCommandOptionChoice {
                                name: "Limit order".to_string(),
                                value: "limit".to_string(),
                            },
                        ],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("Select trade type".to_string()),
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
                    placeholder: Some("Enter amount".to_string()),
                },
                BotCommandParam {
                    name: "slippage".to_string(),
                    description: Some("Maximum slippage percentage".to_string()),
                    param_type: BotCommandParamType::StringParam(StringParam {
                        min_length: 1,
                        max_length: 5,
                        choices: vec![],
                        multi_line: false,
                    }),
                    required: true,
                    placeholder: Some("Enter slippage %".to_string()),
                },
            ],
            permissions: BotPermissions::from_message_permission(MessagePermission::Text),
            default_role: None,
            direct_messages: Some(true),
        }
    }
}
