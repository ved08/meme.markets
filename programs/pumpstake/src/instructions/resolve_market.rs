// INTEGRATE amm creation here only
use anchor_lang::prelude::*;

use crate::{error::PumpstakeErrors, state::PredictionMarket};

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [b"market", signer.key().as_ref(), market.market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, PredictionMarket>,
}
impl<'info> ResolveMarket<'info> {
    pub fn resolve_winner(&mut self, option: u8) -> Result<()> {
        require_keys_eq!(
            self.market.owner,
            self.signer.to_account_info().key(),
            PumpstakeErrors::NotAuthorized
        );
        let timestamp = Clock::get().unwrap().unix_timestamp;
        require!(
            self.market.end_time <= timestamp,
            PumpstakeErrors::MarketActive
        );
        self.market.is_active = false;
        self.market.winner = option;
        Ok(())
    }
}
