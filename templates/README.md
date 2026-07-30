# Contract templates

Each subdirectory is a template consumable by `soroban-forge new --template <name>`.
Owned by Module 2 (scaffolding) — see [`crates/scaffold`](../crates/scaffold)
for the template format and how to add a new one.

- `hello-world` — minimal greeter contract
- `nft` — non-fungible token with metadata, minting and burning
- `token` — SEP-41 fungible token (`soroban_sdk::token::TokenInterface`)
- `crowdfund` — escrow/deadline crowdfunding example
- `upgradeable` — admin-gated upgradeable contract (`update_current_contract_wasm`)
- `escrow` — token escrow with approval or timeout-based refund
- `vesting` — token vesting with cliff + linear release schedule
- `payment-splitter` — splits received funds between payees by fixed shares
- `subscription` — recurring payment charged once per elapsed interval
- `merkle-airdrop` — one-claim-per-address airdrop verified against a merkle root
- `amm` — constant-product AMM / liquidity pool
- `atomic-swap` — atomic two-party token swap
- `governance` — DAO governance with weighted voting and quorum
- `multisig` — M-of-N multisig account contract

Manifests are shipped as `Cargo.toml.hbs` so cargo doesn't treat these
directories as packages; the `.hbs` suffix is stripped when a project is
generated.

Each template also carries a `template.toml` (description, any extra
prompted variables, post-generate hints) — see
[`crates/scaffold`](../crates/scaffold) for the format. It is metadata only
and is never copied into the generated project.
