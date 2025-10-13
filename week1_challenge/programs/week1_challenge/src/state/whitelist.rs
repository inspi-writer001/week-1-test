use anchor_lang::prelude::*;
// pub use shared_types::Whitelist;

#[account]
// #[derive(InitSpace)]can't use initspace since i'm using a dynamic vec
pub struct Whitelist {
    pub address: Vec<(Pubkey, u64)>,
    pub whitelist_bump: u8,
}

impl Whitelist {
    pub fn contains_address(&self, address: &Pubkey) -> bool {
        self.address.iter().any(|(addr, _)| addr == address)
    }
}
