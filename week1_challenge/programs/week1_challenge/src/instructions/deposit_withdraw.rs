
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{ Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};

use crate::{error::VaultError, Vault, Whitelist};
pub const VAULT_SEED: &[u8] = b"vault";

#[derive(Accounts)]
pub struct DepositWithdraw<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(mut)]
    pub owner: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = mint.mint_authority == Some(owner.key()).into() @VaultError::WrongMintAuthority,
        extensions::transfer_hook::program_id = hook_program_id.key(), 
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Program id of the tf hook
    pub hook_program_id: UncheckedAccount<'info>,

    #[account(
        mut @VaultError::VaultNotCreatedByAdmin,
        seeds = [mint.key().as_ref(),VAULT_SEED],
        bump = vault_state.vault_bump
    )]
    pub vault_state: Account<'info, Vault>,

    #[account(
        mut,
        constraint = vault_ata.mint == mint.key() @VaultError::WrongMint        
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_ata.mint == mint.key() @VaultError::WrongMint        
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

     #[account(
        mut,
        seeds = [b"whitelist"],
        bump = whitelist.whitelist_bump,
    )]
    pub whitelist: Account<'info, Whitelist>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> DepositWithdraw<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {

        require!(self.user_ata.amount >= amount,VaultError::InsufficientBalance);

        let transfer_accounts = TransferChecked{
             authority: self.sender.to_account_info(),
             from: self.user_ata.to_account_info(),
             mint: self.mint.to_account_info(),
             to: self.vault_ata.to_account_info()
        };

        let transfer_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        transfer_checked(transfer_ctx, amount, self.mint.decimals)?;



        // update user amount in vec
        let user_key = self.user_ata.key();
    
    // Try to find existing entry
    if let Some(entry) = self.whitelist.address.iter_mut().find(|(addr, _, _)| *addr == user_key) {
        // User exists, add to their existing amount
        entry.1 = entry.1.checked_add(amount)
            .ok_or(VaultError::AdditionAtUpdateUserOverflow)?;
    } 
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {

        // check that user exists in map
        let is_user_exist = self.whitelist.contains_address(&self.user_ata.key());
        require!(is_user_exist, VaultError::UserNotExistInVecForReal);

        // check that user withdrawable amount is sufficient
        let user_amount = self.whitelist.address.iter().find(|(addr, _, _)| self.user_ata.key() == *addr).map(|(_, amount, _)| amount);
        require!(user_amount >= Some(&amount), VaultError::InsufficientBalance);


        // let vault_seed: &[&[&[u8]]] = &[
        //         &[self.mint.key().as_ref()],
        //         &[VAULT_SEED],
        //         &[&self.vault_state.vault_bump.to_le_bytes()]
        // ];

        let token_mint = &self.mint.key();

        let new_vault_seed: &[&[&[u8]]] = &[&[
            token_mint.as_ref(),
            &VAULT_SEED,
            &self.vault_state.vault_bump.to_le_bytes()
        ]];


        let transfer_accounts = TransferChecked {
            from: self.vault_ata.to_account_info(),
            authority: self.vault_state.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.user_ata.to_account_info()
        };

        let transfer_context = CpiContext::new_with_signer(self.token_program.to_account_info(), transfer_accounts, new_vault_seed);

        transfer_checked(transfer_context, amount, self.mint.decimals)?;

        // deduct amount from user vec vault balance

        let user_key = self.user_ata.key();
    if let Some(pos) = self.whitelist.address.iter().position(|(addr, _, _)| *addr == user_key) {
        let (_, user_balance, _) = &mut self.whitelist.address[pos];
        *user_balance = user_balance.checked_sub(amount)
            .ok_or(VaultError::SubtractionAtUpdateUserUnderflow)?;
        
        // Optionally remove entry if balance is 0
        if *user_balance == 0 {
            self.whitelist.address.remove(pos);
        }
    }

        Ok(())
    }

    // pub fn deposit(&mut self, amount: u64)
}

//   #[account(
//         mut,
//         associated_token::authority =
//     )]
//     pub owner_ata: InterfaceAccount<'info, TokenAccount>,
