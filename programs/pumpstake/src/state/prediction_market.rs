use anchor_lang::prelude::*;
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
    pub market_type: u8,
    pub start_time: i64,
    pub end_time: i64,
    pub is_active: bool,
    pub winner: Pubkey,
    pub data: PredictionMarketParams,
}

#[account]
#[derive(InitSpace)]
pub struct Bet {
    pub market_id: u64,
    pub placed_at: i64,
    pub bettor: Pubkey,
    pub amount: u64,
    pub option: Pubkey,
}
