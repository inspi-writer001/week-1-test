use anchor_lang::prelude::*;

// Dummy program ID - not actually used
declare_id!("11111111111111111111111111111111");

#[account]
// #[derive(InitSpace)]
pub struct Whitelist {
    pub address: Vec<(Pubkey, u64)>,
    pub whitelist_bump: u8,
}

impl Whitelist {
    pub fn contains_address(&self, address: &Pubkey) -> bool {
        self.address.iter().any(|(addr, _)| addr == address)
    }
}
