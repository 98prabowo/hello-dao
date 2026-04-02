use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

use crate::{
    states::{ProposalStatus, ProposalV1, VaultV1},
    utils::InstructionProcessor,
};

pub struct CreateProposalV1Accounts<'a, 'info> {
    pub author: &'a AccountInfo<'info>,
    pub proposal: &'a AccountInfo<'info>,
    pub vault: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for CreateProposalV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [author, proposal, vault, system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !author.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !author.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if !proposal.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if system_program.key != &solana_system_interface::program::ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(Self {
            author,
            proposal,
            vault,
            system_program,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CreateProposalV1InstructionData {
    pub target_recipient: Pubkey,
    pub amount: u64,
}

pub struct CreateProposalV1<'a, 'info> {
    pub accounts: CreateProposalV1Accounts<'a, 'info>,
    pub instruction_data: CreateProposalV1InstructionData,
    pub program_id: &'a Pubkey,
    pub proposal_bump: u8,
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        CreateProposalV1InstructionData,
        &'a Pubkey,
    )> for CreateProposalV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            CreateProposalV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = CreateProposalV1Accounts::try_from(accounts)?;

        let (proposal_pda, proposal_bump) = Pubkey::find_program_address(
            &[
                ProposalV1::SEED,
                accounts.author.key.as_ref(),
                instruction_data.target_recipient.as_ref(),
            ],
            program_id,
        );
        if proposal_pda != *accounts.proposal.key {
            msg!(
                "Invalid PDA: expected {}, got {}",
                proposal_pda,
                accounts.proposal.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        let (vault_pda, _vault_bump) = Pubkey::find_program_address(&[VaultV1::SEED], program_id);
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
            instruction_data,
            program_id,
            proposal_bump,
        })
    }
}

impl<'a, 'info> CreateProposalV1<'a, 'info> {
    fn validate_budget(&self) -> ProgramResult {
        let vault_data = VaultV1::try_from_slice(&self.accounts.vault.data.borrow())?;

        if self.instruction_data.amount > vault_data.amount {
            msg!("Error: Proposal amount exceeds DAO budget cap!");
            return Err(ProgramError::InsufficientFunds);
        }

        Ok(())
    }

    fn init_proposal(&self) -> ProgramResult {
        let rent = Rent::get()?;
        let space = ProposalV1::LEN;
        let lamports = rent.minimum_balance(space);

        let ix = solana_system_interface::instruction::create_account(
            self.accounts.author.key,
            self.accounts.proposal.key,
            lamports,
            space as u64,
            self.program_id,
        );

        invoke_signed(
            &ix,
            &[
                self.accounts.author.clone(),
                self.accounts.proposal.clone(),
                self.accounts.system_program.clone(),
            ],
            &[&[
                ProposalV1::SEED,
                self.accounts.author.key.as_ref(),
                self.instruction_data.target_recipient.as_ref(),
                &[self.proposal_bump],
            ]],
        )?;

        let proposal_data = ProposalV1 {
            author: *self.accounts.author.key,
            target_recipient: self.instruction_data.target_recipient,
            amount: self.instruction_data.amount,
            current_vote: 0,
            status: ProposalStatus::Active,
        };

        proposal_data.serialize(&mut &mut self.accounts.proposal.data.borrow_mut()[..])?;

        msg!("DAO Configured: {}", self.accounts.proposal.key);
        Ok(())
    }
}

impl<'a, 'info> InstructionProcessor for CreateProposalV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        self.validate_budget()?;
        self.init_proposal()
    }
}
