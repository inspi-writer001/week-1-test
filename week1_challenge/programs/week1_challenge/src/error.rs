use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("You can't recreate a vault")]
    VaultAlreadyExist,
    #[msg("Wrong Admin Provided")]
    NotAdmin,
    #[msg("Cannot remove user, user does not exist")]
    UserNotExistInVec,
    #[msg("overflow at multiplication on mint")]
    MUltiplicationAtMint,
    #[msg("Admin should already have created this Vault")]
    VaultNotCreatedByAdmin,
}
