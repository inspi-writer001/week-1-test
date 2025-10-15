
use anchor_lang::{prelude::*, solana_program::program::invoke};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{ Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}
};
use spl_token_2022::{onchain,instruction as token_instruction};


use crate::{error::VaultError, Vault, Whitelist};
pub const VAULT_SEED: &[u8] = b"vault";

#[derive(Accounts)]
pub struct DepositWithdraw<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    /// CHECK: owner is admin of platform
    #[account(mut)]
    pub owner: UncheckedAccount<'info>,

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
    /// CHECK: Account containing the extra account
    pub extra_account_meta_list: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> DepositWithdraw<'info> {
    pub fn deposit(&mut self, amount: u64, remaining_accounts: &[AccountInfo<'info>]) -> Result<()> {

        require!(self.user_ata.amount >= amount,VaultError::InsufficientBalance);

          // 1. Manually build the `transfer_checked` instruction provided by the SPL Token program.
        let mut transfer_ix = token_instruction::transfer_checked(
            &self.token_program.key(),
            &self.user_ata.key(),
            &self.mint.key(),
            &self.vault_ata.key(),
            &self.sender.key(),
            &[], // No multisig signers are needed.
            amount,
            self.mint.decimals,
        )?;

        // 2. Manually add the extra accounts required by the transfer hook.
        // The Token 2022 program expects these to follow the core transfer accounts.
        transfer_ix.accounts.push(AccountMeta::new_readonly(self.extra_account_meta_list.key(), false));
        transfer_ix.accounts.push(AccountMeta::new(self.whitelist.key(), false));
        

        // 3. Create a flat list of all AccountInfos that the instruction needs.
        // This includes all accounts for the core transfer and the hook.
        let account_infos = &[
            self.user_ata.to_account_info(),
            self.mint.to_account_info(),
            self.vault_ata.to_account_info(),
            self.sender.to_account_info(),
            self.token_program.to_account_info(), // The Token Program must be in this list for `invoke`
            self.extra_account_meta_list.to_account_info(),
            self.whitelist.to_account_info(),
            self.hook_program_id.to_account_info(),
        ];

        // 4. Use the low-level `invoke` function to execute the CPI.
        invoke(&transfer_ix, account_infos)?;

        // let  account_infos = vec![
        // // self.user_ata.to_account_info(),          // source
        // // self.mint.to_account_info(),               // mint
        // // self.vault_ata.to_account_info(),          // destination
        // // self.sender.to_account_info(),             // authority
        // self.extra_account_meta_list.to_account_info(),
        // self.whitelist.to_account_info()
        // ];

    // onchain::invoke_transfer_checked(
    //     &self.token_program.key(),
    //     self.user_ata.to_account_info(),
    //     self.mint.to_account_info(),
    //     self.vault_ata.to_account_info(),
    //     self.sender.to_account_info(),
    //     &remaining_accounts.to_vec(),
    //     amount,
    //     self.mint.decimals,
    //     &[],  // No signer seeds needed here
    // )?;

        // let transfer_accounts = TransferChecked{
        //      authority: self.sender.to_account_info(),
        //      from: self.user_ata.to_account_info(),
        //      mint: self.mint.to_account_info(),
        //      to: self.vault_ata.to_account_info()
        // };

        
        // let  transfer_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts).with_remaining_accounts(account_infos.to_vec());

        

        // transfer_checked(transfer_ctx, amount, self.mint.decimals)?;
        let user_key = self.user_ata.key();
    
    // Try to find existing entry
    if let Some(entry) = self.whitelist.address.iter_mut().find(|(addr, _, _)| *addr == user_key) {
        // User exists, add to their existing amount
        entry.1 = entry.1.checked_add(amount)
            .ok_or(VaultError::AdditionAtUpdateUserOverflow)?;
    } 
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64, remaining_accounts: &[AccountInfo<'info>]) -> Result<()> {

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

         let extra_added_accounts = CpiContext::with_remaining_accounts(transfer_context, remaining_accounts.to_vec());

        transfer_checked(extra_added_accounts, amount, self.mint.decimals)?;

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
