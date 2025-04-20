pub fn calculate_tokens_to_send(
    staked_amount: u64,
    total_liquidity: u64,
    token_reserve: u64,
    is_winner: bool,
) -> u64 {
    // Determine allocation limits
    if staked_amount == 0 || total_liquidity == 0 || token_reserve == 0 {
        return 0;
    }

    let minimum: u64 = 30_000_000_000_000; // 3% of 80 Million
    let tokens_allocated = if is_winner {
        ((token_reserve as u128) * (staked_amount as u128)) / (total_liquidity as u128)
    } else {
        ((token_reserve as u128) * ((staked_amount / 2) as u128)) / (total_liquidity as u128)
    };

    // Return the lesser of `tokens_allocated` or `minimum`
    if tokens_allocated < minimum as u128 {
        tokens_allocated as u64
    } else {
        minimum
    }
}
