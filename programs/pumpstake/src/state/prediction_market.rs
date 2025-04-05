use anchor_lang::prelude::*;

use super::MAX_OPTIONS;
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct PredictionMarketParams {
    market_type: u8,
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
    pub option_id: u8,
    pub name: String,
    pub image: String,
    pub description: String,
    pub liquidity: u64,
}

impl Space for BettingOption {
    const INIT_SPACE: usize = 1 + 10 + 100 + 154 + 8;
}
