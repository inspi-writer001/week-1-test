pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("73zUmjohyBGgv7JvqguwzENwrTensYbVk3WoZHMSrM2j");

#[program]
pub mod week1_challenge {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        create_vault::handler(ctx)
    }
}

// create_vault ->
// deposit
// withdraw
// transfer_hook

// logic to check will be in transfer_hook
// mint of the vault will have token extension
// Don't forget to mint the token directly in the program
