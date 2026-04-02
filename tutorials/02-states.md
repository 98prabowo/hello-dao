# State Definitions

In Solana, we don't have a centralized database. Instead, every "row" of data is its own **Account**. To find these accounts without a database, we use **PDAs (Program Derived Addresses)**.

## 1. The PDA: Your Deterministic Index

A PDA is an address derived from **Seeds** and your **Program ID**. Think of it like a file path in a folder:

$$PDA = hash(seeds, program\_id)$$

Because the address is calculated, the Offchain (TypeScript) and the Onchain (Rust) will always find the **exact same account** as long as they use the same seeds.

## 2. Defining our "Rows" (Structs)

To store data in these accounts, we define Rust `structs`. However, Solana accounts are just raw buffers of bytes. We use **Borsh** to turn our Rust objects into bytes and back again.

### A. The DAO State

**Seed:** `[b"dao_v1"]`

This is a singleton account (only one exists per program) that holds global configuration.

```rust
#[derive(BorshSerialize, BorshDeserialize)]

pub struct DaoV1 {
    pub admin: Pubkey,  // The boss of the DAO
    pub fee: u64,       // Example config
}

impl DaoV1 {
    pub const LEN: usize = 32 + 8;
    pub const SEED: &[u8] = b"dao_v1";
}
```

### B. The Proposal

**Seeds:** `[b"proposal_v1", author_pubkey, recipient_pubkey]`

By using the `author` and `recipient` as seeds, we create a unique "ID" for every proposal.

```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
#[repr(u8)] // Forces the enum to take exactly 1 byte
pub enum ProposalStatus {
    Active,
    Passed,
    Executed,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct ProposalV1 {
    pub author: Pubkey,                 // 32 bytes
        pub target_recipient: Pubkey,   // 32 bytes
        pub amount: u64,                // 8 bytes
        pub current_vote: u64,          // 8 bytes
        pub status: ProposalStatus,     // 1 byte
}

impl ProposalV1 {
    // Total: 32 + 32 + 8 + 8 + 1 = 81 bytes
    pub const LEN: usize = 81; 
    pub const SEED: &[u8] = b"proposal_v1";
}
```

### C. The Vault

**Seeds:** `[b"vault_v1"]`

This is the account that will function as DAO treasury

```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct VaultV1 {
    pub amount: u64, // Vested amount
}

impl VaultV1 {
    pub const LEN: usize = 8;
    pub const SEED: &[u8; 8] = b"vault_v1";
}
```

## 3. The "Memory Alignment" Trap

> [!CAUTION]
You might notice we use `pub const LEN: usize = 81;` instead of `std::mem::size_of::<Self>()`.
**Why?** Rust often adds **"padding"** bytes to align data in memory (making 81 bytes become 88 bytes) to optimize CPU access. However, **Borsh** is a compact serializer; it does not care about padding and only writes the raw data.
If you reserve 88 bytes in the account but Borsh only writes 81, the program will see 7 "leftover" bytes and throw the dreaded `Not all bytes read` error during deserialization.

> [!TIP]
Always calculate your LEN manually by summing up the exact bytes of each field:
- `Pubkey` = 32 bytes
- `u64` = 8 bytes
- `u8` / `Enum` = 1 byte

## 4. How the Program "Signs" for the Vault

Our **Vault** (the Treasury) is also a PDA with the seed `[b"vault_v1"]`.

Because it is a PDA, it has no private key. When we need to move money out of it during the **Execute** phase, our program tells the Solana Runtime:

"I am the Program ID that owns the 'vault_v1' seeds. I authorize this movement of lamports."

This is called **CPI (Cross-Program Invocation)** with seeds, and it's how smart contracts control funds securely.

[⬅️ Previous: Environment Setup](01-setup.md) | [Next: Initialize DAO ➡️](03-init-dao.md)
