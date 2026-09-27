# Pinocchio Vault

A minimal native SOL vault built with Pinocchio. Each owner uses a PDA derived
from `["vault", owner]`.

## Instructions

- `0 - Deposit`: transfers a non-zero lamport amount from the signing owner to
  an empty vault PDA.
- `1 - Withdraw`: transfers the vault's full balance back to its signing owner.

The deposit payload is an eight-byte little-endian `u64`. Both instructions
expect the owner followed by the vault account.

## Build

```bash
cargo build-sbf
```

This is educational code and has not been audited.
