use anchor_lang::prelude::*;

declare_id!("6Qs48uxeHV4ZaJeQsGczXNj2kbJSYFVyXs34QvXYPN5E");

#[program]
pub mod global_counter_avs {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
