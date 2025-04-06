use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{state::{Bet, PredictionMarket}, utils::calculate_tokens_to_send, error::PumpstakeErrors};

#[derive(Accounts)]
pub struct ClaimTokenReward<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut, 
        seeds = [b"market", signer.key().as_ref(), market.market_id.to_le_bytes().as_ref()],
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
    #[account(
        seeds = [b"mint", market.key().as_ref()],
        bump,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = market
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

        let is_winner = self.bet.option == self.market.winner.unwrap();
        if is_winner {
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
                to: self.receiver.to_account_info(),
            };
            let ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                accounts,
                signer_seeds,
            );
            transfer(ctx, winner_share_amount)?;
        }
        let tokens_to_send = calculate_tokens_to_send(
            self.bet.amount, 
            self.market.total_mc,
            self.market.total_tokens,
            is_winner
        );

        let accounts = TransferChecked {
            from: self.token_reserve.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.receiver_ata.to_account_info(),
            authority: self.market.to_account_info(),
        };
        let binding = self.market.market_id.to_le_bytes();
        let seeds = &[
            b"market",
            self.signer.to_account_info().key.as_ref(),
            binding.as_ref(),
            &[bumps.market],
        ];
        let signer = [&seeds[..]];
        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            &signer,
        );
        transfer_checked(ctx, tokens_to_send, self.mint.decimals)?;

        self.market.total_tokens -= tokens_to_send;
        
        msg!("Tokens allocated: {}", tokens_to_send);
        Ok(())
    }
}
