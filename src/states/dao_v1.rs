use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct DaoV1 {
    pub admin: Pubkey,       // Authority to change config
    pub vote_threshold: u64, // Vote minimum before proposal execution
}

impl DaoV1 {
    pub const LEN: usize = 32 + 8;
    pub const SEED: &[u8; 6] = b"dao_v1";
}
