use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AMM {
    pub mint: Pubkey,
    #[max_len(70)]
    pub uri: String,
    pub creator: Pubkey,
    pub cp_ratio: u64,
    pub sol_cap: u64,
    pub mint_cap: u64,
    pub sol_reserve_bump: u8,
    pub seed: u64,
    pub amm_bump: u8,
    pub mint_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitTokenParams {
    pub name: String,
    pub seed: u64,
    pub symbol: String,
    pub uri: String,
    pub sol_cap: u64,
    pub mint_cap: u64,
    pub decimals: u8,
}
