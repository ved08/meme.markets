// INTEGRATE amm creation here only
use anchor_lang::{prelude::*, solana_program::native_token::LAMPORTS_PER_SOL};
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::{error::PumpstakeErrors, state::PredictionMarket};

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"market", market.owner.key().as_ref(), market.market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, PredictionMarket>,
    pub system_program: Program<'info, System>,
}
impl<'info> ResolveMarket<'info> {
    pub fn resolve_winner(&mut self, option: u8) -> Result<()> {
        // ADD THIS LATER
        // require!(self.market.is_active, PumpstakeErrors::MarketExpired);
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp * 1000;
        require!(
            self.market.end_time <= timestamp,
            PumpstakeErrors::MarketActive
        );
        msg!("self.market = {}", self.market.end_time);
        msg!("total market cap = {}", self.market.total_mc);
        // CHANGE THIS TO 100
        if self.market.total_mc < 10 * LAMPORTS_PER_SOL {
            self.market.graduate = Some(false);
        } else {
            self.market.graduate = Some(true); // create amm and graduate to raydium
        }
        self.market.is_active = false;
        self.market.winner = Some(option);
        Ok(())
    }
}

// Add a check in resolve market for 100sol
// if 100 sol, set graduate to true
// else graudate to false
// if true, raydium ix different
// else graduate ix
