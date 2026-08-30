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
- `crowdfund` — escrow/deadline crowdfunding example
- `escrow` — token escrow with approval or timeout-based refund
- `faucet` — token faucet with per-address cooldown
- `governance` — DAO governance with weighted voting and quorum
- `lottery` — randomized lottery with ticket purchases and prize draws
- `merkle-airdrop` — one-claim-per-address airdrop verified against a merkle root
- `multisig` — M-of-N multisig account contract
- `payment-splitter` — splits received funds between payees by fixed shares
- `staking` — proportional reward staking with O(1) `acc_reward_per_share` accumulator
- `streaming` — streams tokens linearly over time with cancel support
- `subscription` — recurring payment charged once per elapsed interval
- `vesting` — token vesting with cliff + linear release schedule
- `wrapped-asset` — 1:1 wrapper token minted on deposit, burned on withdraw
- `english-auction` — ascending-bid auction with seller settlement
- `flash-loan` — uncollateralized single-transaction loan repaid via a borrower callback

Manifests are shipped as `Cargo.toml.hbs` so cargo doesn't treat these
directories as packages; the `.hbs` suffix is stripped when a project is
generated.

Each template also carries a `template.toml` (description, any extra
prompted variables, post-generate hints) — see
[`crates/scaffold`](../crates/scaffold) for the format. It is metadata only
and is never copied into the generated project.
