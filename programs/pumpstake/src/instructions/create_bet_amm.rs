use anchor_lang::{
    prelude::*,
    solana_program::native_token::LAMPORTS_PER_SOL,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};

use crate::{state::PredictionMarket, InitTokenParams, AMM};

#[derive(Accounts)]
#[instruction(params: InitTokenParams)]
pub struct InitalizeAmm<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [b"market", signer.key().as_ref(), market.market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, PredictionMarket>,
    #[account(
        init,
        seeds = [b"mint", market.key().as_ref()],
        bump,
        payer = signer,
        mint::decimals = params.decimals,
        mint::authority = mint
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    // #[account(
    //     seeds = [
    //         b"metadata",
    //         token_metadata_program.key().as_ref(),
    //         mint.key().as_ref()
    //     ],
    //     seeds::program = token_metadata_program.key(),
    //     bump
    // )]
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [b"amm", market.key().as_ref()],
        space = 8 + AMM::INIT_SPACE,
        bump,
    )]
    pub amm: Box<Account<'info, AMM>>,

    #[account(
      init,
      payer = signer,
      associated_token::mint = mint,
      associated_token::authority = amm
    )]
    pub token_reserve: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [b"solVault", amm.key().as_ref()],
        bump
    )]
    pub sol_reserve: SystemAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_metadata_program: Program<'info, Metaplex>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
impl<'info> InitalizeAmm<'info> {
    pub fn create_mint(
        &mut self,
        metadata: &InitTokenParams,
        bumps: &InitalizeAmmBumps,
    ) -> Result<()> {
        let seeds = &[
            "mint".as_bytes(),
            self.market.to_account_info().key.as_ref(),
            // &self.amm.seed.to_le_bytes()[..],
            &[bumps.mint],
        ];
        let signer = [&seeds[..]];
        let mut uri_prefix = "https://green-cheap-galliform-997.mypinata.cloud/ipfs/".to_owned();
        uri_prefix.push_str(&metadata.uri);

        let token_data: DataV2 = DataV2 {
            name: metadata.name.clone(),
            symbol: metadata.symbol.clone(),
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

    pub fn mint_to_reserve(&self, bumps: &InitalizeAmmBumps) -> Result<()> {
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
        let amount = 1_000_000_000 * u64::pow(10, self.mint.decimals as u32);
        mint_to(ctx, amount)?;
        Ok(())
    }

    // TODO: Work on the state of AMM
    pub fn init_amm_state(
        &mut self,
        metadata: &InitTokenParams,
        bumps: &InitalizeAmmBumps,
    ) -> Result<()> {
        // Make sol reserve rent exempt
        let accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.sol_reserve.to_account_info(),
        };
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(0);
        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(ctx, lamports)?;

        let cp_ratio = match metadata.mint_cap.checked_mul(metadata.sol_cap) {
            Some(val) => val,
            None => 0,
        };
        // This is to ensure overflow error doesnt occur
        // require!(cp_ratio != 0, AMMErrors::CPRatioOverflowError);

        // emit!(AmmInitalized {
        //     mint: self.mint.key(),
        //     creator: self.signer.key(),
        //     cp_ratio,
        //     sol_cap: metadata.sol_cap.checked_mul(LAMPORTS_PER_SOL).unwrap() as u64,
        //     mint_cap: metadata
        //         .mint_cap
        //         .checked_mul(u64::pow(10, self.mint.decimals as u32))
        //         .unwrap()
        // });

        self.amm.set_inner(AMM {
            mint: self.mint.key(),

            uri: metadata.uri.clone(),
            creator: self.signer.to_account_info().key(),
            cp_ratio,
            sol_cap: metadata.sol_cap.checked_mul(LAMPORTS_PER_SOL).unwrap() as u64,
            mint_cap: metadata
                .mint_cap
                .checked_mul(u64::pow(10, self.mint.decimals as u32))
                .unwrap() as u64,
            sol_reserve_bump: bumps.sol_reserve,
            amm_bump: bumps.amm,
            mint_bump: bumps.mint,
            seed: metadata.seed,
        });
        Ok(())
    }
}
