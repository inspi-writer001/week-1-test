pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

use spl_discriminator::SplDiscriminate;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::{
    ExecuteInstruction, InitializeExtraAccountMetaListInstruction,
};

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

    // deposit
    pub fn deposit(ctx: Context<DepositWithdraw>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }
    // withdraw
    pub fn withdraw(ctx: Context<DepositWithdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    #[instruction(discriminator = InitializeExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn initialize_transfer_hook(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {
        msg!("Initializing Transfer Hook...");

        // Get the extra account metas for the transfer hook
        let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()?;

        msg!("Extra Account Metas: {:?}", extra_account_metas);
        msg!("Extra Account Metas Length: {}", extra_account_metas.len());

        // initialize ExtraAccountMetaList account with extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }
}

// create_vault ->
// deposit
// withdraw
// transfer_hook

// logic to check will be in transfer_hook
// mint of the vault will have token extension
// Don't forget to mint the token directly in the program
