# Changelog

All notable changes to Soroban Forge will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- `soroban-forge spec` — prints the built contract's interface (every
  entrypoint with its argument and return types, plus the types they refer to)
  by wrapping `stellar contract info interface`; `--json` emits the
  machine-readable spec
- Three new templates: `payment-splitter` (distributes received funds to
  payees by fixed shares), `subscription` (recurring payment charged once per
  elapsed interval) and `merkle-airdrop` (one claim per eligible address,
  verified against an on-chain merkle root)
- `soroban-forge verify <contract-id>` — compares the wasm hash of a deployed
  contract (fetched with `stellar contract fetch`) against the local release
  build, reporting match/mismatch as text or `--json`. Exits `1` on a
  mismatch so CI can gate on it
- Global `--quiet`/`-q` mode for suppressing successful command output while
  preserving errors and exit codes
- Comprehensive documentation suite
- Example contracts for common DeFi patterns
- CI pipeline with fmt, clippy, test, and WASM build steps
