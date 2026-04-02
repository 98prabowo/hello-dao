use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Executed,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct ProposalV1 {
    pub author: Pubkey,           // Proposal author
    pub target_recipient: Pubkey, // Fund recipient if proposal executed
    pub amount: u64,              // Fund requested
    pub current_vote: u64,        // Total votes collected
    pub status: ProposalStatus,   // Proposal lifecycle
}

impl ProposalV1 {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 1;
    pub const SEED: &[u8; 11] = b"proposal_v1";
}
