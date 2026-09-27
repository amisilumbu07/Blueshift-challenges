# Anchor Token Escrow with Custom Discriminators

An Anchor token-swap escrow using explicit one-byte instruction
discriminators: `0` for make, `1` for take, and `2` for refund.

## Instructions

- `make(seed, receive, amount)`: creates an escrow and locks the maker's token A.
- `take()`: completes the exchange between the taker and maker.
- `refund()`: cancels the escrow and returns the locked tokens to the maker.

The escrow account stores the maker, token mints, requested amount, seed, and
PDA bump.

## Build and test

```bash
yarn install
anchor build
anchor test
```

The checked-in program ID is a placeholder. Replace it consistently before
deployment. This is educational code and has not been audited.
