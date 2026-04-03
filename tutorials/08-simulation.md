# Deployment & Client Simulation

In this final chapter, we leave the Rust environment to deploy our program to the Solana Devnet and use a TypeScript client to simulate a full DAO lifecycle: **Initialize -> Propose -> Vote -> Execute**.

## 1. Preparing for Deployment

Before shipping, we need a unique Program ID. Solana programs identify themselves using a public key declared inside the code.

### Step A: Generate a New Program ID

Run the following command to create a new keypair for your program.

```sh
make new-keypair
```

> [!IMPORTANT]
> **Update your Code:** Copy the address printed in the terminal and paste it into your `src/lib.rs` inside the `declare_id!("...")` macro. Then, rebuild the program: `make build`.

### Step B: Check Program Size

Solana requires you to pay "Rent" for the space your program occupies. Check if you have enough SOL:

```sh
make check-size
```

## 2. Shipping to Devnet

To deploy, you need a local wallet with Devnet SOL.

1. **Set to Devnet:** `solana config set --url devnet`
1. **Airdrop SOL:** `solana airdrop 2`
1. **Deploy:**

```sh
# make deploy AUTH=<your-deployer-wallet-path>
make deploy AUTH=~/.config/solana/id.json
```

## 3. Client Side: The Simulation

Your TypeScript client uses `@solana/web3.js` and `borsh` to talk to the program. It must mirror the data structures we built in Rust.

### How the Client "Speaks" to Rust

The client sends a **Buffer**. The first byte (the `instruction_index`) tells our `DaoInstruction` enum in Rust which variant to use.

| Instruction | Index (Byte 0) | Data Following Index |
| --- | --- | --- |
| `InitDao` | `0` | `vote_threshold (u64)`, `vested_amount (u64)` |
| `CreateProposal` | `1` | `recipient (Pubkey)`, `amount (u64)` |
| `CastVote` | `2` | (None) |
| `Execute` | `3` | (None) | 

## 4. The Full DAO Demo Loop

Open your terminal and run these commands in order to see the DAO in action.

### Step 1: Initialize the DAO

Creates the global DAO state and the Treasury Vault.

```sh
make demo-init
```

### Step 2: Create a Spending Proposal

The Payer proposes to send 0.001 SOL to the Recipient.

```sh
make demo-create
```

### Step 3: Cast a Vote

The Payer signs the transaction. Since the Payer has SOL balance (voting power) and we set a threshold in Step 1, this should move the proposal to `Passed`.

```sh
make demo-vote
```

### Step 4: Execute the Payout

The final step. The Vault reallocates lamports to the recipient's wallet.

```sh
make demo-execute
```

## 5. Troubleshooting & Tips

> [!CAUTION]
> **PDA Mismatch:** If your simulation fails with `InvalidAccountData`, double-check your seeds in the TypeScript code.
> They must be identical to the `SEED` constants in your Rust `states/*.rs` (e.g., "doa_v1" vs "dao_v1").

> [!TIP]
> **Logs are your best friend:** While running the simulation, open a separate terminal and run `solana logs -u devnet`. 
> You will see your `msg!` outputs from the Rust code appearing in real-time as the transactions process.

[⬅️ Previous: Entrypoint & Router](07-entrypoint.md) | [Back to: Readme ➡️](../README.md)
