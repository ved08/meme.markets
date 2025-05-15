use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{state::{Bet, PredictionMarket}, utils::calculate_tokens_to_send, error::PumpstakeErrors};

#[derive(Accounts)]
pub struct ClaimTokenReward<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK: This will be market creator for signing PDA
    pub market_creator: UncheckedAccount<'info>,
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
    ///CHECK: This will be validated in the program
    #[account(mut)]
    pub receiver: UncheckedAccount<'info>,
    // #[account(
    //     seeds = [b"mint", market.key().as_ref()],
    //     bump,
    // )]
    ///CHECK:
    pub mint: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = market,
        associated_token::token_program = token_program,
      )]
    pub token_reserve: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = receiver,
        associated_token::token_program = token_program,
    )]
    pub receiver_ata: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
impl<'info> ClaimTokenReward<'info> {
    pub fn claim_tokens(&mut self, bumps: &ClaimTokenRewardBumps) -> Result<()> {
        require!(self.market.winner.is_some(), PumpstakeErrors::MarketActive);
        require!(!self.bet.claimed, PumpstakeErrors::RewardAlreadyClaimed);

        let is_winner = self.bet.option == self.market.winner.unwrap();
        if is_winner {
            let total_winner_liquidity = self
                .market
                .market_options
                .iter()
                .find(|opt| opt.option_id == self.market.winner.unwrap())
                .unwrap()
                .liquidity;
            let loser_liquidity = self.market.total_mc - total_winner_liquidity;

            let reward_pool = loser_liquidity / 2;
            msg!("winner: {}, claimed: {}", total_winner_liquidity, self.market.claimed_winner_liquidity);
            let remaining_winner_liquidity =
                total_winner_liquidity
                - self.market.claimed_winner_liquidity;
            msg!("reward_pool: {}, distributed: {}", reward_pool, self.market.distributed_rewards);
            let remaining_reward_pool = reward_pool.checked_sub(self.market.distributed_rewards).unwrap();

            let ratio = self.bet.amount as u128 * 1_000_000 / remaining_winner_liquidity as u128;
            let reward_share = remaining_reward_pool as u128 * ratio / 1_000_000;

            let winner_share_amount = reward_share as u64;

            self.market.claimed_winner_liquidity += self.bet.amount;
            self.market.distributed_rewards += reward_share as u64;

            let signer_seeds: &[&[&[u8]]] = &[&[
                b"vault",
                self.market.to_account_info().key.as_ref(),
                &[bumps.market_vault],
            ]];
            let accounts = Transfer {
                from: self.market_vault.to_account_info(),
                to: self.receiver.to_account_info(),
            };
            let ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                accounts,
                signer_seeds,
            );
            transfer(ctx, winner_share_amount)?;
            // self.market.total_mc -= winner_share_amount;
        }
        let sol_in_for_allocation;
        if is_winner {
            sol_in_for_allocation = self.bet.amount
        } else {
            sol_in_for_allocation = self.bet.amount / 2
        }
        let (tokens_to_send, refund_amount) = calculate_tokens_to_send(
            sol_in_for_allocation, 
            self.market.virtual_sol_reserve as u128,
            self.market.total_tokens as u128,
            is_winner,
        );
        msg!("bet_amount: {}, sol_refund: {}", sol_in_for_allocation, refund_amount);
        let net_sol = (sol_in_for_allocation).checked_sub(refund_amount).unwrap();
        self.market.total_tokens -= tokens_to_send;
        self.market.virtual_sol_reserve += net_sol;
        
        msg!("virtual_sol_cap: {}, total_tokens: {}", self.market.virtual_sol_reserve, self.market.total_tokens);
        if refund_amount > 0 {
            let signer_seeds: &[&[&[u8]]] = &[&[
                b"vault",
                self.market.to_account_info().key.as_ref(),
                &[bumps.market_vault],
            ]];
            let accounts = Transfer {
                from: self.market_vault.to_account_info(),
                to: self.receiver.to_account_info(),
            };
            let ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                accounts,
                signer_seeds,
            );
            transfer(ctx, refund_amount)?;
        }

        let accounts = TransferChecked {
            from: self.token_reserve.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.receiver_ata.to_account_info(),
            authority: self.market.to_account_info(),
        };
        let binding = self.market.market_id.to_le_bytes();
        let seeds = &[
            b"market",
            self.market_creator.to_account_info().key.as_ref(),
            binding.as_ref(),
            &[bumps.market],
        ];
        let signer = [&seeds[..]];
        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            &signer,
        );
        transfer_checked(ctx, tokens_to_send, 6)?;

        let option = self.market.market_options.iter_mut()
        .find(|opt| opt.option_id == self.bet.option).unwrap();
        
        if is_winner == false {
            option.liquidity -= self.bet.amount;
        }
        
        msg!("Tokens allocated: {}", tokens_to_send);
        self.bet.claimed = true;
        Ok(())
    }
}
