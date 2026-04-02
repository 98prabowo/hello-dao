mod cast_vote_v1;
mod create_proposal_v1;
mod execute_proposal_v1;
mod init_dao_v1;

pub use cast_vote_v1::*;
pub use create_proposal_v1::*;
pub use execute_proposal_v1::*;
pub use init_dao_v1::*;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum DaoInstruction {
    InitDaoV1(InitDaoV1InstructionData),
    CreateProposalV1(CreateProposalV1InstructionData),
    CastVoteV1,
    ExecuteProposalV1,
}
