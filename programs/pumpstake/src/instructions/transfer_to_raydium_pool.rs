use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{sync_native, SyncNative},
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{error::PumpstakeErrors, events::TokenDataForRaydium, state::PredictionMarket};

#[derive(Accounts)]
pub struct TransferTokensToCreator<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    ///CHECK: This should be the market creator account
    #[account(mut)]
    pub market_creator: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"market", market.owner.as_ref(), market.market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Box<Account<'info, PredictionMarket>>,
    // #[account(
    //     seeds = [b"mint", market.key().as_ref()],
    //     bump,
    //     mint::authority = mint,
    //     mint::decimals = 6
    // )]
    ///CHECK:
    pub mint: UncheckedAccount<'info>,
    /// CHECK: this will be sol native mint id
    pub wsol_mint: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = market,
        associated_token::token_program = token1_program
      )]
    pub token_reserve: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump
    )]
    pub market_vault: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token1_program,
    )]
    pub creator_token_account: InterfaceAccount<'info, TokenAccount>,
    ///CHECK:
    // pub creator_token_account: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = wsol_mint,
        associated_token::authority = signer,
        associated_token::token_program = token0_program,

    )]
    pub creator_wsol_account: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token0_program: Interface<'info, TokenInterface>,
    pub token1_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> TransferTokensToCreator<'info> {
    pub fn transfer_tokens_to_creator_ata(
        &mut self,
        bumps: &TransferTokensToCreatorBumps,
    ) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            self.market.to_account_info().key.as_ref(),
            &[bumps.market_vault],
        ]];
        let accounts = Transfer {
            from: self.market_vault.to_account_info(),
            to: self.creator_wsol_account.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        let balance = self.market_vault.lamports();
        transfer(ctx, balance)?;

        let ctx = CpiContext::new(
            self.token0_program.to_account_info(),
            SyncNative {
                account: self.creator_wsol_account.to_account_info(),
            },
        );
        sync_native(ctx)?;

        let accounts = TransferChecked {
            from: self.token_reserve.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.creator_token_account.to_account_info(),
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
        let ctx =
            CpiContext::new_with_signer(self.token1_program.to_account_info(), accounts, &signer);
        let tokens_to_send = self.token_reserve.amount;
        transfer_checked(ctx, tokens_to_send, 6)?;
        emit!(TokenDataForRaydium {
            wsol_amount: balance,
            token_amount: tokens_to_send,
            mint: self.mint.to_account_info().key(),
            market_owner: self.market.owner
        });
        Ok(())
    }
}
