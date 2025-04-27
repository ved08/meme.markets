use anchor_lang::prelude::*;

#[event]
pub struct StakeEvent {
    pub bet_id: u64,
    pub market_id: u64,
    pub placed_at: i64,
    pub bettor: Pubkey,
    pub amount: u64,
    pub option: u8,
    pub market: Pubkey,
    pub claimed: bool,
}

#[event]
pub struct TokenDataForRaydium {
    pub wsol_amount: u64,
    pub token_amount: u64,
    pub mint: Pubkey,
    pub market_owner: Pubkey,
}
