# Changelog

All notable changes to Soroban Forge will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- `test-init` now generates `tests/forge_ttl.rs` for contracts that use
  persistent storage: tests that bump an entry with `extend_ttl` and assert it
  outlives a ledger advance, as a starting point for rent regressions
- `test-init --layout <tests|inline>` — choose between the `tests/`
  integration-test directory (default) and a single `#[cfg(test)] mod
  forge_tests` inside `src/`
- `ci-init --provider bitbucket` — generates `bitbucket-pipelines.yml`
  mirroring the GitHub build-test preset
- `ci-init --matrix [--msrv <version>]` — a build/test workflow that runs the
  job once per Rust toolchain (stable plus a pinned MSRV)
- `ci-init --provider github` now emits a `lint` job in `build-test.yml` running
  `cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings`, so
  generated CI fails on formatting or clippy violations
- `ci-init --dependabot` writes `.github/dependabot.yml` with weekly update
  schedules for the `cargo` and `github-actions` ecosystems
- `doctor` reports whether Docker is installed and its daemon is running
  (a warning when absent — it is only needed for reproducible wasm builds)
- `doctor` warns when git's `user.name`/`user.email` are unset, printing the
  `git config` commands to set them, since commits in a freshly created
  project otherwise fail confusingly
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
