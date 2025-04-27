use anchor_lang::prelude::*;

use super::MAX_OPTIONS;
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct PredictionMarketParams {
    pub market_type: u8,
    pub ticker: String,
    pub name: String,
    pub image: String,
    pub description: String,
    pub twitter: String,
    pub website: String,
    pub telegram: String,
}

impl Space for PredictionMarketParams {
    const INIT_SPACE: usize = 1 + 30 + 30 + 150 + 200 + 50 + 50 + 35;
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
    pub claimed_winner_liquidity: u64, // How much winner liquidity has been claimed
    pub distributed_rewards: u64,      // How much reward pool has been paid
    pub winner_present: bool,
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
    pub market: Pubkey,
    pub claimed: bool,
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
    const INIT_SPACE: usize = 1 + 30 + 150 + 200 + 8;
}
