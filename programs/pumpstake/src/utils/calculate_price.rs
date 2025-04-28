// pub fn calculate_tokens_to_send(
//     staked_amount: u64,
//     total_liquidity: u64,
//     token_reserve: u64,
//     is_winner: bool,
// ) -> (u64, u64) {
//     // (tokens_allocated, sol_refund)

//     if staked_amount == 0 || total_liquidity == 0 || token_reserve == 0 {
//         return (0, 0);
//     }

//     let maximum: u64 = 30_000_000_000_000u64; // 3% of 1 billion
//     let tokens_allocated = if is_winner {
//         (token_reserve as u128)
//             .checked_mul(staked_amount as u128)
//             .unwrap()
//             .checked_div(total_liquidity as u128)
//             .unwrap()
//     } else {
//         (token_reserve as u128)
//             .checked_mul(staked_amount as u128)
//             .unwrap()
//             .checked_div(total_liquidity as u128)
//             .unwrap()
//             / 2
//     };

//     // Return the lesser of `tokens_allocated` or `maximum`
//     if tokens_allocated < maximum as u128 {
//         (tokens_allocated as u64, 0)
//     } else {
//         let excess_tokens = tokens_allocated.checked_sub(maximum as u128).unwrap();
//         let refund_sol = (excess_tokens * total_liquidity as u128)
//             .checked_div(token_reserve as u128)
//             .unwrap();

//         return (maximum as u64, refund_sol as u64);
//     }
// }

/// Constants (6 decimals for token, 9 decimals for SOL)
const LAMPORTS_PER_SOL: u128 = 1_000_000_000;
const TOKEN_DECIMALS: u128 = 1_000_000;

/// Initial pool reference sizes (not hard-coded in functions)
const INITIAL_SOL_RESERVE: u128 = 25 * LAMPORTS_PER_SOL; // 25 SOL
const INITIAL_TOKEN_RESERVE: u128 = 1_000_000_000 * TOKEN_DECIMALS; // 1 billion tokens

/// k = sol_reserve × token_reserve
const K: u128 = INITIAL_SOL_RESERVE * INITIAL_TOKEN_RESERVE;

/// Total supply = 1 billion tokens (6-decimals)
const TOTAL_SUPPLY: u128 = INITIAL_TOKEN_RESERVE;
/// 3 % cap
const MAX_ALLOCATION: u128 = TOTAL_SUPPLY * 3 / 100; // 30 000 000 000 000 base-units

/// 1) AMM raw swap: “if you send `sol_in`, how many tokens come out?”
pub fn calculate_tokens_out_raw(sol_in: u64, sol_reserve: u128, token_reserve: u128) -> u128 {
    let new_sol = sol_reserve.checked_add(sol_in as u128).unwrap();
    let new_token = K.checked_div(new_sol).unwrap();
    let result: u128 = token_reserve - new_token;
    result
}
/// Given:
///  - sol_in        : how many lamports of SOL the user sent
///  - sol_reserve   : current SOL reserve (lamports)
///  - token_reserve : current token reserve (base units, 6 decimals)
///  - is_winner     : whether they get the full output or only half
///
/// Returns (tokens_allocated, sol_refund)
pub fn calculate_tokens_to_send(
    sol_in: u64,
    sol_reserve: u128,
    token_reserve: u128,
    is_winner: bool,
) -> (u64, u64) {
    // 1) Raw AMM swap output
    let mut tokens = calculate_tokens_out_raw(sol_in, sol_reserve, token_reserve);

    // 2) Apply loser penalty
    if !is_winner {
        tokens /= 2;
    }

    // 3) If under cap, no refund
    if tokens <= MAX_ALLOCATION {
        return (tokens as u64, 0);
    }

    // 4) Cap hit: user gets exactly MAX_ALLOCATION tokens.
    let allocated = MAX_ALLOCATION;

    // 5) Figure out how much SOL buys exactly `allocated` tokens:
    //    new_token_reserve = token_reserve - allocated
    let new_token_reserve = token_reserve.checked_sub(allocated).unwrap();
    //    new_sol_reserve   = K / new_token_reserve
    let new_sol_reserve = K.checked_div(new_token_reserve).unwrap();
    //    sol_spent         = new_sol_reserve - old sol_reserve
    let sol_spent = new_sol_reserve.checked_sub(sol_reserve).unwrap();

    // 6) Refund = what they sent minus what was actually spent
    let sol_refund = (sol_in as u128).checked_sub(sol_spent).unwrap();

    (allocated as u64, sol_refund as u64)
}
