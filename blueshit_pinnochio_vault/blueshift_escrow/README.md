# Pinocchio Escrow Scaffold

An unfinished Pinocchio token-escrow exercise. It defines instruction and
account parsers for make, take, and refund, but the entrypoint currently does
not execute token transfers or persist escrow state.

## Instruction layout

- `0 - Make`: expects `seed`, `receive`, and `amount` as three little-endian
  `u64` values.
- `1 - Take`: parses the accounts needed to accept an escrow.
- `2 - Refund`: parses the accounts needed to cancel an escrow.

## Build

```bash
cargo build-sbf
```

Do not use this program with real funds; the escrow flow is incomplete.
