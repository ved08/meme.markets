pub fn calculate_tokens_to_send(
    staked_amount: u64,
    total_liquidity: u64,
    token_reserve: u64,
    is_winner: bool,
) -> (u64, u64) {
    // (tokens_allocated, sol_refund)

    if staked_amount == 0 || total_liquidity == 0 || token_reserve == 0 {
        return (0, 0);
    }

    let maximum: u64 = 30_000_000_000_000u64; // 3% of 1 billion
    let tokens_allocated = if is_winner {
        (token_reserve as u128)
            .checked_mul(staked_amount as u128)
            .unwrap()
            .checked_div(total_liquidity as u128)
            .unwrap()
    } else {
        (token_reserve as u128)
            .checked_mul(staked_amount as u128)
            .unwrap()
            .checked_div(total_liquidity as u128)
            .unwrap()
            / 2
    };

    // Return the lesser of `tokens_allocated` or `maximum`
    if tokens_allocated < maximum as u128 {
        (tokens_allocated as u64, 0)
    } else {
        let excess_tokens = tokens_allocated.checked_sub(maximum as u128).unwrap();
        let refund_sol = (excess_tokens * total_liquidity as u128)
            .checked_div(token_reserve as u128)
            .unwrap();

        return (maximum as u64, refund_sol as u64);
    }
}
