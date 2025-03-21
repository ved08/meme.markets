use anchor_lang::{
    prelude::*,
    solana_program::native_token::LAMPORTS_PER_SOL,
    system_program::{transfer, Transfer},
};

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
    #[account(mut)]
    pub reciever: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
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
        require_keys_eq!(
            self.bet.bettor,
            self.reciever.to_account_info().key(),
            PumpstakeErrors::IncorrectReceivor
        );
        // If winner
        if self.bet.option.eq(&self.market.winner) {
            // Distribute funds here or calculate for raydium
            if self.market.total_mc < 100 * LAMPORTS_PER_SOL {
                // refund process
                let losers_liquidity = (self.market.total_mc
                    - self
                        .market
                        .market_options
                        .iter()
                        .find(|opt| opt.option_id == self.market.winner)
                        .unwrap()
                        .liquidity);
                let winner_liquidity = self.market.total_mc - losers_liquidity;
                let winner_share_amount =
                    (self.bet.amount / winner_liquidity) * (losers_liquidity / 2);

                let market_id_bytes = self.market.market_id.to_le_bytes();
                let signer_seeds: &[&[&[u8]]] = &[&[
                    b"market",
                    self.signer.to_account_info().key.as_ref(),
                    market_id_bytes.as_ref(),
                    &[self.market.bump],
                ]];
                let accounts = Transfer {
                    from: self.market.to_account_info(),
                    to: self.reciever.to_account_info(),
                };
                let ctx = CpiContext::new_with_signer(
                    self.system_program.to_account_info(),
                    accounts,
                    signer_seeds,
                );
                transfer(ctx, winner_share_amount)?;
            } else {
                // create amm will come in resolve_market.rs
            }
        } else {
            // Refund to the loser
            let losers_liquidity = (self.market.total_mc
                - self
                    .market
                    .market_options
                    .iter()
                    .find(|opt| opt.option_id == self.market.winner)
                    .unwrap()
                    .liquidity);
            let loser_share_amount = (self.bet.amount / losers_liquidity) * (losers_liquidity / 2);

            let market_id_bytes = self.market.market_id.to_le_bytes();
            let signer_seeds: &[&[&[u8]]] = &[&[
                b"market",
                self.signer.to_account_info().key.as_ref(),
                market_id_bytes.as_ref(),
                &[self.market.bump],
            ]];
            let accounts = Transfer {
                from: self.market.to_account_info(),
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
