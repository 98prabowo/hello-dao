# Instruction: Execute Proposal

In this chapter, we implement the "payday" logic. 
This instruction demonstrates how a DAO actually moves funds and the unique way **Program Owned Accounts** handle SOL transfers.

## 1. The Instruction Data (The Input)

Like the voting instruction, `ExecuteProposalV1` doesn't require any custom input data. 
All the information we need (the amount and the recipient) is already stored in the `ProposalV1` account state from when it was created.

## 2. Defining the Accounts Struct

We need all the key players: the executor (who triggers the payout), the proposal being finalized, the Vault (where the money is), and the recipient.

```rust
pub struct ExecuteProposalV1Accounts<'a, 'info> {
    pub executor: &'a AccountInfo<'info>,         // Signer triggering the execution
    pub proposal: &'a AccountInfo<'info>,         // The proposal account (Passed status)
    pub vault: &'a AccountInfo<'info>,            // The treasury PDA
    pub target_recipient: &'a AccountInfo<'info>, // Account receiving the SOL
    pub dao: &'a AccountInfo<'info>,              // The global DAO config
}
```

### The Validation Logic (TryFrom)

Because we are moving money, validation is extremely strict:

- **Signer Check:** The `executor` must sign the transaction.
- **Writable Check:** The `proposal`, `vault`, and `target_recipient` must all be writable because their SOL balances or data will change.

```rust
impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for ExecuteProposalV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        // Check if accounts has correct len
        let [executor, proposal, vault, target_recipient, dao] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
  
        // Validate executor account
        if !executor.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Validate writability for balance/state changes
        if !proposal.is_writable || !vault.is_writable || !target_recipient.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        // Initialize accounts
        Ok(Self {
            executor,
            proposal,
            vault,
            target_recipient,
            dao,
        })
    }
}
```

## 3. The Master Instruction Struct

This struct bundles our validated accounts and the program ID for PDA verification.

```rust
pub struct ExecuteProposalV1<'a, 'info> {
    pub accounts: ExecuteProposalV1Accounts<'a, 'info>,
    pub program_id: &'a Pubkey,
}

```

## 4. PDA Verification

To prevent "fake vault" or "fake proposal" attacks, we re-derive and verify all PDAs.

Inside `TryFrom` for the master struct:

```rust
impl<'a, 'info> TryFrom<(&'a [AccountInfo<'info>], &'a Pubkey)> for ExecuteProposalV1<'a, 'info> {
    type Error = ProgramError;

    fn try_from(
        (accounts, program_id): (&'a [AccountInfo<'info>], &'a Pubkey),
    ) -> Result<Self, Self::Error> {
        // Deserialize accounts list
        let accounts = ExecuteProposalV1Accounts::try_from(accounts)?;

        // Validate DAO PDA
        let (dao_pda, _) = Pubkey::find_program_address(&[DaoV1::SEED], program_id);
        if dao_pda != *accounts.dao.key {
            msg!(
                "Invalid PDA: expected {}, got {}",
                dao_pda,
                accounts.dao.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate vault PDA
        let (vault_pda, _) = Pubkey::find_program_address(&[VaultV1::SEED], program_id);
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
            program_id,
        })
    }
}
```

## 5. The Execution Logic

This is where the actual transfer happens. Since the Vault is a **Program Owned Account** that carries data, we use "Manual Lamport Reallocation."

### A. Security & Integrity Checks

We ensure the proposal has actually passed and that the recipient provided in the instruction matches the one saved in the proposal state.

```rust
fn execute_proposal(&self) -> ProgramResult {
    let mut proposal_data = ProposalV1::try_from_slice(&self.accounts.proposal.data.borrow())?;
    let mut vault_data = VaultV1::try_from_slice(&self.accounts.vault.data.borrow())?;

    if proposal_data.status != ProposalStatus::Passed {
        return Err(ProgramError::InvalidAccountData);
    }

    if proposal_data.target_recipient != *self.accounts.target_recipient.key {
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

    // ...
}
```

### B. Moving the SOL (The "Magic" Reallocation)

In Solana Native, if a program owns an account, it can directly modify the `lamports` field. 
We don't call the System Program. We simply subtract from one and add to the other.

```rust
fn execute_proposal(&self) -> ProgramResult {
    // ...

    let amount_to_send = proposal_data.amount;
    if **self.accounts.vault.lamports.borrow() < amount_to_send {
        return Err(ProgramError::InsufficientFunds);
    }

    // Direct Lamport Reallocation
    **self.accounts.vault.lamports.borrow_mut() -= amount_to_send;
    **self.accounts.target_recipient.lamports.borrow_mut() += amount_to_send;

    // ...
}
```

### C. Updating State

Finally, we mark the proposal as `Executed` and update our internal Vault tracker to keep the data consistent with the actual balance.

```rust
fn execute_proposal(&self) -> ProgramResult {
    // ...

    proposal_data.status = ProposalStatus::Executed;
    vault_data.amount = vault_data
        .amount
        .checked_sub(amount_to_send)
        .ok_or(ProgramError::InsufficientFunds)?;

    // Save back to accounts
    proposal_data.serialize(&mut &mut self.accounts.proposal.data.borrow_mut()[..])?;
    vault_data.serialize(&mut &mut self.accounts.vault.data.borrow_mut()[..])?;

    Ok(())
}
```

## 6. The Instruction Processor (Atomicity)

The processor triggers the execution. If anything fails (like a recipient mismatch), the SOL never leaves the Vault.

```rust
impl<'a, 'info> InstructionProcessor for ExecuteProposalV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        self.execute_proposal()
    }
}
```

> [!CAUTION]
> **The "Must Not Carry Data" Trap**
> If you tried to use `system_instruction::transfer` here, the transaction would fail because the Vault has data. Only "System Owned" accounts (wallets) can use the standard transfer. 
> For Program Owned accounts, **manual reallocation** is the standard way to move funds.

[⬅️ Previous: Cast Vote](05-cast-vote.md) | [Next: Entrypoint & Router ➡️](07-entrypoint.md)
