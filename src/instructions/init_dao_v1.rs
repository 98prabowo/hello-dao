use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

use crate::{
    states::{DaoV1, VaultV1},
    utils::InstructionProcessor,
};

pub struct InitDaoV1Accounts<'a, 'info> {
    pub admin: &'a AccountInfo<'info>,
    pub dao: &'a AccountInfo<'info>,
    pub vault: &'a AccountInfo<'info>,
    pub system_program: &'a AccountInfo<'info>,
}

impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for InitDaoV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        let [admin, dao, vault, system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !admin.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !admin.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if !dao.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if !vault.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if system_program.key != &solana_system_interface::program::ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(Self {
            admin,
            dao,
            vault,
            system_program,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct InitDaoV1InstructionData {
    pub vote_threshold: u64,
    pub vested_amount: u64,
}

pub struct InitDaoV1<'a, 'info> {
    pub accounts: InitDaoV1Accounts<'a, 'info>,
    pub instruction_data: InitDaoV1InstructionData,
    pub program_id: &'a Pubkey,
    pub dao_bump: u8,
    pub vault_bump: u8,
}

impl<'a, 'info>
    TryFrom<(
        &'a [AccountInfo<'info>],
        InitDaoV1InstructionData,
        &'a Pubkey,
    )> for InitDaoV1<'a, 'info>
{
    type Error = ProgramError;

    fn try_from(
        (accounts, instruction_data, program_id): (
            &'a [AccountInfo<'info>],
            InitDaoV1InstructionData,
            &'a Pubkey,
        ),
    ) -> Result<Self, Self::Error> {
        let accounts = InitDaoV1Accounts::try_from(accounts)?;

        let (dao_pda, dao_bump) = Pubkey::find_program_address(&[DaoV1::SEED], program_id);
        if dao_pda != *accounts.dao.key {
            msg!(
                "Invalid PDA: expected {}, got {}",
                dao_pda,
                accounts.dao.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        let (vault_pda, vault_bump) = Pubkey::find_program_address(&[VaultV1::SEED], program_id);
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
            dao_bump,
            vault_bump,
        })
    }
}

impl<'a, 'info> InitDaoV1<'a, 'info> {
    fn init_dao(&self) -> ProgramResult {
        let rent = Rent::get()?;
        let space = DaoV1::LEN;
        let lamports = rent.minimum_balance(space);

        let ix = solana_system_interface::instruction::create_account(
            self.accounts.admin.key,
            self.accounts.dao.key,
            lamports,
            space as u64,
            self.program_id,
        );

        invoke_signed(
            &ix,
            &[
                self.accounts.admin.clone(),
                self.accounts.dao.clone(),
                self.accounts.system_program.clone(),
            ],
            &[&[DaoV1::SEED, &[self.dao_bump]]],
        )?;

        let dao_data = DaoV1 {
            admin: *self.accounts.admin.key,
            vote_threshold: self.instruction_data.vote_threshold,
        };

        dao_data.serialize(&mut &mut self.accounts.dao.data.borrow_mut()[..])?;

        msg!("DAO Configured: {}", self.accounts.dao.key);
        Ok(())
    }

    fn init_vault(&self) -> ProgramResult {
        let rent = Rent::get()?;
        let space = VaultV1::LEN;
        let lamports = rent.minimum_balance(space) + self.instruction_data.vested_amount;

        let ix = solana_system_interface::instruction::create_account(
            self.accounts.admin.key,
            self.accounts.vault.key,
            lamports,
            space as u64,
            self.program_id,
        );

        invoke_signed(
            &ix,
            &[
                self.accounts.admin.clone(),
                self.accounts.vault.clone(),
                self.accounts.system_program.clone(),
            ],
            &[&[VaultV1::SEED, &[self.vault_bump]]],
        )?;

        let vault_data = VaultV1 {
            amount: self.instruction_data.vested_amount,
        };

        vault_data.serialize(&mut &mut self.accounts.vault.data.borrow_mut()[..])?;

        msg!("Vault Configured: {}", self.accounts.vault.key);
        Ok(())
    }
}

impl<'a, 'info> InstructionProcessor for InitDaoV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        self.init_dao()?;
        self.init_vault()
    }
}
