# CLI Reference

## Global options

- `--quiet`, `-q` — suppress informational command output; errors and exit
  codes are unchanged.
- `--verbose`, `-v` — enable debug logging.
- `--log-file <path>` — also write JSON-lines structured logs to a file while
  preserving normal terminal output.
- `--offline` — prohibit network access. Network-dependent operations fail with
  a clear message, while `doctor` skips its connectivity probe.

Global options may appear before or after a subcommand and can be combined.

## Commands

- `soroban-forge new <name> --template <t>` — create a contract project.
- `soroban-forge init [--tests] [--ci]` — add `forge.toml` to an existing
  contract without replacing project files; optionally add test and CI
  scaffolding.
- `soroban-forge templates` — list all bundled contract templates with descriptions.
- `soroban-forge test-init [--layout <tests|inline>]` — generate a test harness.
  `--layout tests` (default) writes a `tests/` integration-test directory;
  `--layout inline` writes a single `#[cfg(test)] mod forge_tests` in `src/`.
  Contracts that use persistent storage also get `forge_ttl.rs`, which
  exercises `extend_ttl` on a persistent entry.
- `soroban-forge ci-init --provider <github|gitlab|circleci|bitbucket>` —
  generate CI workflows. `--matrix` adds a build/test workflow that runs across
  a Rust toolchain matrix (stable plus `--msrv`, default 1.84).
- `soroban-forge doctor [--json]` — check the local Soroban toolchain (optionally emitting machine-readable JSON).
- `soroban-forge bindings ts` — generate a TypeScript client package from the built contract wasm.
- `soroban-forge verify <contract-id> [--network <n>]` — compare a deployed
  contract's wasm hash with the local release build; exits `1` on a mismatch.
  See [Contract Verification](contract-verification.md).
