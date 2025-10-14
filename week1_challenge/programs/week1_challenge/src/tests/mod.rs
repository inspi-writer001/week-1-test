#[cfg(test)]
mod tests {

    use {
        anchor_lang::{
            error::Error,
            prelude::msg,
            solana_program::{
                hash::Hash, native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey,
            },
            system_program::ID as SYSTEM_PROGRAM_ID,
            AccountDeserialize, InstructionData, Key, ToAccountMetas,
        },
        anchor_spl::{
            associated_token::{self, spl_associated_token_account},
            token_2022::spl_token_2022,
        },
        litesvm::LiteSVM,
        solana_address::Address,
        solana_hash::Hash as SolanaHash,
        solana_instruction::Instruction,
        solana_keypair::Keypair,
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::{path::PathBuf, str::FromStr},
    };

    // trait PubkeyExt {
    //     fn to_address(&self) -> Address;
    //     fn to_pubkey(&self) -> Pubkey;
    // }

    // // Implement it for Pubkey
    // impl PubkeyExt for Pubkey {
    //     fn to_address(&self) -> Address {
    //         Address::from(self.to_bytes())
    //     }

    //     fn to_pubkey(&self) -> Pubkey {
    //         Pubkey::from(self.to_bytes())
    //     }
    // }

    // impl PubkeyExt for Address {
    //     fn to_address(&self) -> Address {
    //         *self
    //     }
    //     fn to_pubkey(&self) -> Pubkey {
    //         Pubkey::from(self.to_bytes())
    //     }
    // }

    use crate::ID as PROGRAM_ID;
    const TRANSFER_HOOK_PROGRAM_ID: Pubkey =
        Pubkey::from_str_const("Augb2132S5P1yXCYj7nNZTyksUhCA3k7G5z8SS3o8geh");

    pub struct ReusableData {
        ata_program: Pubkey,
        token_program: Pubkey,
        system_program: Pubkey,
        vault_ata: Pubkey,
        vault_state: Pubkey,
        mint: Keypair,
        admin: Keypair,
    }

    // pub fn pubkey_to_address(pubkey: &Pubkey) -> Address {
    //     Address::from(pubkey.to_bytes())
    // }

    pub fn setup() -> (LiteSVM, ReusableData) {
        let mut svm = LiteSVM::new();
        let admin = Keypair::new();

        svm.airdrop(&admin.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        // Load program SO file
        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/week1_challenge.so");

        let transfer_hook_so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../transfer_hook/target/deploy/transfer_hook.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
        let hook_program_data =
            std::fs::read(transfer_hook_so_path).expect("Failed to read hook program SO file");

        let program_id = PROGRAM_ID;
        let hook_program_id = TRANSFER_HOOK_PROGRAM_ID;
        let spl_program_id = spl_token_2022::ID;
        let ata_program_id = spl_associated_token_account::ID;
        let system_program_id = SYSTEM_PROGRAM_ID;

        let mint = Keypair::new();

        svm.add_program(hook_program_id, &hook_program_data);

        svm.add_program(program_id, &program_data);

        let vault_state =
            Pubkey::find_program_address(&[&mint.pubkey().to_bytes(), b"vault"], &program_id).0;

        let vault_ata = associated_token::get_associated_token_address_with_program_id(
            &vault_state,
            &mint.pubkey(),
            &spl_program_id,
        );

        let exported_state = ReusableData {
            admin: admin,
            ata_program: ata_program_id,
            system_program: system_program_id,
            token_program: spl_program_id,
            mint,
            vault_ata: vault_ata,
            vault_state,
        };

        (svm, exported_state)
    }

    #[test]
    pub fn modify_name() {
        let (mut svm, reusable_data) = setup();

        // 🔥 [1] create vault

        let create_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::VaultOperation {
                associated_token_program: reusable_data.ata_program,
                hook_program_id: TRANSFER_HOOK_PROGRAM_ID,
                mint: reusable_data.mint.pubkey(),
                owner: reusable_data.admin.pubkey(),
                system_program: reusable_data.system_program,
                token_program: reusable_data.token_program,
                vault_ata: reusable_data.vault_ata,
                vault_state: reusable_data.vault_state,
            }
            .to_account_metas(None),
            data: crate::instruction::CreateVault {}.data(),
        };

        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new_signed_with_payer(
            &[create_vault_ix],
            Some(&reusable_data.admin.pubkey()),
            &[&reusable_data.admin, &reusable_data.mint],
            recent_blockhash,
        );

        svm.send_transaction(transaction).unwrap();

        let new_state_of_vault = svm.get_account(&reusable_data.vault_state).unwrap();
        let fetched_vault_state =
            crate::state::Vault::try_deserialize(&mut new_state_of_vault.data.as_ref()).unwrap();

        assert_eq!(
            fetched_vault_state.mint.key(),
            reusable_data.mint.pubkey(),
            "Account wasn't set correctly in Vault"
        );

        assert_eq!(
            fetched_vault_state.owner.key(),
            reusable_data.admin.pubkey(),
            "Account wasn't set correctly in Vault"
        );

        // 🔥🔥 [2] initialize transfer hook

        let extra_account_meta_list = Pubkey::find_program_address(
            &[
                b"extra-account-metas",
                &reusable_data.mint.pubkey().as_ref(),
            ],
            &TRANSFER_HOOK_PROGRAM_ID,
        )
        .0;

        let initialize_transfer_hook_ix = Instruction {
            program_id: TRANSFER_HOOK_PROGRAM_ID,
            accounts: crate::accounts::InitializeExtraAccountMetaList {
                extra_account_meta_list,
                mint: reusable_data.mint.pubkey(),
                payer: reusable_data.admin.pubkey(),
                system_program: reusable_data.system_program.key(),
            }
            .to_account_metas(None),
            data: crate::instruction::InitializeTransferHook {}.data(),
        };

        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new_signed_with_payer(
            &[initialize_transfer_hook_ix],
            Some(&reusable_data.admin.pubkey()),
            &[&reusable_data.admin],
            recent_blockhash,
        );

        svm.send_transaction(transaction).unwrap();

        let new_state_of_metalist = svm.get_account(&reusable_data.vault_state).unwrap();
        // let fetched_metalist_state =
        //     crate::AccountInfo::try_borrow_data(&new_state_of_metalist).unwrap();

        msg!("MetaList state: {:?}", new_state_of_metalist.data);

        // 🔥🔥🔥🔥 [3] add to whitelist
        let whitelist = Pubkey::find_program_address(&[b"whitelist"], &PROGRAM_ID.key()).0;

        let new_user = Keypair::new();

        svm.airdrop(&new_user.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        let new_user_ata = associated_token::get_associated_token_address_with_program_id(
            &new_user.pubkey(),
            &reusable_data.mint.pubkey(),
            &reusable_data.token_program.key(),
        );

        let add_to_whitelist_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::WhitelistOperations {
                whitelist: whitelist.key(),
                vault: reusable_data.vault_state.key(),
                admin: reusable_data.admin.pubkey(),
                system_program: reusable_data.system_program.key(),
            }
            .to_account_metas(None),
            data: crate::instruction::AddToWhitelist {
                address: new_user_ata.key(),
                mint: reusable_data.mint.pubkey(),
            }
            .data(),
        };

        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new_signed_with_payer(
            &[add_to_whitelist_ix],
            Some(&reusable_data.admin.pubkey()),
            &[&reusable_data.admin],
            recent_blockhash,
        );

        svm.send_transaction(transaction).unwrap();

        let contains_address = crate::state::Whitelist::contains_address(
            &crate::state::Whitelist::try_deserialize(&mut whitelist.as_ref()).unwrap(),
            &new_user_ata,
        );

        msg!("contains you: {}", contains_address);

        // 🔥🔥🔥 [4] mint token to self

        // let new_user = Keypair::new();
        let admin_ata = associated_token::get_associated_token_address_with_program_id(
            &reusable_data.admin.pubkey(),
            &reusable_data.mint.pubkey(),
            &reusable_data.token_program.key(),
        );

        //  svm.airdrop(&admin.pubkey(), 10 * LAMPORTS_PER_SOL)
        //     .expect("Failed to airdrop SOL to payer");

        let mint_token_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::TokenFactory {
                extra_account_meta_list,
                mint: reusable_data.mint.pubkey(),
                associated_token_program: reusable_data.ata_program.key(),
                blocklist: whitelist,
                hook_program_id: TRANSFER_HOOK_PROGRAM_ID.key(),
                source_token_account: admin_ata.key(),
                system_program: reusable_data.system_program.key(),
                token_program: reusable_data.token_program.key(),
                user: reusable_data.admin.pubkey(),
            }
            .to_account_metas(None),
            data: crate::instruction::MintToken {
                amount: 10_000,
                decimals: 9,
            }
            .data(),
        };

        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new_signed_with_payer(
            &[mint_token_ix],
            Some(&reusable_data.admin.pubkey()),
            &[&reusable_data.admin, &reusable_data.mint],
            recent_blockhash,
        );

        svm.send_transaction(transaction).unwrap();

        let new_state_of_admin_ata = svm.get_account(&admin_ata).unwrap();

        // 1. Fetch the raw account data
        let new_state_of_admin_ata = svm.get_account(&admin_ata).unwrap();

        // 2. Get the data as a mutable slice (&[u8] or &mut [u8])
        let account_data_slice: &[u8] = new_state_of_admin_ata.data.as_ref();

        // 3. Use Pack::unpack (or TokenAccount::unpack, which calls the trait implementation)
        //    TokenAccount is from the spl_token_2022 crate.
        use anchor_spl::token_2022::spl_token_2022::state::Account as TokenAccountState;

        let fetched_admin_ata_state = TokenAccountState::unpack(account_data_slice).unwrap();

        // Example of checking the balance:
        msg!(
            "Admin ATA token balance: {}",
            fetched_admin_ata_state.amount
        );
        // let fetched_ata_state =
        //     crate::Account::try_from(&new_state_of_admin_ata.data.to_account_info()).unwrap();

        // msg!("New Admin Balance: {:?}", fetched_metalist_state.amount);

        // 🔥🔥🔥🔥🔥 [5] deposit

        // 🔥🔥🔥🔥🔥 [6] remove from whitelist

        // 🔥🔥🔥🔥🔥 [7] withdraw
    }

    #[test]
    pub fn say_name() {}
}
