use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{DaoV1, ProposalStatus, ProposalV1},
    utils::InstructionProcessor,
};

pub struct CastVoteV1Accounts<'a, 'info> {
    pub voter: &'a AccountInfo<'info>,
    pub proposal: &'a AccountInfo<'info>,
    pub dao: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for CastVoteV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [voter, proposal, dao] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !voter.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !proposal.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            voter,
            proposal,
            dao,
        })
    }
}

pub struct CastVoteV1<'a, 'info> {
    pub accounts: CastVoteV1Accounts<'a, 'info>,
    pub program_id: &'a Pubkey,
}

impl<'a, 'info> TryFrom<(&'a [AccountInfo<'info>], &'a Pubkey)> for CastVoteV1<'a, 'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, program_id): (&'a [AccountInfo<'info>], &'a Pubkey),
    ) -> Result<Self, Self::Error> {
        let accounts = CastVoteV1Accounts::try_from(accounts)?;

        let (dao_pda, _dao_bump) = Pubkey::find_program_address(&[DaoV1::SEED], program_id);
        if dao_pda != *accounts.dao.key {
            msg!(
                "Invalid PDA: expected {}, got {}",
                dao_pda,
                accounts.dao.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            accounts,
            program_id,
        })
    }
}

impl<'a, 'info> CastVoteV1<'a, 'info> {
    fn vote(&self) -> ProgramResult {
        let mut proposal_data = ProposalV1::try_from_slice(&self.accounts.proposal.data.borrow())?;
        let dao_data = DaoV1::try_from_slice(&self.accounts.dao.data.borrow())?;

        if proposal_data.status != ProposalStatus::Active {
            msg!("Error: Voting is closed for this proposal.");
            return Err(ProgramError::InvalidAccountData);
        }

        let (proposal_pda, _proposal_bump) = Pubkey::find_program_address(
            &[
                ProposalV1::SEED,
                proposal_data.author.as_ref(),
                proposal_data.target_recipient.as_ref(),
            ],
            self.program_id,
        );
        if proposal_pda != *self.accounts.proposal.key {
            msg!(
                "Invalid PDA: expected {}, got {}",
                proposal_pda,
                self.accounts.proposal.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        let voting_power = self.accounts.voter.lamports();

        proposal_data.current_vote = proposal_data.current_vote.saturating_add(voting_power);

        msg!(
            "Voter casted {} power. Total votes: {}",
            voting_power,
            proposal_data.current_vote
        );

        if proposal_data.current_vote >= dao_data.vote_threshold {
            proposal_data.status = ProposalStatus::Passed;
            msg!("Proposal status updated to PASSED!");
        }

        proposal_data.serialize(&mut &mut self.accounts.proposal.data.borrow_mut()[..])?;

        Ok(())
    }
}

impl<'a, 'info> InstructionProcessor for CastVoteV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        self.vote()
    }
}
