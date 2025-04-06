use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::error::PumpstakeErrors;
use crate::state::BettingOption;
use crate::state::PredictionMarket;
use crate::state::PredictionMarketParams;
use crate::state::MAX_OPTIONS;
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct CreatePredictionMarket<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        seeds = [b"market", signer.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        space = 8 + PredictionMarket::INIT_SPACE,
    )]
    pub market: Account<'info, PredictionMarket>,
    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump
    )]
    pub market_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> CreatePredictionMarket<'info> {
    pub fn create_prediction_market(
        &mut self,
        seed: u64,
        total_options: u8,
        duration: i64,
        params: PredictionMarketParams,
        options: Vec<BettingOption>,
        bumps: &CreatePredictionMarketBumps,
    ) -> Result<()> {
        require!(
            total_options <= MAX_OPTIONS,
            PumpstakeErrors::MaxOptionsExceeded
        );
        let clock = Clock::get().unwrap();
        let start_time = clock.unix_timestamp * 1000;
        let end_time = start_time.checked_add(duration).unwrap();

        let rent = Rent::get()?;
        let amount = rent.minimum_balance(0);
        let accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.market_vault.to_account_info(),
        };
        let cpi = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(cpi, amount)?;

        // let mut options = Vec::with_capacity(total_options as usize);
        // for i in 0..total_options {
        //     options.push(BettingOption {
        //         option_id: i,
        //         name: "",
        //         image: "",
        //         description: "this a option".to_owned(),
        //         liquidity: 0,
        //     });
        // }
        self.market.set_inner(PredictionMarket {
            market_id: seed,
            bump: bumps.market,
            owner: self.signer.to_account_info().key(),
            market_options: options,
            start_time,
            end_time,
            is_active: true,
            data: params,
            total_mc: 0,
            winner: None,
            graduate: None,
            total_tokens: 80_000_000_000_000, //total supply is this
        });
        Ok(())
    }
}
