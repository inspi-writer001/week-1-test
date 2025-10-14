#[cfg(test)]
mod tests {

    use {
        anchor_lang::{
            solana_program::{
                hash::Hash, native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey,
            },
            system_program::ID as SYSTEM_PROGRAM_ID,
            AccountDeserialize, InstructionData, ToAccountMetas,
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

        // create mint [mint_token]

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

        let tx = svm.send_transaction(transaction).unwrap();

        // initialize transfer hook

        // create vault

        // add to whitelist

        // deposit

        // remove from whitelist

        // withdraw
    }

    #[test]
    pub fn say_name() {}
}
