use anchor_lang::prelude::*;

#[error_code]
pub enum PumpstakeErrors {
    #[msg("Market already expired. Cannot place bet")]
    MarketExpired,
    #[msg("Market is still active")]
    MarketActive,
    #[msg("Not authorized to execute the instruction")]
    NotAuthorized,
    #[msg("Number of options exceed max options")]
    MaxOptionsExceeded,
    #[msg("Bettor pubkey and reciever dont match")]
    IncorrectReceivor,
    #[msg("Not enough liquidity to graduate")]
    InsufficientLiquidityToGraduate,
    #[msg("Enough liquidity to graduate. Cannot refund")]
    SufficientLiquidityToGraduate,
    #[msg("cannot create token. market cannot graduate")]
    TokenCreationNotAllowed,
    #[msg("No tokens present in the vault")]
    NoFundsInVault,
    #[msg("Bet reward already claimed")]
    RewardAlreadyClaimed,
    #[msg("Winner already present. Cannot refund")]
    CannotRefund,
    #[msg("Cannot proceed. Please refund all the amounts")]
    RefundOnly,
    #[msg("Incorrect revenue wallet. Please provide correct one")]
    IncorrectRevenueWallet,
    #[msg("Incorrect ATA passed while creating coin")]
    IncorrectAtaPassed,
}
