use anyhow::Result;
use candid::Principal;
use ic_cdk::caller;

pub fn get_bot_identity() -> Result<Principal> {
    // Get the caller's principal - this is the identity of whoever called this canister
    let identity = caller();

    // For specific scenarios where you need a predetermined principal:
    // let identity = Principal::from_text("specific-principal-id")?;

    Ok(identity)
}
