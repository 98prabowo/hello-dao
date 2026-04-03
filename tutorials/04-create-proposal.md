# Instruction: Create Proposal

In this chapter, we will implement the logic that allows members to create a spending proposal. 
This instruction introduces **Composite Seeds** and **Pre-flight Validation** against existing program state.

## 1. The Instruction Data (The Input)

To create a proposal, the author needs to specify who gets the money and how much they should receive.

```rust
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CreateProposalV1InstructionData {
    pub target_recipient: Pubkey, // Who receives the SOL if passed?
    pub amount: u64,              // How much SOL (in lamports)?
}
```

## 2. Defining the Accounts Struct

We list the accounts required to initialize a new proposal record on-chain.

```rust
pub struct CreateProposalV1Accounts<'a, 'info> {
    pub author: &'a AccountInfo<'info>,         // The person creating the proposal
    pub proposal: &'a AccountInfo<'info>,       // The new PDA to be created
    pub vault: &'a AccountInfo<'info>,          // The DAO Treasury (for budget checks)
    pub system_program: &'a AccountInfo<'info>, // Required for account creation
}
```

### The Validation Logic (TryFrom)

We verify that the accounts provided are exactly what the program expects:

- **Signer Check:** Only the `author` can initiate a proposal from their wallet.
- **Writable Check:** The `author` pays the rent, and the `proposal` account is being initialized, so both must be writable.
- **Safety Check:** We ensure the `system_program` is the official one to prevent "fake" account creation calls.

```rust
impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for CreateProposalV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        // Check if accounts has correct len
        let [author, proposal, vault, system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Validate author account
        if !author.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !author.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate proposal account
        if !proposal.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate system program
        if system_program.key != &solana_system_interface::program::ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Initialize accounts
        Ok(Self {
            author,
            proposal,
            vault,
            system_program,
        })
    }
}
```

## 3. The Master Instruction Struct

This struct holds our validated accounts, the user's input, and the metadata required for signing.

```rust
pub struct CreateProposalV1<'a, 'info> {
    pub accounts: CreateProposalV1Accounts<'a, 'info>,
    pub instruction_data: CreateProposalV1InstructionData,
    pub program_id: &'a Pubkey,
    pub proposal_bump: u8, // Saved from PDA derivation
}
```

## 4. PDA Verification with Composite Seeds

Unlike the DAO or Vault, which use a single static string as a seed, the **Proposal PDA** is unique to the author and the recipient. 
This allows one person to have multiple proposals active for different targets.

Inside `TryFrom` for the master struct:

```rust
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
        // Deserialize accounts list
        let accounts = CreateProposalV1Accounts::try_from(accounts)?;

        // Validate proposal PDA
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

        // Validate vault PDA
        let (vault_pda, _vault_bump) = Pubkey::find_program_address(&[VaultV1::SEED], program_id);
        if vault_pda != *accounts.vault.key {
            msg!(
                "Invalid PDA: expected {}, got {}",
                vault_pda,
                accounts.vault.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        // Initialize master instruction struct
        Ok(Self {
            accounts,
            instruction_data,
            program_id,
            proposal_bump,
        })
    }
}
```

> [!IMPORTANT]
> By including the `author` and `target_recipient` in the seeds, we ensure that the address is **deterministic**. 
> If the same author tries to create a proposal for the same recipient twice, the address will conflict, naturally preventing duplicate spam.

## 5. The Execution Logic

### A. Budget Validation (Reading State)

Before spending money on account rent, we check if the DAO even has enough SOL in its Vault to fulfill the request.

```rust
fn validate_budget(&self) -> ProgramResult {
    // Deserialize the Vault data to check the balance
    let vault_data = VaultV1::try_from_slice(&self.accounts.vault.data.borrow())?;

    if self.instruction_data.amount > vault_data.amount {
        msg!("Error: Proposal amount exceeds DAO budget cap!");
        return Err(ProgramError::InsufficientFunds);
    }

    Ok(())
}
```

### B. Initializing the Proposal

We create the account and fill it with the starting state: `current_vote: 0` and `status: Active`.

```rust
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

    // Sign with Composite Seeds
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

    // Serialize Proposal Data
    let proposal_data = ProposalV1 {
        author: *self.accounts.author.key,
        target_recipient: self.instruction_data.target_recipient,
        amount: self.instruction_data.amount,
        current_vote: 0,
        status: ProposalStatus::Active,
    };
    proposal_data.serialize(&mut &mut self.accounts.proposal.data.borrow_mut()[..])?;
    
    Ok(())
}
```

## 6. The Instruction Processor (Atomicity)

We use our processor to ensure we never initialize a proposal that the DAO cannot afford to pay out.

```rust
impl<'a, 'info> InstructionProcessor for CreateProposalV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        // Step 1: Check if the Vault has enough funds
        self.validate_budget()?; 

        // Step 2: Create the proposal account
        self.init_proposal()
    }
}
```

> [!CAUTION]
> **Read Before You Write**
> In Solana Native, always perform your logic checks (like `validate_budget`) before you call `invoke_signed` to create an account. 
> This saves the user from paying transaction fees for a creation that would have been invalid anyway.

[⬅️ Previous: Initialize DAO](03-init-dao.md) | [Next: Cast Vote ➡️](05-cast-vote.md)
