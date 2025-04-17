use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    error::PumpstakeErrors,
    state::{Bet, PredictionMarket},
};

#[derive(Accounts)]
pub struct ClaimReward<'info> {
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

impl<'info> ClaimReward<'info> {
    pub fn claim_reward(&mut self, bumps: &ClaimRewardBumps) -> Result<()> {
        // require_keys_eq!(
        //     self.signer.key(),
        //     self.market.owner.key(),
        //     PumpstakeErrors::NotAuthorized
        // );
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
        require!(
            self.market.graduate == Some(false),
            PumpstakeErrors::SufficientLiquidityToGraduate
        );
        // If winner
        if self.bet.option.eq(&self.market.winner.unwrap()) {
            // Distribute funds here or calculate for raydium
            // if self.market.total_mc < 100 * LAMPORTS_PER_SOL {
            // refund process
            let losers_liquidity = self.market.total_mc
                - self
                    .market
                    .market_options
                    .iter()
                    .find(|opt| opt.option_id == self.market.winner.unwrap())
                    .unwrap()
                    .liquidity;
            let winner_liquidity = self.market.total_mc - losers_liquidity;
            let winner_share_amount = (self.bet.amount / winner_liquidity) * (losers_liquidity / 2);

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
            transfer(ctx, winner_share_amount)?;
            // } else {
            //     // create amm will come in resolve_market.rs
            // }
        } else {
            // Refund to the loser
            let losers_liquidity = (self.market.total_mc
                - self
                    .market
                    .market_options
                    .iter()
                    .find(|opt| opt.option_id == self.market.winner.unwrap())
                    .unwrap()
                    .liquidity);
            let loser_share_amount = (self.bet.amount / losers_liquidity) * (losers_liquidity / 2);

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
            transfer(ctx, loser_share_amount)?;
        }

        Ok(())
    }
}
