# Changelog

All notable changes to Soroban Forge will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- `soroban-forge new --var NAME=VALUE` (repeatable) plus support for a
  per-template `template.toml` manifest declaring custom variables. Missing
  values are prompted for when the session is interactive; non-interactive runs
  fall back to the declared default or fail with a message naming the flag
- `soroban-forge test-init --budget [ENTRYPOINT]` — generates
  `tests/forge_budget.rs`, a benchmark that measures one entrypoint with
  `env.cost_estimate().budget()` and asserts CPU-instruction and memory
  ceilings
- `soroban-forge test-init` now emits `tests/forge_init_once.rs` whenever the
  contract exposes an initialize-style entrypoint, asserting a second call is
  rejected

### Changed
- `soroban-forge new --force` asks for confirmation before overwriting an
  existing directory. `--yes`, `--json` and non-interactive sessions skip the
  question, so scripts and CI are unaffected
- The `escrow`, `governance` and `vesting` templates now reject a second
  `initialize` call instead of silently resetting their state

- `soroban-forge verify <contract-id>` — compares the wasm hash of a deployed
  contract (fetched with `stellar contract fetch`) against the local release
  build, reporting match/mismatch as text or `--json`. Exits `1` on a
  mismatch so CI can gate on it
- Global `--quiet`/`-q` mode for suppressing successful command output while
  preserving errors and exit codes
- Comprehensive documentation suite
- Example contracts for common DeFi patterns
- CI pipeline with fmt, clippy, test, and WASM build steps
