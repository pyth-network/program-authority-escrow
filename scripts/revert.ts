import { PublicKey } from "@solana/web3.js";
import {
  parseArgs as parseArgs,
  createWallet,
  initEscrowProgram,
  parseMultisig,
  executeOrPropose,
} from "./helpers";

function usage(): never {
  console.error(
    "Usage: yarn revert --keypair <path|ledger> --program <address> --authority <address> [--derivation-path <account/change>] [--multisig <address>] [--url <rpc_url>]"
  );
  process.exit(1);
}

async function main() {
  const args = parseArgs();

  if (!args.program || !args.authority || !args.keypair) {
    usage();
  }

  let programToTransfer: PublicKey;
  let newAuthority: PublicKey;

  try {
    programToTransfer = new PublicKey(args.program);
  } catch {
    console.error(`Invalid program address: ${args.program}`);
    process.exit(1);
  }

  try {
    newAuthority = new PublicKey(args.authority);
  } catch {
    console.error(`Invalid authority address: ${args.authority}`);
    process.exit(1);
  }

  const wallet = await createWallet(args);
  const { connection, program, programId } = initEscrowProgram(args, wallet);

  const BPF_UPGRADABLE_LOADER = new PublicKey(
    "BPFLoaderUpgradeab1e11111111111111111111111"
  );

  const escrowAuthority = PublicKey.findProgramAddressSync(
    [wallet.publicKey.toBuffer(), newAuthority.toBuffer()],
    programId
  )[0];

  const programData = PublicKey.findProgramAddressSync(
    [programToTransfer.toBuffer()],
    BPF_UPGRADABLE_LOADER
  )[0];

  console.log("Reverting program authority transfer...");
  console.log(`  Wallet: ${wallet.publicKey.toBase58()}`);
  console.log(`  Program: ${programToTransfer.toBase58()}`);
  console.log(`  New Authority: ${newAuthority.toBase58()}`);
  console.log(`  Escrow Authority: ${escrowAuthority.toBase58()}`);

  const multisigAddress = parseMultisig(args);
  if (multisigAddress) {
    console.log(`  Multisig: ${multisigAddress.toBase58()}`);
  }

  const instruction = await program.methods
    .revert()
    .accounts({
      currentAuthority: wallet.publicKey,
      newAuthority: newAuthority,
      programAccount: programToTransfer,
    })
    .accountsPartial({
      escrowAuthority: escrowAuthority,
      programData: programData,
    })
    .instruction();

  await executeOrPropose(connection, wallet, multisigAddress, instruction);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
