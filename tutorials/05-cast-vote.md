# Instruction: Cast Vote

In this chapter, we will implement the logic for the DAO's voting mechanism. This instruction demonstrates how to read from multiple state accounts simultaneously and perform **conditional status updates** based on shared thresholds.

## 1. The Instruction Data (The Input)

Interestingly, our `CastVoteV1` instruction does not require any external instruction data from the user.

```rust
// No custom struct needed! 
// The "Power" of the vote is derived directly from the account balance.
```

[!NOTE]
In this simple DAO design, we use **1 Lamport = 1 Vote**. Because we derive voting power from the account state itself, the user doesn't need to pass a "vote amount" parameter.

## 2. Defining the Accounts Struct

We need the voter's authority, the proposal being voted on, and the DAO config to check the required threshold.

```rust
pub struct CastVoteV1Accounts<'a, 'info> {
    pub voter: &'a AccountInfo<'info>,    // The person voting
        pub proposal: &'a AccountInfo<'info>, // The proposal being updated
        pub dao: &'a AccountInfo<'info>,      // The global DAO config (to read threshold)
}
```

### The Validation Logic (TryFrom)

We ensure the voter is legitimate and the proposal is ready to receive data:

- **Signer Check:** The `voter` must sign to prove they own the account providing the voting power.
- **Writable Check:** The `proposal` account must be writable because we are incrementing its vote count and potentially changing its status.

```rust
impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for CastVoteV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        // Check if accounts has correct len
        let [voter, proposal, dao] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Validate voter account
        if !voter.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Validate proposal account
        if !proposal.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        // Initialize accounts
        Ok(Self { voter, proposal, dao })
    }
}
```

## 3. The Master Instruction Struct

This struct holds our validated accounts and the `program_id` required for PDA security checks.

```rust
pub struct CastVoteV1<'a, 'info> {
    pub accounts: CastVoteV1Accounts<'a, 'info>,
    pub program_id: &'a Pubkey,
}
```

## 4. PDA Verification

We must verify that both the **DAO** and the **Proposal** accounts are authentic PDAs belonging to our program.

Inside `TryFrom` for the master struct:

```rust
impl<'a, 'info> TryFrom<(&'a [AccountInfo<'info>], &'a Pubkey)> for CastVoteV1<'a, 'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, program_id): (&'a [AccountInfo<'info>], &'a Pubkey),
    ) -> Result<Self, Self::Error> {
        // Deserialize accounts list
        let accounts = CastVoteV1Accounts::try_from(accounts)?;

        // Validate DAO PDA
        let (dao_pda, _dao_bump) = Pubkey::find_program_address(&[DaoV1::SEED], program_id);
        if dao_pda != *accounts.dao.key {
            msg!(
                "Invalid PDA: expected {}, got {}",
                dao_pda,
                accounts.dao.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        // Initialize master instruction struct
        Ok(Self {
            accounts,
            program_id,
        })
    }
}
```

[!IMPORTANT]
Even though we aren't creating the DAO account here, we must verify its address. If we didn't, a malicious user could pass a "fake" DAO account with a `vote_threshold` of 0, allowing any proposal to pass instantly!

## 5. The Execution Logic

The logic follows a "Read-Modify-Write" pattern.

### A. State Verification and Voting Power

We load the current data and calculate the voter's weight based on their SOL balance.

```rust
fn vote(&self) -> ProgramResult {
    let mut proposal_data = ProposalV1::try_from_slice(&self.accounts.proposal.data.borrow())?;
    let dao_data = DaoV1::try_from_slice(&self.accounts.dao.data.borrow())?;

    // Check if proposal is still Active
    if proposal_data.status != ProposalStatus::Active {
        return Err(ProgramError::InvalidAccountData);
    }

    // Self validate proposal PDA here because it needs `author` and `target_recipient`
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

    // Determine weight: 1 Lamport = 1 Voting Power
    let voting_power = self.accounts.voter.lamports();

    // Use saturating_add to prevent overflow crashes
    proposal_data.current_vote = proposal_data.current_vote.saturating_add(voting_power);

    // ...
}
```

### B. Conditional Status Update

After adding the new votes, we check if the proposal has crossed the finish line defined in the DAO config.

```rust
fn vote(&self) -> ProgramResult {
    // ...

    if proposal_data.current_vote >= dao_data.vote_threshold {
        proposal_data.status = ProposalStatus::Passed;
        msg!("Proposal status updated to PASSED!");
    }

    // Save the updated state back to the proposal account
    proposal_data.serialize(&mut &mut self.accounts.proposal.data.borrow_mut()[..])?;

    Ok(())
}
```

## 6. The Instruction Processor (Atomicity)

Since this instruction only performs one logical action, the processor simply calls the vote function.

```rust
impl<'a, 'info> InstructionProcessor for CastVoteV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        self.vote()
    }
}
```

[!CAUTION]
**Saturating Math**
Always use `saturating_add` or `checked_add` in financial or voting logic. If you use a standard `+` and the vote count exceeds the maximum value of a `u64`, the program will panic and the transaction will fail, effectively breaking the voting process for very popular proposals.

[⬅️ Previous: Create Proposal](04-create-proposal.md) | [Next: Execute Proposal ➡️](06-execute-proposal.md)
