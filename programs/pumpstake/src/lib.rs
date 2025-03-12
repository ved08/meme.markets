use anchor_lang::prelude::*;
mod instructions;
use instructions::*;
mod state;
use state::*;
mod error;
use error::*;
declare_id!("5eYrZR3FJYiuoGG7YsjhZP97EPofN65zM4PeLtUW8ZL3");

#[program]
pub mod pumpstake {

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
        seed: u64,
        market_type: u8,
        start_time: i64,
        end_time: i64,
        details: PredictionMarketParams,
    ) -> Result<()> {
        ctx.accounts.create_prediction_market(
            seed,
            market_type,
            start_time,
            end_time,
            details,
            &ctx.bumps,
        )?;
        Ok(())
    }
    pub fn roll_die_init_accounts(ctx: Context<RollDieInit>) -> Result<()> {
        ctx.accounts.roll_die_accounts_init()?;
        Ok(())
    }
    pub fn coin_toss_init_accounts(ctx: Context<CoinTossInit>) -> Result<()> {
        ctx.accounts.coin_toss_accounts_init()?;
        Ok(())
    }
    pub fn stake(ctx: Context<Stake>, option: u8, amount: u64) -> Result<()> {
        ctx.accounts.place_bet(option, amount)
    }
    pub fn sell(ctx: Context<CreatePredictionMarket>) -> Result<()> {
        todo!()
    }
    pub fn distribute_fund(ctx: Context<CreatePredictionMarket>) -> Result<()> {
        todo!()
    }
}
