use crate::error::*;
use crate::state::{Bet, PredictionMarket};
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

#[derive(Accounts)]
#[instruction(bet_id: u64)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [b"market", signer.key().as_ref(), market.market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, PredictionMarket>,
    #[account(
        init,
        payer = signer,
        seeds = [b"bet", market.key().as_ref(), signer.key().as_ref(), bet_id.to_le_bytes().as_ref()],
        space = 8 + Bet::INIT_SPACE,
        bump
    )]
    pub bet: Account<'info, Bet>,
    ///CHECK: the vault will be derived in frontend as there are multiple options
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> Stake<'info> {
    pub fn place_bet(&mut self, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        let timestamp = clock.unix_timestamp;
        require!(
            timestamp < self.market.end_time,
            PumpstakeErrors::MarketExpired
        );
        self.bet.set_inner(Bet {
            market_id: self.market.market_id,
            placed_at: timestamp,
            bettor: self.signer.to_account_info().key(),
            amount,
            option: self.vault.key(),
        });
        let accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(ctx, amount)?;
        Ok(())
    }
}
