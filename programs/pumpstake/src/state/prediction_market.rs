use anchor_lang::prelude::*;
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct PredictionMarket {
    ticker: String,
    name: String,
    image: String,
    description: String,
    twitter: String,
    website: String,
    telegram: String,
}
