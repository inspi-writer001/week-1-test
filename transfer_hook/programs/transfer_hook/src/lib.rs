pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Augb2132S5P1yXCYj7nNZTyksUhCA3k7G5z8SS3o8geh");

#[program]
pub mod transfer_hook {
    use super::*;

    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_hook(amount)
    }
}
