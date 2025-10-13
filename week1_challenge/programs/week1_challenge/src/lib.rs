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

    pub fn create_vault(ctx: Context<VaultOperation>) -> Result<()> {
        ctx.accounts.create_vault(&ctx.bumps)
    }

    pub fn mint_token(ctx: Context<TokenFactory>, amount: u64, decimals: u8) -> Result<()> {
        ctx.accounts.init_mint(amount, decimals)
    }

    pub fn add_to_whitelist(ctx: Context<WhitelistOperations>, address: Pubkey) -> Result<()> {
        ctx.accounts.add_to_whitelist(address)
    }

    pub fn remove_from_whitelist(ctx: Context<WhitelistOperations>, address: Pubkey) -> Result<()> {
        ctx.accounts.remove_from_whitelist(address)
    }
}

// create_vault ->
// deposit
// withdraw
// transfer_hook

// logic to check will be in transfer_hook
// mint of the vault will have token extension
// Don't forget to mint the token directly in the program
