use anyhow::Result;
use candid::{Nat, Principal};
use ic_cdk::api::call;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc2::approve::ApproveArgs;

const DEFAULT_FEE: u128 = 10000;
const DEFAULT_DECIMALS: u8 = 8;

pub async fn icrc2_approve(
    canister_id: &str,
    spender: &str,
    amount: f64,
    expires_at: Option<u64>,
) -> Result<()> {
    let canister = Principal::from_text(canister_id)
        .map_err(|e| anyhow::anyhow!("Invalid canister ID: {}", e))?;

    let token_amount = super::amount_to_tokens(amount, DEFAULT_DECIMALS);

    let args = ApproveArgs {
        from_subaccount: None,
        spender: Account {
            owner: Principal::from_text(spender)
                .map_err(|e| anyhow::anyhow!("Invalid spender principal: {}", e))?,
            subaccount: None,
        },
        amount: Nat::from(token_amount),
        expected_allowance: None,
        expires_at,
        fee: Some(Nat::from(DEFAULT_FEE)),
        memo: None,
        created_at_time: None,
    };

    // Use ic_cdk's call function to make inter-canister calls
    // Fix: The return type should be a tuple (Nat,) not a Result
    let _block_index: (Nat,) = call::call(canister, "icrc2_approve", (args,))
        .await
        .map_err(|e| anyhow::anyhow!("Call failed: {:?}", e))?;

    // If we got here, the call succeeded
    Ok(())
}
