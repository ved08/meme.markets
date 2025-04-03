use anchor_lang::prelude::*;

use super::MAX_OPTIONS;
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct PredictionMarketParams {
    ticker: String,
    name: String,
    image: String,
    description: String,
    twitter: String,
    website: String,
    telegram: String,
}

impl Space for PredictionMarketParams {
    const INIT_SPACE: usize = 8 + 36 + 76 + 154 + 36 + 36 + 36;
}

#[account]
#[derive(InitSpace)]
pub struct PredictionMarket {
    pub market_id: u64,
    pub bump: u8,
    pub owner: Pubkey,
    #[max_len(MAX_OPTIONS)]
    pub market_options: Vec<BettingOption>,
    pub start_time: i64,
    pub end_time: i64,
    pub is_active: bool,
    pub winner: Option<u8>,
    pub total_mc: u64,
    pub data: PredictionMarketParams,
    pub total_tokens: u64,
    pub graduate: Option<bool>,
}

#[account]
#[derive(InitSpace)]
pub struct Bet {
    pub bet_id: u64,
    pub market_id: u64,
    pub placed_at: i64,
    pub bettor: Pubkey,
    pub amount: u64,
    pub option: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BettingOption {
    pub option_id: u8,       // Unique ID for this option (e.g., 0, 1, 2...)
    pub description: String, // Description (e.g., "Team A wins")
    pub liquidity: u64,      // Amount of tokens staked/pooled for this option
}

impl Space for BettingOption {
    const INIT_SPACE: usize = 1 + 154 + 8;
}
