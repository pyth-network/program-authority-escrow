# program-authority-escrow

A stateless Solana program for safe program authority transfers.

The current authority calls `propose` to transfer authority to an escrow PDA seeded by `(current_authority, new_authority)`. From there:
- The current authority can call `revert` to reclaim authority
- The new authority can call `accept` to complete the transfer

This ensures the new authority has signed before accepting, making accidental transfers to wrong keys reversible.

Build with `cargo build-sbf` and test with `cargo test-sbf`.

## Tutorial: Transfer Program Authority to a Squads Multisig

This tutorial walks through transferring a Solana program's upgrade authority from a single keypair to a Squads v3 multisig vault.

### Prerequisites

Install dependencies:

```shell
pnpm install
```

You'll need:
- The current program authority keypair (file path or Ledger hardware wallet)
- Your program's address
- Your Squads v3 multisig address
- A multisig member keypair to create and approve the proposal

### Step 1: Propose the Transfer

The current authority proposes transferring the program to the escrow. This moves authority from your keypair to a temporary escrow PDA, where it waits for the new authority to accept.

```shell
pnpm propose \
  --keypair /path/to/current-authority.json \
  --program <PROGRAM_ADDRESS> \
  --authority <MULTISIG_VAULT_ADDRESS>
```

The `--authority` should be the multisig vault PDA that will become the new program authority. You can find this address in the Squads UI or derive it using vault index 1 (the default).

If using a Ledger:

```shell
pnpm propose \
  --keypair ledger \
  --derivation-path 0/0 \
  --program <PROGRAM_ADDRESS> \
  --authority <MULTISIG_VAULT_ADDRESS>
```

### Step 2: Accept via Multisig Proposal

The new authority (the multisig vault) must accept the transfer. Since a vault can't sign directly, this creates a Squads proposal that members vote on.

```shell
pnpm accept \
  --keypair /path/to/multisig-member.json \
  --program <PROGRAM_ADDRESS> \
  --authority <PREVIOUS_AUTHORITY_ADDRESS> \
  --multisig <MULTISIG_ADDRESS>
```

Here:
- `--keypair` is a multisig member who will create and initially approve the proposal
- `--authority` is the previous authority that proposed the transfer (from Step 1)
- `--multisig` is the Squads multisig address (not the vault)

This command:
1. Creates a new transaction in the multisig
2. Adds the accept instruction
3. Activates the transaction
4. Casts the first approval vote

### Step 3: Complete Multisig Approval

Other multisig members must now approve the proposal in the Squads UI until it reaches the threshold. Once approved and executed, the program authority transfer is complete.

### Reverting a Transfer

If you need to cancel before the new authority accepts, the current authority can revert:

```shell
pnpm revert \
  --keypair /path/to/current-authority.json \
  --program <PROGRAM_ADDRESS> \
  --authority <NEW_AUTHORITY_ADDRESS>
```

If the current authority is already a multisig vault:

```shell
pnpm revert \
  --keypair /path/to/multisig-member.json \
  --program <PROGRAM_ADDRESS> \
  --authority <NEW_AUTHORITY_ADDRESS> \
  --multisig <MULTISIG_ADDRESS>
```

### Advanced Options

All scripts support these additional options:

| Option | Description |
|--------|-------------|
| `-u, --url <rpc_url>` | RPC endpoint (default: `https://api.mainnet-beta.solana.com`) |
| `-i, --authority-index <n>` | Multisig vault index (default: 1) |
| `-M, --multisig-program <address>` | Custom Squads v3 program address |
| `-d, --derivation-path <path>` | Ledger derivation path as `account/change` |

Run any script without arguments to see full usage:

```shell
pnpm propose
pnpm accept
pnpm revert
```
