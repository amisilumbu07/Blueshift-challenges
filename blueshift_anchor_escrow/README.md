# Anchor Token Escrow

An Anchor token-swap escrow. A maker locks token A in a vault and requests a
fixed amount of token B from a taker.

## Instructions

- `make(seed, receive, amount)`: creates the escrow state and deposits token A.
- `take()`: exchanges the taker's token B for the maker's locked token A.
- `refund()`: lets the maker cancel and recover the locked token A.

Escrow state records the maker, both token mints, the requested amount, the
seed, and the PDA bump.

## Build and test

```bash
yarn install
anchor build
anchor test
```

The checked-in program ID is a placeholder. Replace it consistently before
deployment. This is educational code and has not been audited.
