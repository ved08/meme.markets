use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    error::PumpstakeErrors,
    state::{Bet, PredictionMarket},
};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"market", market.owner.key().as_ref(), market.market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, PredictionMarket>,
    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump
    )]
    pub market_vault: SystemAccount<'info>,
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    #[account(mut)]
    /// CHECK: This will come from bet pda in frontend
    pub reciever: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self, bumps: &RefundBumps) -> Result<()> {
        require!(
            self.market.winner_present == false,
            PumpstakeErrors::CannotRefund
        );
        let timestamp = Clock::get().unwrap().unix_timestamp * 1000;

        require!(
            self.market.end_time <= timestamp,
            PumpstakeErrors::MarketActive
        );
        require!(
            self.market.is_active == false,
            PumpstakeErrors::MarketActive
        );
        require_keys_eq!(
            self.bet.bettor,
            self.reciever.to_account_info().key(),
            PumpstakeErrors::IncorrectReceivor
        );

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            self.market.to_account_info().key.as_ref(),
            &[bumps.market_vault],
        ]];
        let accounts = Transfer {
            from: self.market_vault.to_account_info(),
            to: self.reciever.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        let amount = self.bet.amount;
        transfer(ctx, amount)?;

        self.bet.claimed = true;
        Ok(())
    }
}
