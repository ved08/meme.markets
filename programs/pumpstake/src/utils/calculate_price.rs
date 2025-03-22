use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

use crate::MAX_AMM_ERROR;
pub fn calculate_tokens_to_send(
    sol_amount: u64,
    sol_available: u64,
    tokens_available: u64,
    cp_ratio: u64,
    decimals: u8,
) -> Result<u64, i32> {
    let sol_amount: f64 = sol_amount as f64 / LAMPORTS_PER_SOL as f64;
    // TODO
    let new_sol_market_cap = (sol_available as f64 / LAMPORTS_PER_SOL as f64) as f64 + sol_amount;

    let tokens_before_sale: f64 = tokens_available as f64 / u64::pow(10, decimals as u32) as f64;
    let tokens_after_sale: f64 = cp_ratio as f64 / new_sol_market_cap;

    let tokens_to_send = round(tokens_before_sale - tokens_after_sale, 1);
    let tokens_to_send_lamports = (tokens_to_send * u64::pow(10, 1) as f64) as u64;

    // THIS IS SAME AS tokens_after_sale
    // let new_tokens_cap = BILLION as f64 - tokens_to_send;

    // Remove floating point precsion error
    let new_cp = new_sol_market_cap as f64 * tokens_after_sale;
    let error = new_cp - cp_ratio as f64;
    if error.abs() < MAX_AMM_ERROR as f64 {
        return Ok(tokens_to_send_lamports);
    } else {
        return Err(-1);
    }
}
pub fn calculate_sol_to_send(
    sol_available: u64,
    tokens_available: u64,
    token_amount: u64,
    cp_ratio: u64,
    decimals: u8,
) -> Result<u64, i32> {
    let sol_available_before_sale: f64 = sol_available as f64 / LAMPORTS_PER_SOL as f64;
    let tokens_available: f64 = tokens_available as f64 / u64::pow(10, decimals as u32) as f64;
    let token_amount = token_amount as f64 / u64::pow(10, decimals as u32) as f64;

    let sol_available_after_sale = cp_ratio as f64 / (tokens_available + token_amount);
    let sol_to_send = sol_available_before_sale - sol_available_after_sale;
    let sol_to_send = (sol_to_send * LAMPORTS_PER_SOL as f64) as u64;

    let new_cp = sol_available_after_sale * (tokens_available + token_amount);
    let error = new_cp - cp_ratio as f64;
    if error.abs() < MAX_AMM_ERROR as f64 {
        return Ok(sol_to_send);
    } else {
        return Err(-1);
    }
}
fn round(x: f64, decimals: u32) -> f64 {
    let y = 10i32.pow(decimals) as f64;
    (x * y).round() / y
}
