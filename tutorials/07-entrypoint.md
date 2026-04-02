# The Entrypoint & Router

In this final technical chapter, we will implement the "Brain" of our program. This is the `Entrypoint`, the single door through which every transaction must pass, and the `Router` that directs traffic to our specific instruction logic.

## 1. The Instruction Enum (The Map)

In Solana Native, the program receives a raw buffer of bytes (`&[u8]`) as instruction data. We use a Rust `enum` to define every possible action our DAO can perform. When the client sends a `0`, it means "init_dao_v1"; a `1` means "create_proposal_v1," and so on.

```rust
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum DaoInstruction {
    InitDaoV1(InitDaoV1InstructionData),                // index 0
    CreateProposalV1(CreateProposalV1InstructionData),  // index 1
    CastVoteV1,                                         // index 2
    ExecuteProposalV1,                                  // index 3
}
```

[!TIP]

**💡 Why use an Enum?**
Using `BorshDeserialize` on this enum allows the program to automatically look at the first byte of the data to decide which variant to "unpack." This is the standard way to handle multiple instructions in a single Solana program.

## 2. The Entrypoint (The Front Door)

The `entrypoint!` macro is where the Solana Runtime hands over control to your code. It provides three things:

1. `program_id`: The public key of our deployed program.
1. `accounts`: The list of accounts involved in this specific transaction.
1. `instruction_data`: The raw bytes sent by the user.

```rust
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 1. Unpack the user's intent
    let instruction = DaoInstruction::try_from_slice(instruction_data)?;

    // 2. Route to the correct logic
    // ...
}
```

## 3. The Router (The Switchboard)

Once we know what the user wants to do (`instruction`), we use a `match` statement to bridge the gap between raw data and our high-level Instruction Structs.

This is where our `TryFrom` implementations and `InstructionProcessor` trait finally pay off.

```rust
match instruction {
    DaoInstruction::InitDaoV1(data) => {
        // Convert raw accounts + data into our validated Struct
        InitDaoV1::try_from((accounts, data, program_id))?.process()
    }
    DaoInstruction::CreateProposalV1(data) => {
        CreateProposalV1::try_from((accounts, data, program_id))?.process()
    }
    DaoInstruction::CastVoteV1 => {
        CastVoteV1::try_from((accounts, program_id))?.process()
    },
    DaoInstruction::ExecuteProposalV1 => {
        ExecuteProposalV1::try_from((accounts, program_id))?.process()
    }
}
```

### Why this pattern is "Bulletproof":

- **Encapsulation:** The `lib.rs` file stays extremely clean. It doesn't care how a vote is cast. It only cares about routing the request.
- **Fail-Fast:** If `try_from` fails (e.g., a missing signature or wrong PDA), the program returns an error immediately before any logic is executed.
- **Scalability:** Want to add a `CancelProposal` instruction? Just add a variant to the `enum` and a line in the `match` statement.

## 4. Final Project Structure

Your project should now look like this:

- `src/lib.rs`: The Entrypoint and Router.
- `src/states/`: The "Account Model" schema (DAO, Proposal, Vault).
- `src/instructions/`: The detailed logic for each action.
- `src/utils/`: The shared project helpers.

[⬅️ Previous: Execute Proposal](06-execute-proposal.md) | [Next: Deployment & Simulation ➡️](08-simulation.md)
