# Modular Pinocchio Vault

A Pinocchio native SOL vault. It implements the same per-owner PDA flow as the
top-level vault while keeping instruction parsing and processing in a separate
module.

## Instructions

- `0 - Deposit`: transfers a non-zero lamport amount into the empty vault PDA.
- `1 - Withdraw`: returns the vault's entire balance to its signing owner.

The vault address is derived from `["vault", owner]`. The deposit payload is an
eight-byte little-endian `u64`.

## Build

```bash
cargo build-sbf
```

This is educational code and has not been audited.
