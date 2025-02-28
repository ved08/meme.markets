use anchor_lang::{
    prelude::*,
    system_program::{create_account, transfer, CreateAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct RollDieInit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"1", seed.to_le_bytes().as_ref(), payer.key().as_ref()],
        bump
    )]
    pub vault_1: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"2", seed.to_le_bytes().as_ref(), payer.key().as_ref()],
        bump
    )]
    pub vault_2: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"3", seed.to_le_bytes().as_ref(), payer.key().as_ref()],
        bump
    )]
    pub vault_3: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"4", seed.to_le_bytes().as_ref(), payer.key().as_ref()],
        bump
    )]
    pub vault_4: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"5", seed.to_le_bytes().as_ref(), payer.key().as_ref()],
        bump
    )]
    pub vault_5: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"6", seed.to_le_bytes().as_ref(), payer.key().as_ref()],
        bump
    )]
    pub vault_6: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
impl<'info> RollDieInit<'info> {
    pub fn roll_die_accounts_init(&self) -> Result<()> {
        self.create_vault_account(self.vault_1.to_account_info())?;
        self.create_vault_account(self.vault_2.to_account_info())?;
        self.create_vault_account(self.vault_3.to_account_info())?;
        self.create_vault_account(self.vault_4.to_account_info())?;
        self.create_vault_account(self.vault_5.to_account_info())?;
        self.create_vault_account(self.vault_6.to_account_info())?;
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
