# Instruction: Initialize DAO & Vault

In this chapter, we will write the code that creates our DAO's global configuration and the Treasury (Vault).

## 1. The Instruction Data (The Input)

Before we look at accounts, we must define what information the user is sending. We use a dedicated struct for the parameters.

```rust
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct InitDaoV1InstructionData {
    pub vote_threshold: u64, // Number of votes required to pass a proposal
    pub vested_amount: u64,  // Initial SOL to deposit into the Vault
}
```

> [!NOTE]
> We use `u64` for the `vested_amount` because Solana balances (lamports) are always 64-bit unsigned integers. 
> This avoids rounding errors that occur with floating-point numbers.

## 2. Defining the Accounts Struct

We define the `InitDaoV1Accounts` struct to list every account required for this operation. 
In Solana Native, we must manually verify that these accounts are valid.

```rust
pub struct InitDaoV1Accounts<'a, 'info> {
    pub admin: &'a AccountInfo<'info>,          // The signer paying for the transaction
    pub dao: &'a AccountInfo<'info>,            // The PDA for DAO config [b"dao_v1"]
    pub vault: &'a AccountInfo<'info>,          // The PDA for the Treasury [b"vault_v1"]
    pub system_program: &'a AccountInfo<'info>, // Required to create new accounts
}
```

### The Validation Logic (TryFrom)

We implement `TryFrom` to ensure the client isn't sending malicious or incorrect accounts:

- **Signer Check:** The `admin` must sign the transaction.
- **Writable Check:** The `dao` and `vault` must be writable because we are creating them.
- **Program Check:** We verify the `system_program` matches the official Solana System Program ID.

```rust
impl<'a, 'info> TryFrom<&'a [AccountInfo<'info>]> for InitDaoV1Accounts<'a, 'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo<'info>]) -> Result<Self, Self::Error> {
        // Check if accounts has correct len
        let [admin, dao, vault, system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Validate admin account
        if !admin.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !admin.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate dao config account
        if !dao.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate vault account
        if !vault.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate system program
        if system_program.key != &solana_system_interface::program::ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Initialize accounts
        Ok(Self {
            admin,
            dao,
            vault,
            system_program,
        })
    }
}
```

## 3. The Master Instruction Struct

We wrap the **Accounts** and the **Instruction Data** into one object. 
This struct also stores the **Bumps** for our PDAs so we don't have to re-calculate them later, saving processing power (Compute Units).

```rust
pub struct InitDaoV1<'a, 'info> {
    pub accounts: InitDaoV1Accounts<'a, 'info>,
    pub instruction_data: InitDaoV1InstructionData,
    pub program_id: &'a Pubkey,
    pub dao_bump: u8,
    pub vault_bump: u8,
}
```

## 4. PDA Verification

Inside the `TryFrom` for the master struct, we perform the most important security check: **PDA Derivation**.

```rust
let (dao_pda, dao_bump) = Pubkey::find_program_address(&[DaoV1::SEED], program_id);
if dao_pda != *accounts.dao.key {
    msg!("Invalid PDA: expected {}, got {}", dao_pda, accounts.dao.key);
    return Err(ProgramError::InvalidAccountData);
}
```

> [!IMPORTANT]
> By re-calculating the PDA inside the program using our hardcoded `SEED`, we guarantee that the user is interacting with our DAO and not a fake one they created elsewhere.

Inside `TryFrom` for our master instruction struct we have `program_id`. We can check PDA derivation.

```rust
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
        // Deserialize accounts list
        let accounts = InitDaoV1Accounts::try_from(accounts)?;

        // Validate DAO PDA
        let (dao_pda, dao_bump) = Pubkey::find_program_address(&[DaoV1::SEED], program_id);
        if dao_pda != *accounts.dao.key {
            msg!(
                "Invalid PDA: expected {}, got {}",
                dao_pda,
                accounts.dao.key
            );
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate vault PDA
        let (vault_pda, vault_bump) = Pubkey::find_program_address(&[VaultV1::SEED], program_id);
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
            dao_bump,
            vault_bump,
        })
    }
}
```

## 5. The Execution Logic

Because we are using **Solana Native**, we must manually handle the account creation. The flow looks like this:

1. **Calculate Rent:** Ask the network how many lamports are needed for our LEN.
1. **Invoke System Program:** Call `create_account` using `invoke_signed`.
1. **Serialize Data:** Write our initial Rust struct into the new account.

### A. Initialize DAO Config

This is the function that actually puts the DAO on the map.

```rust
fn init_dao(accounts: &InitializeDaoAccounts, program_id: &Pubkey, vault_bump: u8) -> ProgramResult {
    let rent = Rent::get()?;

    // Use the LEN we defined in 02-states!
    let space = DaoV1::LEN; 
    let lamports = rent.minimum_balance(space);

    // Create the account via System Program
    let create_idx = solana_program::system_instruction::create_account(
        accounts.admin.key,
        accounts.dao.key,
        lamports,
        space as u64,
        program_id,
    );

    // Sign with PDA seeds so the Runtime knows we have authority
    invoke_signed(
        &create_idx,
        &[accounts.admin.clone(), accounts.dao.clone(), accounts.system_program.clone()],
        &[&[DaoV1::SEED, &[dao_bump]]], // We provide the 'signature' here
    )?;

    // Serialize initial data
    let dao_data = DaoV1 {
        admin: *accounts.admin.key,
        fee: 500, 
    };
    dao_data.serialize(&mut &mut accounts.dao.data.borrow_mut()[..])?;

    Ok(())
}
```

### B. Initialize Vault

This follows the same pattern but adds the `vested_amount` to the initial balance, allowing the Admin to fund the DAO immediately upon creation.

```rust
fn init_vault(&self) -> ProgramResult {
    let rent = Rent::get()?;
    let space = VaultV1::LEN;

    // We add 'vested_amount' to the rent-exempt minimum
    let lamports = rent.minimum_balance(space) + self.instruction_data.vested_amount;

    // Create account instruction
    let ix = solana_system_interface::instruction::create_account(
        self.accounts.admin.key,
        self.accounts.vault.key,
        lamports,
        space as u64,
        self.program_id,
    );

    // Send instruction with PDA signature (invoke_signed)
    invoke_signed(
        &ix,
        &[
            self.accounts.admin.clone(),
            self.accounts.vault.clone(),
            self.accounts.system_program.clone(),
        ],
        &[&[VaultV1::SEED, &[self.vault_bump]]],
    )?;

    // Store the initial amount in the data for internal tracking
    let vault_data = VaultV1 {
        amount: self.instruction_data.vested_amount,
    };
    vault_data.serialize(&mut &mut self.accounts.vault.data.borrow_mut()[..])?;

    msg!("Vault Configured with {} lamports", self.instruction_data.vested_amount);
    Ok(())
}
```

## 6. The Instruction Processor (Atomicity)

Finally, we implement the `InstructionProcessor` trait. This ensures that both the DAO and the Vault are created together.

```rust
impl<'a, 'info> InstructionProcessor for InitDaoV1<'a, 'info> {
    fn process(self) -> ProgramResult {
        // If init_dao fails, the transaction halts.
        self.init_dao()?; 

        // If init_vault fails, the DAO creation is REVERTED.
        self.init_vault()
    }
}
```

> [!CAUTION]
> **The Power of Atomicity**
> In Solana, a transaction is "Atomic". 
> If `init_vault` fails (for example, the admin doesn't have enough SOL for the `vested_amount`), the entire transaction fails. 
> The dao account will not be created. This prevents "broken states" where you have a DAO config but no Treasury.

---

### 🏁 Checkpoint

At this point, the DAO exists and the Vault is ready to receive funds. In the next chapter, we will allow users to start proposing how to spend that money.

[⬅️ Previous: State Definitions](02-states.md) | [Next: Create Proposal ➡️](04-create-proposal.md)
