import { PublicKey } from "@solana/web3.js";
import { getAuthorityPDA, DEFAULT_MULTISIG_PROGRAM_ID } from "@sqds/sdk";
import BN from "bn.js";
import {
  parseArgs,
  createWallet,
  initEscrowProgram,
  parseMultisig,
  executeOrPropose,
} from "./helpers";

const BPF_UPGRADABLE_LOADER = new PublicKey(
  "BPFLoaderUpgradeab1e11111111111111111111111"
);

async function main() {
  const args = parseArgs();
  const wallet = await createWallet(args);
  const { connection, program, programId } = initEscrowProgram(args, wallet);
  const multisigAddress = parseMultisig(args);

  const programToTransfer = new PublicKey(args.program);
  const currentAuthority = new PublicKey(args.authority);

  const programData = PublicKey.findProgramAddressSync(
    [programToTransfer.toBuffer()],
    BPF_UPGRADABLE_LOADER
  )[0];

  // In multisig mode, the vault becomes the new authority
  let newAuthority: PublicKey;
  if (multisigAddress) {
    [newAuthority] = getAuthorityPDA(
      multisigAddress,
      new BN(1),
      DEFAULT_MULTISIG_PROGRAM_ID
    );
  } else {
    newAuthority = wallet.publicKey;
  }

  const escrowAuthority = PublicKey.findProgramAddressSync(
    [currentAuthority.toBuffer(), newAuthority.toBuffer()],
    programId
  )[0];

  console.log("Accepting program authority transfer...");
  console.log(`  Wallet: ${wallet.publicKey.toBase58()}`);
  console.log(`  Program: ${programToTransfer.toBase58()}`);
  console.log(`  Current Authority: ${currentAuthority.toBase58()}`);
  console.log(`  New Authority: ${newAuthority.toBase58()}`);
  console.log(`  Escrow Authority: ${escrowAuthority.toBase58()}`);

  if (multisigAddress) {
    console.log(`  Multisig: ${multisigAddress.toBase58()}`);
  }

  const instruction = await program.methods
    .accept()
    .accounts({
      currentAuthority: currentAuthority,
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
