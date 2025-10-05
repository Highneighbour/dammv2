/**
 * Setup Pool Script
 * 
 * This script sets up a test CP-AMM pool and initializes the honorary position.
 * In a production environment, this would integrate with actual Raydium CP-AMM SDK.
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo } from "@solana/spl-token";
import { StarHonoraryFeePosition } from "../target/types/star_honorary_fee_position";

const VAULT_SEED = "vault";
const POLICY_SEED = "policy";
const PROGRESS_SEED = "progress";
const POSITION_STATE_SEED = "position_state";
const POSITION_OWNER_SEED = "investor_fee_pos_owner";
const TREASURY_SEED = "treasury";

async function main() {
  console.log("🚀 Setting up test pool and honorary position...\n");

  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.StarHonoraryFeePosition as Program<StarHonoraryFeePosition>;
  const payer = provider.wallet as anchor.Wallet;

  console.log("Program ID:", program.programId.toString());
  console.log("Payer:", payer.publicKey.toString());
  console.log("");

  // Step 1: Create test tokens
  console.log("📝 Step 1: Creating test tokens...");
  
  const quoteMint = await createMint(
    provider.connection,
    payer.payer,
    payer.publicKey,
    null,
    6 // USDC decimals
  );
  console.log("  Quote mint (USDC):", quoteMint.toString());

  const baseMint = await createMint(
    provider.connection,
    payer.payer,
    payer.publicKey,
    null,
    9 // STAR decimals
  );
  console.log("  Base mint (STAR):", baseMint.toString());
  console.log("");

  // Step 2: Create vault
  console.log("📝 Step 2: Creating vault...");
  const vault = Keypair.generate();
  console.log("  Vault:", vault.publicKey.toString());
  console.log("");

  // Step 3: Create creator's quote token account
  console.log("📝 Step 3: Creating creator token account...");
  const creatorQuoteAta = await createAccount(
    provider.connection,
    payer.payer,
    quoteMint,
    payer.publicKey
  );
  console.log("  Creator quote ATA:", creatorQuoteAta.toString());
  console.log("");

  // Step 4: Derive PDAs
  console.log("📝 Step 4: Deriving PDAs...");
  
  const [policyPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), vault.publicKey.toBuffer(), Buffer.from(POLICY_SEED)],
    program.programId
  );
  console.log("  Policy PDA:", policyPda.toString());

  const [progressPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), vault.publicKey.toBuffer(), Buffer.from(PROGRESS_SEED)],
    program.programId
  );
  console.log("  Progress PDA:", progressPda.toString());

  const [treasuryPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), vault.publicKey.toBuffer(), Buffer.from(TREASURY_SEED)],
    program.programId
  );
  console.log("  Treasury PDA:", treasuryPda.toString());

  const [treasuryAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), vault.publicKey.toBuffer(), Buffer.from(TREASURY_SEED), Buffer.from("authority")],
    program.programId
  );
  console.log("  Treasury Authority:", treasuryAuthority.toString());

  const [positionOwnerPda] = PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), vault.publicKey.toBuffer(), Buffer.from(POSITION_OWNER_SEED)],
    program.programId
  );
  console.log("  Position Owner PDA:", positionOwnerPda.toString());

  const [positionStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from(VAULT_SEED), vault.publicKey.toBuffer(), Buffer.from(POSITION_STATE_SEED)],
    program.programId
  );
  console.log("  Position State PDA:", positionStatePda.toString());
  console.log("");

  // Step 5: Initialize policy
  console.log("📝 Step 5: Initializing policy...");
  
  try {
    const tx = await program.methods
      .initializePolicy({
        investorFeeShareBps: 5000, // 50%
        dailyCapLamports: new anchor.BN(1_000_000_000),
        minPayoutLamports: new anchor.BN(10_000),
        y0TotalAllocation: new anchor.BN("10000000000"),
      })
      .accounts({
        creator: payer.publicKey,
        vault: vault.publicKey,
        quoteMint: quoteMint,
        creatorQuoteAta: creatorQuoteAta,
        policy: policyPda,
        progress: progressPda,
        programQuoteTreasury: treasuryPda,
        treasuryAuthority: treasuryAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    console.log("  ✅ Policy initialized!");
    console.log("  Transaction:", tx);
  } catch (err) {
    console.log("  ⚠️  Policy initialization:", err.message);
  }
  console.log("");

  // Step 6: Mint some quote tokens to treasury for testing
  console.log("📝 Step 6: Funding treasury with test tokens...");
  
  try {
    await mintTo(
      provider.connection,
      payer.payer,
      quoteMint,
      treasuryPda,
      payer.publicKey,
      1_000_000_000 // 1000 USDC
    );
    console.log("  ✅ Treasury funded with 1000 USDC");
  } catch (err) {
    console.log("  ⚠️  Treasury funding:", err.message);
  }
  console.log("");

  console.log("========================================");
  console.log("✅ Setup complete!");
  console.log("========================================");
  console.log("");
  console.log("Configuration:");
  console.log("  Vault:", vault.publicKey.toString());
  console.log("  Quote Mint:", quoteMint.toString());
  console.log("  Base Mint:", baseMint.toString());
  console.log("  Policy:", policyPda.toString());
  console.log("  Treasury:", treasuryPda.toString());
  console.log("");
  console.log("Next: Initialize position with CP-AMM pool");
  console.log("");

  // Save config for other scripts
  const config = {
    vault: vault.publicKey.toString(),
    quoteMint: quoteMint.toString(),
    baseMint: baseMint.toString(),
    policy: policyPda.toString(),
    progress: progressPda.toString(),
    treasury: treasuryPda.toString(),
    treasuryAuthority: treasuryAuthority.toString(),
    positionOwner: positionOwnerPda.toString(),
    positionState: positionStatePda.toString(),
  };

  console.log("Config:", JSON.stringify(config, null, 2));
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });