use std::str::FromStr;

use crate::error::*;
use crate::events::StakeEvent;
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
    #[account(
        init,
        payer = signer,
        seeds = [b"bet", market.key().as_ref(), signer.key().as_ref(), bet_id.to_le_bytes().as_ref()],
        space = 8 + Bet::INIT_SPACE,
        bump
    )]
    pub bet: Account<'info, Bet>,
    ///CHECK: this is revenue wallet
    #[account(mut)]
    pub revenue: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> Stake<'info> {
    pub fn place_bet(&mut self, bet_id: u64, option_id: u8, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        let timestamp = clock.unix_timestamp;
        let revenue_pubkey =
            Pubkey::from_str("GmkqS3uguupCzEbwcWYnRrhtSvNZj2ycUWWSCE4QHedr").unwrap();
        require_keys_eq!(
            self.revenue.key(),
            revenue_pubkey,
            PumpstakeErrors::IncorrectRevenueWallet
        );
        require!(
            timestamp < self.market.end_time,
            PumpstakeErrors::MarketExpired
        );
        let share = amount * 2 / 100;
        let amount = amount - share;
        self.bet.set_inner(Bet {
            bet_id,
            market_id: self.market.market_id,
            placed_at: timestamp,
            bettor: self.signer.to_account_info().key(),
            amount,
            option: option_id,
            market: self.market.to_account_info().key(),
            claimed: false,
        });
        let option = self
            .market
            .market_options
            .iter_mut()
            .find(|opt| opt.option_id == option_id)
            .unwrap();
        option.liquidity += amount;

        let accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.market_vault.to_account_info(),
        };
        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(ctx, amount)?;
        let accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.revenue.to_account_info(),
        };
        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(ctx, share)?;

        emit!(StakeEvent {
            bet_id,
            market_id: self.market.market_id,
            placed_at: timestamp,
            bettor: self.signer.to_account_info().key(),
            amount,
            option: option_id,
            market: self.market.to_account_info().key(),
            claimed: false,
        });
        self.market.total_mc += amount;
        Ok(())
    }
}
