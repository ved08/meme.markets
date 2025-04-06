pub fn calculate_tokens_to_send(
    staked_amount: u64,
    total_liquidity: u64,
    token_reserve: u64,
    is_winner: bool,
) -> u64 {
    // Determine allocation limits
    if staked_amount <= 0 || total_liquidity <= 0 || token_reserve <= 0 {
        return 0;
    }

    let minimum: u64 = 24_00_000; // 3% of 80 Million
    let tokens_allocated;
    if is_winner {
        tokens_allocated = (token_reserve * staked_amount) / total_liquidity;
    } else {
        tokens_allocated = (token_reserve * (staked_amount / 2)) / total_liquidity;
    }
    if minimum < tokens_allocated {
        minimum
    } else {
        tokens_allocated
    }
}
