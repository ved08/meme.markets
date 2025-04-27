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
        require!(!self.bet.claimed, PumpstakeErrors::RewardAlreadyClaimed);
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
            let total_winner_liquidity = self
                .market
                .market_options
                .iter()
                .find(|opt| opt.option_id == self.market.winner.unwrap())
                .unwrap()
                .liquidity;
            let loser_liquidity = self.market.total_mc - total_winner_liquidity;

            let reward_pool = loser_liquidity / 2;

            let remaining_winner_liquidity =
                total_winner_liquidity - self.market.claimed_winner_liquidity;
            let remaining_reward_pool = reward_pool - self.market.distributed_rewards;

            let ratio = self.bet.amount as u128 * 1_000_000 / remaining_winner_liquidity as u128;
            let reward_share = remaining_reward_pool as u128 * ratio / 1_000_000;

            let winner_share_amount = self.bet.amount + reward_share as u64;

            self.market.claimed_winner_liquidity += self.bet.amount;
            self.market.distributed_rewards += reward_share as u64;
            msg!(
                "Winner bet amount + reward = {} + {}",
                self.bet.amount,
                reward_share
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
            transfer(ctx, winner_share_amount)?;
            // } else {
            //     // create amm will come in resolve_market.rs
            // }
        } else {
            let loser_share_amount = self.bet.amount / 2;

            msg!(
                "loser bet amt = {}, refund = {}",
                self.bet.amount,
                loser_share_amount
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
            transfer(ctx, loser_share_amount)?;
        }
        self.bet.claimed = true;
        Ok(())
    }
}
