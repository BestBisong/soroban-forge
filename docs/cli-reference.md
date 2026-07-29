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
  - `--force` — overwrite an existing target directory. In a terminal this asks
    for confirmation first; `--yes`, `--json` and non-interactive sessions
    proceed without asking. Without `--force`, an existing directory aborts
    with exit code `1`.
  - `--var NAME=VALUE` (repeatable) — supply a variable declared in the
    template's `template.toml`. Anything still missing is prompted for in a
    terminal; otherwise the declared default is used, or the run fails.
    See [Templates](templates.md).
- `soroban-forge init [--tests] [--ci]` — add `forge.toml` to an existing
  contract without replacing project files; optionally add test and CI
  scaffolding.
- `soroban-forge templates` — list all bundled contract templates with descriptions.
- `soroban-forge test-init` — generate a test harness. A contract with an
  initialize-style entrypoint also gets `tests/forge_init_once.rs`, asserting
  the entrypoint refuses a second call.
  - `--budget [ENTRYPOINT]` — also emit `tests/forge_budget.rs`, which measures
    one entrypoint's CPU instructions and memory with
    `env.cost_estimate().budget()` and asserts an upper bound. Defaults to the
    first detected entrypoint.
- `soroban-forge ci-init --provider github` — generate CI workflows.
- `soroban-forge doctor [--json]` — check the local Soroban toolchain (optionally emitting machine-readable JSON).
- `soroban-forge bindings ts` — generate a TypeScript client package from the built contract wasm.
- `soroban-forge verify <contract-id> [--network <n>]` — compare a deployed
  contract's wasm hash with the local release build; exits `1` on a mismatch.
  See [Contract Verification](contract-verification.md).
