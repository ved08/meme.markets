use anchor_lang::prelude::*;

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
    pub system_program: Program<'info, System>,
}
impl<'info> CreatePredictionMarket<'info> {
    pub fn create_prediction_market(
        &mut self,
        seed: u64,
        total_options: u8,
        start_time: i64,
        end_time: i64,
        params: PredictionMarketParams,
        bumps: &CreatePredictionMarketBumps,
    ) -> Result<()> {
        require!(
            total_options <= MAX_OPTIONS,
            PumpstakeErrors::MaxOptionsExceeded
        );
        let mut options = Vec::with_capacity(total_options as usize);
        for i in 0..total_options {
            options.push(BettingOption {
                option_id: i,
                description: "this a option".to_owned(),
                liquidity: 0,
            });
        }
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
            winner: 0,
        });
        Ok(())
    }
}
