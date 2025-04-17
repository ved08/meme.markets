use anchor_lang::prelude::*;
mod instructions;
use instructions::*;
mod state;
use state::*;
mod error;
mod utils;
declare_id!("HUq6sXsfmNeGj3mfcaeSohyhBoVB29u5QMsUWs4bEgpj");

#[program]
pub mod pumpstake {

    use super::*;

    pub fn create_prediction_market(
        ctx: Context<CreatePredictionMarket>,
        seed: u64,
        total_options: u8,
        duration: i64,
        details: PredictionMarketParams,
        option_details: Vec<BettingOption>,
    ) -> Result<()> {
        ctx.accounts.create_prediction_market(
            seed,
            total_options,
            duration,
            details,
            option_details,
            &ctx.bumps,
        )?;
        Ok(())
    }
    pub fn stake(ctx: Context<Stake>, bet_id: u64, option: u8, amount: u64) -> Result<()> {
        ctx.accounts.place_bet(bet_id, option, amount)
    }
    pub fn resolve_market(ctx: Context<ResolveMarket>, option: u8) -> Result<()> {
        ctx.accounts.resolve_winner(option)?;
        Ok(())
    }
    pub fn create_coin(ctx: Context<CreateCoin>) -> Result<()> {
        ctx.accounts.create_mint(&ctx.bumps)?;
        ctx.accounts.mint_to_reserve(&ctx.bumps)?;
        Ok(())
    }
    pub fn claim(ctx: Context<ClaimReward>) -> Result<()> {
        ctx.accounts.claim_reward(&ctx.bumps)?;
        Ok(())
    }
    pub fn claim2(ctx: Context<ClaimTokenReward>) -> Result<()> {
        ctx.accounts.claim_tokens(&ctx.bumps)?;
        Ok(())
    }
    pub fn transfer_tokens_to_creator(ctx: Context<TransferTokensToCreator>) -> Result<()> {
        ctx.accounts.transfer_tokens_to_creator_ata(&ctx.bumps)?;
        Ok(())
    }
    pub fn proxy_initialize(
        ctx: Context<ProxyInitialize>,
        init_amount_0: u64,
        init_amount_1: u64,
        open_time: u64,
    ) -> Result<()> {
        instructions::proxy_initialize(ctx, init_amount_0, init_amount_1, open_time)
    }
}
