use anchor_lang::prelude::*;

#[error_code]
pub enum PumpstakeErrors {
    #[msg("Market already expired. Cannot place bet")]
    MarketExpired,
}
