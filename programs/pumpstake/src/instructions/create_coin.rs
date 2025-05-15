use std::{io::Seek, u64};

use anchor_lang::{prelude::*, solana_program::program::invoke};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id, AssociatedToken},
    metadata::{
        create_metadata_accounts_v3,
        mpl_token_metadata::{
            instructions::CreateV1CpiBuilder,
            types::{DataV2, TokenStandard},
        },
        CreateMetadataAccountsV3, Metadata as Metaplex,
    },
    token_2022::{
        initialize_mint2,
        spl_token_2022::{
            self,
            extension::{
                transfer_fee::{
                    self, instruction::initialize_transfer_fee_config, TransferFeeConfig,
                },
                BaseStateWithExtensions, ExtensionType,
            },
        },
        InitializeMint2, SetAuthority,
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
    // #[account(
    //     init,
    //     space = 300,
    //     payer = signer,
    //     seeds = [b"mint", market.key().as_ref()],
    //     bump,
    // )]
    // ///CHECK: THIS IS SAFE WE ARE MANUALLY INITALIZING ACCOUNT
    // pub mint: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub mint: Signer<'info>,
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    // #[account(
    //     init,
    //     payer = signer,
    //     associated_token::mint = mint,
    //     associated_token::authority = market
    //   )]
    // pub token_reserve: InterfaceAccount<'info, TokenAccount>,
    ///CHECK: this is revenue wallet
    #[account(mut)]
    pub revenue: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK:
    pub token_reserve: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_metadata_program: Program<'info, Metaplex>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    ///CHECK:
    pub sysvar_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> CreateCoin<'info> {
    pub fn create_mint(&mut self, bumps: &CreateCoinBumps) -> Result<()> {
        require!(self.market.winner_present, PumpstakeErrors::RefundOnly);
        require!(
            self.market.graduate.unwrap(),
            PumpstakeErrors::TokenCreationNotAllowed
        );
        require!(
            self.market.is_active == false,
            PumpstakeErrors::MarketActive
        );
        let extensions = [ExtensionType::TransferFeeConfig];
        let space: usize =
            ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&extensions)?;
        let lamports = self.rent.minimum_balance(space);
        let create_account_ix = anchor_lang::solana_program::system_instruction::create_account(
            &self.signer.key(),
            &self.mint.key(),
            lamports,
            space as u64,
            &spl_token_2022::ID,
        );
        invoke(
            &create_account_ix,
            &[
                self.signer.to_account_info(),
                self.mint.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        // let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        let ix = initialize_transfer_fee_config(
            self.token_program.key,
            self.mint.key,
            Some(self.revenue.key),
            Some(self.revenue.key),
            200,
            u64::MAX,
        )?;
        invoke(
            &ix,
            &[
                self.signer.to_account_info(),
                self.mint.to_account_info(),
                self.revenue.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;
        let cpi_accounts = InitializeMint2 {
            mint: self.mint.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        initialize_mint2(cpi_ctx, 6, self.mint.to_account_info().key, None)?;
        // let seeds = &[
        //     "mint".as_bytes(),
        //     self.market.to_account_info().key.as_ref(),
        //     &[bumps.mint],
        // ];
        // let signer = [&seeds[..]];

        // let mint2_ctx = CpiContext::new_with_signer(
        //     self.token_program.to_account_info(),
        //     InitializeMint2 {
        //         mint: self.mint.to_account_info(),
        //     },
        //     &signer,
        // );
        // initialize_mint2(mint2_ctx, 6, &self.mint.key(), None)?;

        // let tranfer_fee_ix = initialize_transfer_fee_config(
        //     self.token_program.key,
        //     &self.mint.key(),
        //     Some(&self.mint.key()),
        //     Some(&self.mint.key()),
        //     200,
        //     2_000_000_000,
        // )?;
        // anchor_lang::solana_program::program::invoke_signed(
        //     &tranfer_fee_ix,
        //     &[self.mint.to_account_info(), self.rent.to_account_info()],
        //     &signer,
        // )?;

        msg!("Token mint created!");
        Ok(())
    }

    pub fn mint_to_reserve(&self, bumps: &CreateCoinBumps) -> Result<()> {
        require!(
            self.market.graduate.unwrap(),
            PumpstakeErrors::TokenCreationNotAllowed
        );
        let ata = get_associated_token_address_with_program_id(
            self.market.to_account_info().key,
            self.mint.to_account_info().key,
            self.token_program.key,
        );
        require_keys_eq!(
            ata,
            self.token_reserve.key(),
            PumpstakeErrors::IncorrectAtaPassed
        );
        let cpi_accounts = associated_token::Create {
            payer: self.signer.to_account_info(),
            associated_token: self.token_reserve.to_account_info(),
            authority: self.market.to_account_info(),
            mint: self.mint.to_account_info(),
            system_program: self.system_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            self.associated_token_program.to_account_info(),
            cpi_accounts,
        );
        associated_token::create(cpi_ctx)?;
        let binding = self.market.owner.key();
        let binding2 = self.market.market_id.to_le_bytes();
        let seeds = &[
            "market".as_bytes(),
            binding.as_ref(),
            binding2.as_ref(),
            &[bumps.market],
        ];
        let signer = [&seeds[..]];
        let accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.token_reserve.to_account_info(),
            authority: self.mint.to_account_info(),
        };
        let ctx = CpiContext::new(self.token_program.to_account_info(), accounts);
        let amount = 1_000_000_000 * u64::pow(10, 6 as u32);
        mint_to(ctx, amount)?;
        CreateV1CpiBuilder::new(&self.token_metadata_program.to_account_info())
            .metadata(&self.metadata.to_account_info())
            .mint(&self.mint.to_account_info(), true)
            .authority(&self.mint.to_account_info())
            .payer(&self.signer.to_account_info())
            .update_authority(&self.mint.to_account_info(), true)
            .spl_token_program(Some(&self.token_program.to_account_info()))
            .token_standard(TokenStandard::Fungible)
            .seller_fee_basis_points(0)
            .name(self.market.data.name.clone())
            .symbol(self.market.data.ticker.clone())
            .uri(self.market.data.image.clone())
            .decimals(6)
            .system_program(&self.system_program.to_account_info())
            .sysvar_instructions(&self.sysvar_program.to_account_info())
            .invoke()?;
        // let token_data: DataV2 = DataV2 {
        //     name: self.market.data.name.clone(),
        //     symbol: self.market.data.ticker.clone(),
        //     uri: self.market.data.image.clone(),
        //     seller_fee_basis_points: 0,
        //     creators: None,
        //     collection: None,
        //     uses: None,
        // };
        // let accounts = CreateMetadataAccountsV3 {
        //     metadata: self.metadata.to_account_info(),
        //     mint: self.mint.to_account_info(),
        //     mint_authority: self.market.to_account_info(),
        //     payer: self.signer.to_account_info(),
        //     update_authority: self.market.to_account_info(),
        //     system_program: self.system_program.to_account_info(),
        //     rent: self.rent.to_account_info(),
        // };
        // let metadata_ctx = CpiContext::new_with_signer(
        //     self.token_metadata_program.to_account_info(),
        //     accounts,
        //     &signer,
        // );
        // create_metadata_accounts_v3(metadata_ctx, token_data, false, true, None)?;

        Ok(())
    }
}
