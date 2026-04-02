#![allow(unexpected_cfgs)]

use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::{
    instructions::{CastVoteV1, CreateProposalV1, DaoInstruction, ExecuteProposalV1, InitDaoV1},
    utils::InstructionProcessor,
};

mod instructions;
mod states;
mod utils;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = DaoInstruction::try_from_slice(instruction_data)?;

    match instruction {
        DaoInstruction::InitDaoV1(data) => {
            InitDaoV1::try_from((accounts, data, program_id))?.process()
        }
        DaoInstruction::CreateProposalV1(data) => {
            CreateProposalV1::try_from((accounts, data, program_id))?.process()
        }
        DaoInstruction::CastVoteV1 => CastVoteV1::try_from((accounts, program_id))?.process(),
        DaoInstruction::ExecuteProposalV1 => {
            ExecuteProposalV1::try_from((accounts, program_id))?.process()
        }
    }
}
