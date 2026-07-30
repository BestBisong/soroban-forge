# Merkle Airdrop Example

Distribute tokens to a large allowlist using a Merkle proof, storing only the root on-chain.

```sh
soroban-forge new my-airdrop --template merkle-airdrop
```

## How It Works

1. Off-chain: build a Merkle tree of `(address, amount)` pairs
2. Store only the Merkle root in the contract
3. Claimants submit a proof — the contract verifies and transfers

## Key Functions

| Function | Description |
|----------|-------------|
| `initialize(admin, token, root)` | Store the Merkle root and the token |
| `fund(amount)` | Admin deposits the tokens to be claimed |
| `claim(claimant, amount, proof)` | Claim once against the current root |
| `verify(claimant, amount, proof)` | Read-only eligibility check |
| `set_root(root)` | Admin replaces the Merkle root |

Leaves are `sha256(xdr(address) || be_bytes(amount))` and pairs are hashed in
sorted order, so proofs carry sibling hashes only. See
[docs/templates.md](../../docs/templates.md#merkle-airdrop).
