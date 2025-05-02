use anyhow::Result;
use candid::{Nat, Principal};
use ic_cdk::api::call;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::TransferArg;

const DEFAULT_FEE: u128 = 10000;
const DEFAULT_DECIMALS: u8 = 8;

pub async fn icrc1_transfer(canister_id: &str, _from: &str, to: &str, amount: f64) -> Result<()> {
    let canister = Principal::from_text(canister_id)
        .map_err(|e| anyhow::anyhow!("Invalid canister ID: {}", e))?;

    let token_amount = super::amount_to_tokens(amount, DEFAULT_DECIMALS);

    let args = TransferArg {
        from_subaccount: None,
        to: Account {
            owner: Principal::from_text(to)
                .map_err(|e| anyhow::anyhow!("Invalid recipient principal: {}", e))?,
            subaccount: None,
        },
        fee: Some(Nat::from(DEFAULT_FEE)),
        created_at_time: None,
        memo: None,
        amount: Nat::from(token_amount),
    };

    // Use ic_cdk's call function to make inter-canister calls
    // The return type should be a tuple (Nat,) not a Result
    let _block_index: (Nat,) = call::call(canister, "icrc1_transfer", (args,))
        .await
        .map_err(|e| anyhow::anyhow!("Call failed: {:?}", e))?;

    // If we got here, the call succeeded
    Ok(())
}
