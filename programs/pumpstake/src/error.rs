use anchor_lang::prelude::*;

#[error_code]
pub enum PumpstakeErrors {
    #[msg("Market already expired. Cannot place bet")]
    MarketExpired,
    #[msg("Market is still active")]
    MarketActive,
    #[msg("Not authorized to execute the instruction")]
    NotAuthorized,
    #[msg("Number of options exceed max options")]
    MaxOptionsExceeded,
}
