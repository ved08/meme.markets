use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct CoinTossInit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"head", market_id.to_le_bytes().as_ref(), payer.key().as_ref()],
        bump
    )]
    pub vault_1: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"tail", market_id.to_le_bytes().as_ref(), payer.key().as_ref()],
        bump
    )]
    pub vault_2: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
impl<'info> CoinTossInit<'info> {
    pub fn coin_toss_accounts_init(&self) -> Result<()> {
        self.create_vault_account(self.vault_1.to_account_info())?;
        self.create_vault_account(self.vault_2.to_account_info())?;
        Ok(())
    }
    fn create_vault_account(&self, new_account: AccountInfo<'info>) -> Result<()> {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(0);
        let cpi_accounts = Transfer {
            from: self.payer.to_account_info(),
            to: new_account,
        };
        let cpi_context = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);
        transfer(cpi_context, lamports)?;
        Ok(())
    }
}
