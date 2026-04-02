import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import * as borsh from "borsh";
import * as fs from "fs";
import * as path from "path";

// --- Program ID ---

function loadKeypair(fileName: string): Keypair {
  try {
    const keyPath = path.resolve(__dirname, fileName);
    const secretKeyString = fs.readFileSync(keyPath, { encoding: "utf8" });
    const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
    return Keypair.fromSecretKey(secretKey);
  } catch (err) {
    console.error(
      `❌ Gagal membaca keypair: ${fileName}. Pastikan file tersedia.`,
    );
    process.exit(1);
  }
}

function getProgramId(): PublicKey {
  try {
    const keypairPath = path.resolve(
      __dirname,
      "../target/deploy/hello_dao-keypair.json",
    );
    const secretKeyString = fs.readFileSync(keypairPath, { encoding: "utf8" });
    const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
    const programKeypair = Keypair.fromSecretKey(secretKey);

    console.log(
      `✅ Program ID detected: ${programKeypair.publicKey.toBase58()}`,
    );
    return programKeypair.publicKey;
  } catch (err) {
    console.error("❌ Failed reading Program ID. Please run 'make build'");
    process.exit(1);
  }
}

const PROGRAM_ID = getProgramId();
const connection = new Connection("https://api.devnet.solana.com", "confirmed");

// --- Seeds ---

const DAO_SEED = Buffer.from("dao_v1");
const PROPOSAL_SEED = Buffer.from("proposal_v1");
const VAULT_SEED = Buffer.from("vault_v1");

// --- Schemas ---

class InitDaoArgs {
  instruction_index = 0;
  vote_threshold: bigint;
  vested_amount: bigint;

  constructor(fields: { vote_threshold: bigint; vested_amount: bigint }) {
    this.vote_threshold = fields.vote_threshold;
    this.vested_amount = fields.vested_amount;
  }

  static schema = new Map([
    [
      InitDaoArgs,
      {
        kind: "struct",
        fields: [
          ["instruction_index", "u8"],
          ["vote_threshold", "u64"],
          ["vested_amount", "u64"],
        ],
      },
    ],
  ]);
}

class CreateProposalArgs {
  instruction_index = 1;
  target_recipient: Uint8Array;
  amount: bigint;

  constructor(fields: { target_recipient: Uint8Array; amount: bigint }) {
    this.target_recipient = fields.target_recipient;
    this.amount = fields.amount;
  }

  static schema = new Map([
    [
      CreateProposalArgs,
      {
        kind: "struct",
        fields: [
          ["instruction_index", "u8"],
          ["target_recipient", [32]],
          ["amount", "u64"],
        ],
      },
    ],
  ]);
}

// --- Individual IX Functions ---

async function initDao(payer: Keypair) {
  console.log("🎬 Executing: InitDao");

  const [daoPda] = PublicKey.findProgramAddressSync([DAO_SEED], PROGRAM_ID);
  const [vaultPda] = PublicKey.findProgramAddressSync([VAULT_SEED], PROGRAM_ID);

  const args = new InitDaoArgs({
    vote_threshold: BigInt(5 * LAMPORTS_PER_SOL),
    vested_amount: BigInt(0.1 * LAMPORTS_PER_SOL),
  });
  const data = Buffer.from(borsh.serialize(InitDaoArgs.schema, args));

  const ix = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: daoPda, isSigner: false, isWritable: true },
      { pubkey: vaultPda, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data,
  });

  const tx = new Transaction().add(ix);
  return await sendAndConfirmTransaction(connection, tx, [payer]);
}

async function createProposal(payer: Keypair, recipient: Keypair) {
  console.log("🎬 Executing: CreateProposal");

  const [proposalPda] = PublicKey.findProgramAddressSync(
    [PROPOSAL_SEED, payer.publicKey.toBuffer(), recipient.publicKey.toBuffer()],
    PROGRAM_ID,
  );
  const [vaultPda] = PublicKey.findProgramAddressSync([VAULT_SEED], PROGRAM_ID);

  const args = new CreateProposalArgs({
    target_recipient: recipient.publicKey.toBuffer(),
    amount: BigInt(0.001 * LAMPORTS_PER_SOL),
  });
  const data = Buffer.from(borsh.serialize(CreateProposalArgs.schema, args));

  const ix = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: proposalPda, isSigner: false, isWritable: true },
      { pubkey: vaultPda, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: data,
  });

  const sig = await sendAndConfirmTransaction(
    connection,
    new Transaction().add(ix),
    [payer],
  );

  return sig;
}

async function castVote(payer: Keypair, recipient: Keypair) {
  console.log("🎬 Executing: CastVote");

  const [proposalPda] = PublicKey.findProgramAddressSync(
    [PROPOSAL_SEED, payer.publicKey.toBuffer(), recipient.publicKey.toBuffer()],
    PROGRAM_ID,
  );
  const [daoPda] = PublicKey.findProgramAddressSync([DAO_SEED], PROGRAM_ID);

  const ix = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      { pubkey: proposalPda, isSigner: false, isWritable: true },
      { pubkey: daoPda, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: Buffer.from([2]),
  });

  return await sendAndConfirmTransaction(
    connection,
    new Transaction().add(ix),
    [payer],
  );
}

async function executeProposal(payer: Keypair, recipient: Keypair) {
  console.log("🎬 Executing: ExecuteProposal");

  const [proposalPda] = PublicKey.findProgramAddressSync(
    [PROPOSAL_SEED, payer.publicKey.toBuffer(), recipient.publicKey.toBuffer()],
    PROGRAM_ID,
  );
  const [vaultPda] = PublicKey.findProgramAddressSync([VAULT_SEED], PROGRAM_ID);
  const [daoPda] = PublicKey.findProgramAddressSync([DAO_SEED], PROGRAM_ID);

  const ix = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: proposalPda, isSigner: false, isWritable: true },
      { pubkey: vaultPda, isSigner: false, isWritable: true },
      { pubkey: recipient.publicKey, isSigner: false, isWritable: true },
      { pubkey: daoPda, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: Buffer.from([3]),
  });

  return await sendAndConfirmTransaction(
    connection,
    new Transaction().add(ix),
    [payer],
  );
}

// --- Main CLI Logic ---

const command = process.argv[2];

async function run() {
  const payer = loadKeypair("payer.json");
  const recipient = loadKeypair("recipient.json");

  console.log(`🔑 Payer: ${payer.publicKey.toBase58()}`);
  console.log(`🔑 Recipient: ${recipient.publicKey.toBase58()}`);

  let sig;
  switch (command) {
    case "init":
      sig = await initDao(payer);
      break;

    case "create":
      sig = await createProposal(payer, recipient);
      break;

    case "vote":
      sig = await castVote(payer, recipient);
      break;

    case "execute":
      sig = await executeProposal(payer, recipient);
      break;

    default:
      console.log("Usage: npm run simulate [init|create|vote|execute]");
      return;
  }

  if (sig) console.log(`✅ Success! Signature: ${sig}`);
}

run().catch(console.error);
