# Quickstart — 5 Minutes to Your First Soroban Contract

## Prerequisites

Before you begin, make sure the following are installed:

- **Rust toolchain** — [Install via rustup](https://rustup.rs/)
- **`wasm32-unknown-unknown` target** — `rustup target add wasm32-unknown-unknown`
- **Stellar CLI** (≥ 21.0) — [Install guide](https://soroban.stellar.org/docs/getting-started/setup)

You can verify your environment is ready by running:

```sh
soroban-forge doctor
```

This checks that all required tools are installed and correctly configured.

## Steps

```sh
# 1. Install Soroban Forge
npm install -g soroban-forge

# 2. Scaffold a project
forge init hello-soroban --template token
cd hello-soroban

# 3. Build
forge build

# 4. Test
forge test

# 5. Deploy to testnet
forge deploy --network testnet
```

That's it. Your contract is live on Stellar Testnet.
