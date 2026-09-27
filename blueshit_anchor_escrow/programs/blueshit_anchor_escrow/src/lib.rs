use anchor_lang::prelude::*;

declare_id!("E3ezrKENZdZJd6jm6DQ8xz3ASJd33XvKhTnDaCGeixv1");

#[program]
pub mod blueshit_anchor_escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
