use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreatePredictionMarket<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
}
