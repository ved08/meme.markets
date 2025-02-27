use anchor_lang::prelude::*;
mod instructions;
use instructions::*;
mod state;
use state::*;
declare_id!("5eYrZR3FJYiuoGG7YsjhZP97EPofN65zM4PeLtUW8ZL3");

#[program]
pub mod pumpstake {
    use crate::instruction::DistributeFund;

    use super::*;

    pub fn proxy_initialize(
        ctx: Context<ProxyInitialize>,
        init_amount_0: u64,
        init_amount_1: u64,
        open_time: u64,
    ) -> Result<()> {
        instructions::proxy_initialize(ctx, init_amount_0, init_amount_1, open_time)
    }
    pub fn create_prediction_market(
        ctx: Context<CreatePredictionMarket>,
        type_of_market: u8,
        details: PredictionMarket,
    ) -> Result<()> {
        Ok(())
    }
    pub fn buy(ctx: Context<CreatePredictionMarket>) -> Result<()> {
        todo!()
    }
    pub fn sell(ctx: Context<CreatePredictionMarket>) -> Result<()> {
        todo!()
    }
    pub fn distribute_fund(ctx: Context<CreatePredictionMarket>) -> Result<()> {
        todo!()
    }
}
