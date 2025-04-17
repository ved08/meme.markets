use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};

use crate::{error::PumpstakeErrors, state::PredictionMarket};

#[derive(Accounts)]
pub struct CreateCoin<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"market", market.owner.key().as_ref(), market.market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, PredictionMarket>,
    #[account(
        init,
        seeds = [b"mint", market.key().as_ref()],
        bump,
        payer = signer,
        mint::decimals = 6,
        mint::authority = mint
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = market
      )]
    pub token_reserve: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_metadata_program: Program<'info, Metaplex>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
impl<'info> CreateCoin<'info> {
    pub fn create_mint(&mut self, bumps: &CreateCoinBumps) -> Result<()> {
        require!(
            self.market.graduate.unwrap(),
            PumpstakeErrors::TokenCreationNotAllowed
        );
        require!(
            self.market.is_active == false,
            PumpstakeErrors::MarketActive
        );
        let seeds = &[
            "mint".as_bytes(),
            self.market.to_account_info().key.as_ref(),
            &[bumps.mint],
        ];
        let signer = [&seeds[..]];
        let winner_option = self
            .market
            .market_options
            .iter()
            .find(|opt| opt.option_id == self.market.winner.unwrap())
            .unwrap();
        let uri_prefix = winner_option.image.clone();

        let token_data: DataV2 = DataV2 {
            name: winner_option.name.clone(),
            symbol: winner_option.name.clone(),
            uri: uri_prefix,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };
        let accounts = CreateMetadataAccountsV3 {
            metadata: self.metadata.to_account_info(),
            mint: self.mint.to_account_info(),
            mint_authority: self.mint.to_account_info(),
            payer: self.signer.to_account_info(),
            update_authority: self.mint.to_account_info(),
            system_program: self.system_program.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        let metadata_ctx = CpiContext::new_with_signer(
            self.token_metadata_program.to_account_info(),
            accounts,
            &signer,
        );
        create_metadata_accounts_v3(metadata_ctx, token_data, false, true, None)?;
        msg!("Token mint created!");
        Ok(())
    }

    pub fn mint_to_reserve(&self, bumps: &CreateCoinBumps) -> Result<()> {
        require!(
            self.market.graduate.unwrap(),
            PumpstakeErrors::TokenCreationNotAllowed
        );
        let seeds = &[
            "mint".as_bytes(),
            self.market.to_account_info().key.as_ref(),
            &[bumps.mint],
        ];
        let signer = [&seeds[..]];
        let accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.token_reserve.to_account_info(),
            authority: self.mint.to_account_info(),
        };
        let ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, &signer);
        let amount = 80_000_000 * u64::pow(10, self.mint.decimals as u32);
        mint_to(ctx, amount)?;
        Ok(())
    }
}
