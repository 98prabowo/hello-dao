# Hello DAO 🏛️

**Mastering Solana Native by Building a Trustless Governance Engine**

**Hello DAO** is a high-intensity, educational repository designed to strip away the abstractions of high-level frameworks.
Here, we build a functional decentralized treasury from scratch using **Solana Native** (Rust).
Most tutorials hide the "plumbing", we embrace it.

## What You Will Learn

1. **Borsh Serialization:** How to handle serialization without errors.
1. **PDA (Program Derived Address):** How to create "Vaults" that don't have a private key and are controlled only by your program.
1. **Manual Lamport Reallocations:** Transferring SOL from Data Accounts.
1. **CPI (Cross-Program Invocation):** How the Governance program talks to the System Program to move SOL.
1. **On-Chain Voting:** Implementing a simple weighted voting system using SPL Tokens.
1. **Atomics & Security:** Why Solana transactions are "all-or-nothing" and how to prevent common attacks.

## The Developer Toolkit

We provide a specialized Makefile to handle the heavy lifting of the Solana CLI.

- `make new-keypair` — Generate a fresh Program ID and update your environment.
- `make build` — Compile to SBF (Solana Bytecode Format).
- `make check-size` — Calculate the exact SOL rent required for deployment.
- `make demo-init` — Trigger the TypeScript simulation to see the DAO go live.

## Tutorial Path:

1.  [Environment Setup](./tutorials/01-setup.md)
1.  [State Definitions](./tutorials/02-states.md)
1.  [Instruction: Initialize DAO & Vault](./tutorials/03-init-dao.md)
1.  [Instruction: Create Proposal](./tutorials/04-create-proposal.md)
1.  [Instruction: Cast Vote](./tutorials/05-cast-vote.md)
1.  [Instruction: Execute Proposal](./tutorials/06-execute-proposal.md)
1.  [The Entrypoint & Router](./tutorials/07-entrypoint.md)
1.  [Deployment & Client Simulation](./tutorials/08-simulation.md)

## Tutorial

> [!IMPORTANT]
> Before start the tutorial, after cloning this repository please create your own branch with `main` as base branch.

[Let's start learning ➡️](./tutorials/01-setup.md)

## Simulation Suite

The repository includes a TypeScript client (`client/simulate.ts`) that acts as a real-world user. It demonstrates:

- How to encode instructions using **Borsh**.
- How to handle **PDA derivation** in the browser/client-side.
- How to confirm transactions on the **Solana Devnet**.

## Resources

- [Solana Cookbook: PDAs](https://solana.com/docs/core/pda)
- [Developing Solana Programs in Rust](https://solana.com/docs/programs/rust)
- [SPL Governance](https://github.com/solana-labs/solana-program-library/blob/master/governance/README.md)

## Contributing

This is a **learning-first** repository. 
If you spot a "magic number" that needs explaining or a logic check that could be more readable, please open a PR. 
Help us build the best "Anchorless" guide on GitHub!

## Further Readings

- [Anchor](https://www.anchor-lang.com/docs)
- [Realms](https://docs.realms.todaynch)
