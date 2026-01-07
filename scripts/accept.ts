import { PublicKey } from "@solana/web3.js";
import {
  parseArgs,
  createWallet,
  initEscrowProgram,
  parseMultisig,
  executeOrPropose,
  getMultisigVault,
  BPF_UPGRADABLE_LOADER,
} from "./lib/helpers";

function usage(): never {
  console.error(`Usage: pnpm accept [options]

Accept a proposed program authority transfer.

Required:
  -k, --keypair <path|ledger>       Path to keypair file or 'ledger' for hardware wallet.
                                    This is the new authority (fee payer) accepting the transfer.
  -p, --program <address>           Address of the program to accept authority for.
  -a, --authority <address>         Address of the previous authority that proposed the transfer.

Optional:
  -d, --derivation-path <path>      Ledger derivation path as 'account/change' (default: 0).
  -m, --multisig <address>          Squads v3 multisig address. When provided, the multisig
                                    vault becomes the new authority and a proposal is created
                                    instead of executing directly.
  -M, --multisig-program <address>  Squads v3 program address (default: SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu).
  -i, --authority-index <index>     Multisig vault/authority index (default: 1).
  -u, --url <rpc_url>               RPC endpoint URL (default: https://api.mainnet-beta.solana.com).
`);
  process.exit(1);
}

async function main() {
  const args = parseArgs();

  if (!args.program || !args.authority || !args.keypair) {
    usage();
  }

  let programToTransfer: PublicKey;
  let currentAuthority: PublicKey;

  try {
    programToTransfer = new PublicKey(args.program);
  } catch {
    console.error(`Invalid program address: ${args.program}`);
    process.exit(1);
  }

  try {
    currentAuthority = new PublicKey(args.authority);
  } catch {
    console.error(`Invalid authority address: ${args.authority}`);
    process.exit(1);
  }

  const wallet = await createWallet(args);
  const { connection, program, programId } = initEscrowProgram(args, wallet);
  const multisigAddress = parseMultisig(args);
  const multisigProgramId = new PublicKey(args["multisig-program"]);
  const authorityIndex = parseInt(args["authority-index"], 10);

  const [programData] = PublicKey.findProgramAddressSync(
    [programToTransfer.toBuffer()],
    BPF_UPGRADABLE_LOADER
  );

  // In multisig mode, the vault becomes the new authority
  const newAuthority = multisigAddress
    ? getMultisigVault(multisigAddress, authorityIndex, multisigProgramId)
    : wallet.publicKey;

  const [escrowAuthority] = PublicKey.findProgramAddressSync(
    [currentAuthority.toBuffer(), newAuthority.toBuffer()],
    programId
  );

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
    .accountsPartial({
      currentAuthority,
      newAuthority,
      programAccount: programToTransfer,
      escrowAuthority,
      programData,
    })
    .instruction();

  await executeOrPropose(
    connection,
    wallet,
    multisigAddress,
    instruction,
    authorityIndex,
    multisigProgramId
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
