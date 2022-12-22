import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { ProgramAuthorityEscrow } from "../target/types/program_authority_escrow";
import {PublicKey } from "@solana/web3.js"

describe("program-authority-escrow", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.ProgramAuthorityEscrow as Program<ProgramAuthorityEscrow>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.propose().accounts( {currentAuthority : new PublicKey(0), newAuthority : new PublicKey(0), program : new PublicKey(0), bpfUpgradableLoader : new PublicKey(0)} ).rpc();
    console.log("Your transaction signature", tx);
  });
});
