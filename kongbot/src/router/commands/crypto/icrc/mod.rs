pub mod icrc1;
pub mod icrc2;
pub mod identity;

/// Converts a floating point amount to token units based on decimals
pub fn amount_to_tokens(amount: f64, decimals: u8) -> u128 {
    let factor = 10u128.pow(decimals as u32);
    (amount * (factor as f64)) as u128
}
