# Changelog

All notable changes to Soroban Forge will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
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
- `soroban-forge verify <contract-id>` — compares the wasm hash of a deployed
  contract (fetched with `stellar contract fetch`) against the local release
  build, reporting match/mismatch as text or `--json`. Exits `1` on a
  mismatch so CI can gate on it
- Global `--quiet`/`-q` mode for suppressing successful command output while
  preserving errors and exit codes
- Comprehensive documentation suite
- Example contracts for common DeFi patterns
- CI pipeline with fmt, clippy, test, and WASM build steps
