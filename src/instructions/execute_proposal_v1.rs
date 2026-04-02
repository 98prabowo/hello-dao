use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    states::{DaoV1, ProposalStatus, ProposalV1, VaultV1},
    utils::InstructionProcessor,
};

pub struct ExecuteProposalV1Accounts<'a, 'info> {
    pub executor: &'a AccountInfo<'info>,
    pub proposal: &'a AccountInfo<'info>,
    pub vault: &'a AccountInfo<'info>,
    pub target_recipient: &'a AccountInfo<'info>,
    pub dao: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for ExecuteProposalV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [executor, proposal, vault, target_recipient, dao] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !executor.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !executor.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if !proposal.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if !vault.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if !target_recipient.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            executor,
            proposal,
            vault,
            target_recipient,
            dao,
        })
    }
}

pub struct ExecuteProposalV1<'a, 'info> {
    pub accounts: ExecuteProposalV1Accounts<'a, 'info>,
    pub program_id: &'a Pubkey,
}

impl<'a, 'info> TryFrom<(&'a [AccountInfo<'info>], &'a Pubkey)> for ExecuteProposalV1<'a, 'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, program_id): (&'a [AccountInfo<'info>], &'a Pubkey),
    ) -> Result<Self, Self::Error> {
        let accounts = ExecuteProposalV1Accounts::try_from(accounts)?;

        let (dao_pda, _) = Pubkey::find_program_address(&[DaoV1::SEED], program_id);
        if dao_pda != *accounts.dao.key {
            msg!(
                "Invalid PDA: expected {}, got {}",
                dao_pda,
                accounts.dao.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        let (vault_pda, _) = Pubkey::find_program_address(&[VaultV1::SEED], program_id);
        if vault_pda != *accounts.vault.key {
            msg!(
                "Invalid PDA: expected {}, got {}",
                vault_pda,
                accounts.vault.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            accounts,
            program_id,
        })
    }
}

impl<'a, 'info> ExecuteProposalV1<'a, 'info> {
    fn execute_proposal(&self) -> ProgramResult {
        let mut proposal_data = ProposalV1::try_from_slice(&self.accounts.proposal.data.borrow())?;
        let mut vault_data = VaultV1::try_from_slice(&self.accounts.vault.data.borrow())?;

        if proposal_data.status != ProposalStatus::Passed {
            msg!("Error: Proposal is not in Passed status.");
            return Err(ProgramError::InvalidAccountData);
        }

        if proposal_data.target_recipient != *self.accounts.target_recipient.key {
            msg!("Error: Target recipient mismatch.");
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

        let amount_to_send = proposal_data.amount;
        if **self.accounts.vault.lamports.borrow() < amount_to_send {
            return Err(ProgramError::InsufficientFunds);
        }

        **self.accounts.vault.lamports.borrow_mut() -= amount_to_send;
        **self.accounts.target_recipient.lamports.borrow_mut() += amount_to_send;

        proposal_data.status = ProposalStatus::Executed;
        vault_data.amount = vault_data
            .amount
            .checked_sub(amount_to_send)
            .ok_or(ProgramError::InsufficientFunds)?;

        proposal_data.serialize(&mut &mut self.accounts.proposal.data.borrow_mut()[..])?;
        vault_data.serialize(&mut &mut self.accounts.vault.data.borrow_mut()[..])?;

        msg!(
            "Proposal Executed by {}! {} lamports sent to {}",
            self.accounts.executor.key,
            proposal_data.amount,
            self.accounts.target_recipient.key
        );
        Ok(())
    }
}

impl<'a, 'info> InstructionProcessor for ExecuteProposalV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        self.execute_proposal()
    }
}
