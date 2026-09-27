# Blueshift Solana Challenges

A collection of small Solana programs built while working through Blueshift
challenges. The repository explores the same vault and escrow ideas with both
[Anchor](https://www.anchor-lang.com/) and
[Pinocchio](https://github.com/anza-xyz/pinocchio).

## Projects

- [`blueshift_vault`](./blueshift_vault) — a compact Pinocchio SOL vault with
  deposit and withdraw instructions.
- [`blueshit_pinnochio_vault/blueshift_vault`](./blueshit_pinnochio_vault/blueshift_vault)
  — a modular version of the Pinocchio vault.
- [`blueshit_anchor_vault`](./blueshit_anchor_vault) — an Anchor SOL vault
  backed by a per-user PDA.
- [`blueshift_anchor_escrow`](./blueshift_anchor_escrow) — an Anchor token
  escrow with make, take, and refund flows.
- [`anchor_escrow/blueshift_anchor_escrow`](./anchor_escrow/blueshift_anchor_escrow)
  — an Anchor escrow variant using explicit one-byte instruction
  discriminators.
- [`blueshit_anchor_escrow`](./blueshit_anchor_escrow) — a minimal generated
  Anchor starter program.
- [`blueshit_pinnochio_vault/blueshift_escrow`](./blueshit_pinnochio_vault/blueshift_escrow)
  — an unfinished Pinocchio escrow parsing scaffold.

Directory names are kept as originally created, including the `blueshit` and
`pinnochio` spellings.

## Requirements

- Rust and Cargo
- Solana CLI
- Anchor CLI, Node.js, and Yarn for the Anchor projects

## Build

Run commands from the project you want to work on.

For a Pinocchio program:

```bash
cargo build-sbf
```

For an Anchor program:

```bash
yarn install
anchor build
anchor test
```

Each project README describes its implemented instructions and current state.

## Disclaimer

These programs are learning exercises. They have not been audited and should
not be used to hold real funds.
