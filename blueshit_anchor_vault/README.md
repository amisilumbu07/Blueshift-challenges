# Anchor Vault

An Anchor program for depositing native SOL into a per-user vault PDA and
withdrawing the complete balance.

## Instructions

- `deposit(amount)`: requires an empty vault and transfers more than the
  rent-exempt minimum from the user.
- `withdraw()`: signs with the vault PDA and returns every lamport to the user.

The vault PDA uses `["vault", user]` as its seeds.

## Build and test

```bash
yarn install
anchor build
anchor test
```

This is educational code and has not been audited.
