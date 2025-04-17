use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{sync_native, transfer_checked, SyncNative, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{error::PumpstakeErrors, state::PredictionMarket};

#[derive(Accounts)]
pub struct TransferTokensToCreator<'info> {
    ///CHECK: This should be the market creator account
    #[account(mut)]
    pub creator: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"market", market.owner.as_ref(), market.market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Box<Account<'info, PredictionMarket>>,
    #[account(
        seeds = [b"mint", market.key().as_ref()],
        bump,
        mint::authority = mint,
        mint::decimals = 6
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: this will be sol native mint id
    pub wsol_mint: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = market
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
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator,
        associated_token::token_program = token_program,
    )]
    pub creator_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = wsol_mint,
        associated_token::authority = creator,
    )]
    pub creator_wsol_account: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
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
            self.token_program.to_account_info(),
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
            self.creator.to_account_info().key.as_ref(),
            binding.as_ref(),
            &[bumps.market],
        ];
        let signer = [&seeds[..]];
        let ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, &signer);
        let tokens_to_send = self.token_reserve.amount;
        transfer_checked(ctx, tokens_to_send, self.mint.decimals)?;
        Ok(())
    }
}
