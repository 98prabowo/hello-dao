use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct VaultV1 {
    pub amount: u64, // Vested amount
}

impl VaultV1 {
    pub const LEN: usize = 8;
    pub const SEED: &[u8; 8] = b"vault_v1";
}
