import minimist from "minimist";
import {
  AnchorProvider,
  Program,
  setProvider,
  Wallet,
} from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmRawTransaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { LedgerNodeWallet } from "./ledger";
import * as fs from "fs";
import type { ProgramAuthorityEscrow } from "../../target/types/program_authority_escrow";
import IDL from "../../target/idl/program_authority_escrow.json";
import Squads, {
  getAuthorityPDA,
  DEFAULT_MULTISIG_PROGRAM_ID,
} from "@sqds/sdk";
import BN from "bn.js";

export const BPF_UPGRADABLE_LOADER = new PublicKey(
  "BPFLoaderUpgradeab1e11111111111111111111111"
);

export interface Args {
  program: string;
  authority: string;
  keypair: string;
  "derivation-path": string;
  multisig?: string;
  "multisig-program": string;
  "authority-index": string;
  url: string;
}

export function parseArgs(): Args {
  const args = minimist(process.argv.slice(2), {
    string: [
      // "program" is the pubkey of the program whose authority will be changed.
      "program",
      // "authority" is the **new** authority of the program on `propose` and
      // `revert`, and the **previous** authority of the program on `accept`.
      "authority",
      // "keypair" is the fee-payer and current program authority on `propose`
      // and `revert`, or the new authority on `accept`. It takes a path to a
      // keypair file or the string 'ledger' if using a Ledger hardware wallet.
      "keypair",
      // "derivation-path" is the derivation path of the ledger wallet keypair.
      "derivation-path",
      // "multisig" is used when a Squads v3 multisig is involved in the
      // transfer and changes the transaction sending to create a multisig
      // proposal instead. You probably want to use this with `accept` when
      // transfering the program authority to a multisig.
      "multisig",
      // "multisig-program" is the address of the Squads v3 multisig program.
      "multisig-program",
      // "authority-index" is the vault index of the multisig to use.
      "authority-index",
      // "url" is the URL for the RPC endpoint.
      "url",
    ],
    alias: {
      p: "program",
      a: "authority",
      k: "keypair",
      d: "derivation-path",
      m: "multisig",
      M: "multisig-program",
      i: "authority-index",
      u: "url",
    },
    default: {
      url: "https://api.mainnet-beta.solana.com",
      "derivation-path": "0",
      "multisig-program": DEFAULT_MULTISIG_PROGRAM_ID.toBase58(),
      "authority-index": "1",
    },
  });

  return args as unknown as Args;
}

export async function createWallet(
  args: Args
): Promise<Wallet | LedgerNodeWallet> {
  if (args.keypair === "ledger") {
    const parts = args["derivation-path"].split("/");
    const account = parseInt(parts[0], 10);
    const change = parts[1] ? parseInt(parts[1], 10) : undefined;
    return await LedgerNodeWallet.createWallet(account, change);
  } else {
    const keypairData = JSON.parse(fs.readFileSync(args.keypair, "utf-8"));
    const keypair = Keypair.fromSecretKey(Uint8Array.from(keypairData));
    return new Wallet(keypair);
  }
}

export interface ProgramSetup {
  connection: Connection;
  provider: AnchorProvider;
  program: Program<ProgramAuthorityEscrow>;
  programId: PublicKey;
}

export function initEscrowProgram(
  args: Args,
  wallet: Wallet | LedgerNodeWallet
): ProgramSetup {
  const connection = new Connection(args.url);
  const provider = new AnchorProvider(connection, wallet, {});
  setProvider(provider);
  const program = new Program(IDL as ProgramAuthorityEscrow, provider);
  const programId = new PublicKey(IDL.address);
  return { connection, provider, program, programId };
}

export function parseMultisig(args: Args): PublicKey | undefined {
  if (!args.multisig) return undefined;
  try {
    return new PublicKey(args.multisig);
  } catch {
    console.error(`Invalid multisig address: ${args.multisig}`);
    process.exit(1);
  }
}

export function getMultisigVault(
  multisigAddress: PublicKey,
  authorityIndex: number,
  multisigProgramId: PublicKey = DEFAULT_MULTISIG_PROGRAM_ID
): PublicKey {
  const [vault] = getAuthorityPDA(
    multisigAddress,
    new BN(authorityIndex),
    multisigProgramId
  );
  return vault;
}

export async function executeOrPropose(
  connection: Connection,
  wallet: Wallet | LedgerNodeWallet,
  multisigAddress: PublicKey | undefined,
  instruction: TransactionInstruction,
  authorityIndex: number = 1,
  multisigProgramId: PublicKey = DEFAULT_MULTISIG_PROGRAM_ID
): Promise<void> {
  try {
    if (multisigAddress) {
      const squads = new Squads({
        connection,
        wallet: wallet as any,
        multisigProgramId,
      });
      const msTransaction = await squads.createTransaction(
        multisigAddress,
        authorityIndex
      );
      await squads.addInstruction(msTransaction.publicKey, instruction);
      await squads.activateTransaction(msTransaction.publicKey);
      await squads.approveTransaction(msTransaction.publicKey);
      console.log(
        `Multisig proposal signature: ${msTransaction.publicKey.toBase58()}`
      );
    } else {
      const tx = new Transaction().add(instruction);
      const { blockhash } = await connection.getLatestBlockhash();
      tx.recentBlockhash = blockhash;
      tx.feePayer = wallet.publicKey;
      await wallet.signTransaction(tx);
      const sig = await sendAndConfirmRawTransaction(
        connection,
        tx.serialize()
      );
      console.log(`Signature: ${sig}`);
    }
  } catch (err) {
    console.error("Transaction failed:", err);
    process.exit(1);
  }
}
