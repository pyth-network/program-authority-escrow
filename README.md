# program-authority-escrow

A stateless Solana program for safe program authority transfers.

The current authority calls `propose` to transfer authority to an escrow PDA seeded by `(current_authority, new_authority)`. From there:
- The current authority can call `revert` to reclaim authority
- The new authority can call `accept` to complete the transfer

This ensures the new authority has signed before accepting, making accidental transfers to wrong keys reversible.

Build with `cargo build-sbf` and test with `cargo test-sbf`.

## Scripts

TypeScript scripts are provided to interact with the on-chain program. All scripts support:
- File-based keypairs or Ledger hardware wallets
- Squads v3 multisig proposals via `--multisig`

Install dependencies with `yarn install`. See `scripts/helpers.ts` for documentation on CLI arguments.

### Propose

Transfer program authority to the escrow. The current authority proposes the transfer:

```shell
yarn propose --keypair <path|ledger> --program <program_address> --authority <new_authority>
```

### Accept

Accept a proposed authority transfer. The new authority accepts:

```shell
yarn accept --keypair <path|ledger> --program <program_address> --authority <previous_authority>
```

### Revert

Revert a proposed transfer before it's accepted. The current authority reverts:

```shell
yarn revert --keypair <path|ledger> --program <program_address> --authority <new_authority>
```
