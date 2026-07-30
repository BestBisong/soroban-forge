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
- `soroban-forge test-init` — generate a test harness.
- `soroban-forge ci-init --provider github [--dependabot]` — generate CI
  workflows (build+test and a rustfmt/clippy lint job); `--dependabot` also
  writes `.github/dependabot.yml` for weekly cargo and github-actions updates.
- `soroban-forge doctor [--json]` — check the local Soroban toolchain (optionally emitting machine-readable JSON).
- `soroban-forge bindings ts` — generate a TypeScript client package from the built contract wasm.
- `soroban-forge verify <contract-id> [--network <n>]` — compare a deployed
  contract's wasm hash with the local release build; exits `1` on a mismatch.
  See [Contract Verification](contract-verification.md).
