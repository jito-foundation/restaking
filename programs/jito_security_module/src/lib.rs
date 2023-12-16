use anchor_lang::prelude::*;

declare_id!("FqrKisaNRa1b2xF2FeJNDsNKPNQHsPcvAkcCn28SHnmg");

#[program]
pub mod jito_security_module {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
