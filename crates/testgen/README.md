# soroban-forge-testgen (Module 3)

Test harness generator. **Owner: Person C.**

Implements the `soroban-forge test-init` subcommand: point it at an existing
Soroban contract project and it generates

| file                   | contents                                                             |
|------------------------|----------------------------------------------------------------------|
| `tests/common/mod.rs`  | fixtures: mocked-auth `Env`, account generator, ledger-time control, token (SAC) setup + funding, snapshot assertion helper |
| `tests/forge_smoke.rs` | smoke test registering the detected `#[contract]` type and constructing its client |
| `tests/forge_invariant.rs` | proptest-based invariant testing harness asserting state properties across random call sequences |
| `tests/forge_init_once.rs` | (when an initialize-style entrypoint is detected) asserts a second call to it is rejected |
| `tests/forge_budget.rs` | (with `--budget`) benchmark measuring one entrypoint's CPU instructions and memory via `env.cost_estimate().budget()`, asserting an upper bound |
| `tests/forge_upgrade.rs` | (when an upgrade entrypoint is detected) writes state, upgrades the contract, and asserts the state survived |
| `fuzz/Cargo.toml`      | (with `--fuzz`) cargo-fuzz workspace manifest |
| `fuzz/fuzz_targets/fuzz_target_1.rs` | (with `--fuzz`) property-based fuzzer feeding arbitrary values into detected contract methods |

Pass `--prop` (or `--invariant`/`--property`) to `test-init` to generate the
property-based invariant harness, and `--fuzz` to emit a cargo-fuzz target.

`--budget [ENTRYPOINT]` (alias `--bench`) emits the budget test, measuring the
named entrypoint or the first one detected. It starts at Soroban's
per-transaction ceilings (100M CPU instructions, 40 MiB); run
`cargo test --test forge_budget -- --nocapture` to see the real cost and
tighten the constants so a regression fails the test.

`--contract <name>` targets one member of a multi-contract workspace, matched by
package name, crate name or directory. A workspace with more than one contract
and no `--contract` stops and lists the candidates rather than generating a
harness for every member. Single-contract projects are unaffected.

`tests/forge_upgrade.rs` needs no flag. It is written when the contract exposes
an upgrade entrypoint — one named `upgrade`, `upgrade_contract`, `set_wasm` or
`migrate`, or any method taking a `BytesN<32>` argument whose name mentions
wasm. It writes state, upgrades, and asserts the state survived. The test ships
`#[ignore]`d because a real migration test needs a second wasm; the generated
file documents how to point it at one.

`tests/forge_init_once.rs` needs no flag: it is written whenever the contract
exposes an entrypoint named `initialize`, `initialise`, `init` or `setup`. It
calls the entrypoint, then asserts `try_<entrypoint>` returns `Err` on a second
call. A failure means the contract has no re-initialization guard — a real
finding, not a flaky test.

The global `--quiet` flag suppresses the generated-file report and follow-up
notes without changing which harness files are written.

## How detection works

`detect.rs` inspects the target without heavy parsing:

- `Cargo.toml` → package name, whether dev-dependencies enable soroban-sdk's
  `testutils` feature (warns if not).
- `src/lib.rs` → the struct annotated with exactly `#[contract]`, and whether
  a `__constructor` exists. Contracts with constructors get an `#[ignore]`d
  smoke test with a TODO, since constructor arguments can't be guessed. Also 
  detects method names and their parameters (ignoring `env: Env`) to construct
  `FuzzInput` enums when generating the fuzzer.

## Snapshot helper

`assert_snapshot(name, &value)` compares `value`'s `Debug` output against
`tests/snapshots/<name>.snap`. First run writes the snapshot; subsequent runs
fail on change; `FORGE_UPDATE_SNAPSHOTS=1 cargo test` accepts changes.

## Public surface

```rust
testgen::generate(dir, force, fuzz) -> Result<(ContractInfo, Vec<&str>)>;
testgen::generate_with(dir, &GenerateOptions) -> Result<(ContractInfo, Vec<&str>)>;
testgen::inspect(dir) -> Result<ContractInfo>;
testgen::build_budget_test(&info, entrypoint) -> Result<String>;
testgen::build_init_once_test(&info) -> String;
testgen::build_upgrade_test(&info) -> String;
testgen::upgrade::detect_upgrade_entrypoint(&methods) -> Option<UpgradeEntrypoint>;
testgen::candidates(root, &members) -> Vec<Candidate>;
testgen::resolve(requested, &candidates) -> Result<Selection>;
testgen::detect::detect_init_method(&methods) -> Option<MethodInfo>;
```

`GenerateOptions` carries `force`, `fuzz`, `budget` and `budget_entrypoint`;
`generate` is the two-flag shorthand for it.

## Tests

`cargo test -p soroban-forge-testgen` — includes end-to-end tests that run the
generator against freshly scaffolded `hello-world` and `token` projects.
