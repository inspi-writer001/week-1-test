use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("You can't recreate a vault")]
    VaultAlreadyExist,
}
