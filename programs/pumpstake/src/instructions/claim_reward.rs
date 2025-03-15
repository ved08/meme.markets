use anchor_lang::prelude::*;

use crate::{
    error::PumpstakeErrors,
    state::{Bet, PredictionMarket},
};

#[derive(Accounts)]
struct ClaimReward<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [b"market", signer.key().as_ref(), market.market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, PredictionMarket>,
    #[account(mut)]
    pub bet: Account<'info, Bet>,
}

impl<'info> ClaimReward<'info> {
    pub fn claim_reward(&mut self) -> Result<()> {
        require_keys_eq!(
            self.signer.key(),
            self.market.owner.key(),
            PumpstakeErrors::NotAuthorized
        );
        let timestamp = Clock::get().unwrap().unix_timestamp;
        require!(
            self.market.end_time <= timestamp,
            PumpstakeErrors::MarketActive
        );
        require!(
            self.market.is_active == false,
            PumpstakeErrors::MarketActive
        );
        if self.bet.option.eq(&self.market.winner) {
            // Distribute funds here or calculate for raydium
        }

        Ok(())
    }
}
