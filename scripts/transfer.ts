import { AnchorProvider, BN, Program } from "@project-serum/anchor";
import { LedgerNodeWallet } from "./ledger";
import {Connection, PublicKey} from "@solana/web3.js";
import { ProgramAuthorityTimelock, IDL } from "../target/types/program_authority_timelock";

const PROGRAM_TO_TRANSFER = new PublicKey("pytS9TjG1qyAZypk7n8rw8gfW9sUaqqYyMhJQ4E7JCQ")
const NEW_AUTHORITY = new PublicKey("HVx4oW785bu8QDQ8AwSVfD7H4iuH51ttakc2G5f9XTX8")
const TIMESTAMP = new BN("1706745600")

const BPF_UPGRADABLE_LOADER = new PublicKey(
    "BPFLoaderUpgradeab1e11111111111111111111111"
  );
const TIMELOCK = new PublicKey("t1monUESMN3oVEoAw9HQkaVX6hUGg3hkhN5wKaTvV5f");

async function main(){
    const wallet = await LedgerNodeWallet.createWallet();
    const connection = new Connection("https://api.mainnet-beta.solana.com")
    const provider = new AnchorProvider(connection,wallet, {})

    const program = new Program<ProgramAuthorityTimelock>(IDL, TIMELOCK, provider)

    const programDataAccount = PublicKey.findProgramAddressSync(
        [PROGRAM_TO_TRANSFER.toBuffer()],
        BPF_UPGRADABLE_LOADER
      )[0];

    const escrowAuthority = PublicKey.findProgramAddressSync(
        [NEW_AUTHORITY.toBuffer(), TIMESTAMP.toBuffer("be", 8)],
        TIMELOCK
    )[0]

    await program.methods.transfer(TIMESTAMP).accounts({
        newAuthority: NEW_AUTHORITY,
        programAccount : PROGRAM_TO_TRANSFER,
        escrowAuthority: escrowAuthority,
        programData: programDataAccount,
        bpfUpgradableLoader : BPF_UPGRADABLE_LOADER
    }).rpc({skipPreflight: true})
}

main();