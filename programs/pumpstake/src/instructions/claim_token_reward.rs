use anchor_lang::prelude::*;

use crate::{state::{Bet, PredictionMarket}, utils::{calculate_price, calculate_token_price}};

#[derive(Accounts)]
pub struct ClaimTokenReward<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut, 
        seeds = [b"market", signer.key().as_ref(), market.market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, PredictionMarket>,
    #[account(mut)]
    pub bet: Account<'info, Bet>,
}
impl<'info> ClaimTokenReward<'info> {
    pub fn claim_tokens(&mut self) -> Result<()> {
        
        let tokens_to_send = calculate_token_price(
            self.bet.amount, 
            self.market.total_mc,
            self.market.total_tokens,
        );
        self.market.total_tokens -= tokens_to_send;
        
        msg!("Tokens allocated: {}", tokens_to_send);
        Ok(())
    }
}
