use anchor_lang::prelude::*;

use crate::state::PredictionMarket;
use crate::state::PredictionMarketParams;
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
        market_type: u8,
        start_time: i64,
        end_time: i64,
        params: PredictionMarketParams,
        bumps: &CreatePredictionMarketBumps,
    ) -> Result<()> {
        self.market.set_inner(PredictionMarket {
            market_id: seed,
            bump: bumps.market,
            owner: self.signer.to_account_info().key(),
            market_type,
            start_time,
            end_time,
            is_active: true,
            data: params,
            winner: Pubkey::default(),
        });
        Ok(())
    }
}
