# Follow-up issues for v0.2+

Well-scoped starter work for contributors, grouped by module. Difficulty tags:
**trivial** (good first issue, < 1 day) · **medium** (a few days) ·
**high** (design work involved). Claim an issue by commenting on it in the
GitHub tracker.

Entries 1–20 shipped in v0.1 and their issues are closed; they are kept
here for reference. Entries 21–120 are the open backlog and each links to
its GitHub issue.

## Module 1 — CLI core (`crates/core`)

1. **[trivial] Add `--quiet` global flag** — suppress non-error output;
   thread it through `ForgeContext` next to `verbose`.
2. **[medium] Add `soroban-forge config` subcommand** — print the resolved
   `forge.toml` (with defaults filled in) and warn on unknown keys.
3. **[medium] Structured output mode** — a global `--json` flag plugins can
   consult via `ForgeContext`, emitting machine-readable results (needed for
   editor integrations).
4. **[high] Dynamic plugin discovery** — investigate loading external
   subcommands `soroban-forge-<name>` from PATH (cargo-style), so third-party
   plugins don't need to be compiled in.
21. **[trivial] core: add a global -C/--cwd flag to run against another directory**
    ([#193](https://github.com/soroban-forge-labs/soroban-forge/issues/193)) —
    Add a global `-C <path>` / `--cwd <path>` flag that makes every subcommand
    resolve `forge.toml` and project paths relative to that directory instead
    of the process working directory.
22. **[medium] core: add SOROBAN_FORGE_* environment overrides for global flags**
    ([#194](https://github.com/soroban-forge-labs/soroban-forge/issues/194)) —
    Let global flags be set from the environment (`SOROBAN_FORGE_OFFLINE`,
    `SOROBAN_FORGE_LOG_FILE`, `SOROBAN_FORGE_VERBOSE`, ...) with precedence:
    CLI flag > env var > `forge.toml` > default.
23. **[trivial] core: suggest the closest subcommand on a typo (did-you-mean)**
    ([#195](https://github.com/soroban-forge-labs/soroban-forge/issues/195)) —
    When an unknown subcommand is given, print the nearest registered plugin
    name by edit distance instead of only the usage block.
24. **[trivial] core: warn on unknown forge.toml keys with a did-you-mean hint**
    ([#196](https://github.com/soroban-forge-labs/soroban-forge/issues/196)) —
    Unknown keys in `forge.toml` are silently ignored today. Warn about each
    one and suggest the closest valid key.
25. **[medium] core: resolve forge.toml from parent directories**
    ([#197](https://github.com/soroban-forge-labs/soroban-forge/issues/197)) —
    Search upward from the working directory for `forge.toml` (stopping at the
    filesystem or repository root) so commands work from subdirectories of a
    project.
26. **[medium] core: add a [defaults] table in forge.toml for per-command flags**
    ([#198](https://github.com/soroban-forge-labs/soroban-forge/issues/198)) —
    Allow projects to pin default flag values per subcommand, e.g.
    `[defaults.test-init] layout = "inline"` or `[defaults.ci-init] provider =
    "gitlab"`.
27. **[medium] core: give every ForgeError a stable code and docs URL**
    ([#199](https://github.com/soroban-forge-labs/soroban-forge/issues/199)) —
    Attach a stable identifier (e.g. `FORGE_E0007`) and a documentation anchor
    to each `ForgeError` variant so errors can be looked up and matched by
    scripts.
28. **[medium] core: add a global --timeout for network-capable operations**
    ([#200](https://github.com/soroban-forge-labs/soroban-forge/issues/200)) —
    Add a `--timeout <seconds>` global flag bounding any network call
    (template clones, connectivity checks, friendbot, verification fetches) so
    hung requests cannot stall CI.
29. **[medium] core: redact secrets and identity keys from --log-file output**
    ([#201](https://github.com/soroban-forge-labs/soroban-forge/issues/201)) —
    The structured JSON log can capture command arguments; make sure secret
    keys, seed phrases and token-like values are redacted before they are
    written.
30. **[trivial] core: add --log-level to filter structured log-file records**
    ([#202](https://github.com/soroban-forge-labs/soroban-forge/issues/202)) —
    `--log-file` currently writes every record. Add `--log-level
    <error|warn|info|debug|trace>` to filter what is persisted, independent of
    terminal verbosity.
31. **[high] core: add pre/post command hooks to the ForgePlugin trait**
    ([#203](https://github.com/soroban-forge-labs/soroban-forge/issues/203)) —
    Design optional `before_run`/`after_run` hooks (or a middleware chain) in
    `crates/core` so cross-cutting concerns — timing, telemetry-free
    diagnostics, cleanup on failure — live in one place rather than in every
    plugin.
32. **[medium] core: centralize progress reporting (spinner/steps) in ForgeContext**
    ([#204](https://github.com/soroban-forge-labs/soroban-forge/issues/204)) —
    Plugins print progress ad hoc today. Provide a small reporter on
    `ForgeContext` that honours `--quiet`, `--json` and non-TTY output in one
    place.
33. **[trivial] core: add subcommand aliases (new/n, test-init/ti, doctor/dr)**
    ([#205](https://github.com/soroban-forge-labs/soroban-forge/issues/205)) —
    Register short clap aliases for the most-used subcommands to cut typing in
    day-to-day use.
34. **[trivial] core: unit-test exit-code mapping for every ForgeError variant**
    ([#206](https://github.com/soroban-forge-labs/soroban-forge/issues/206)) —
    Exit codes are contractual (0 ok / 1 user / 2 missing tool / 3 internal)
    but not exhaustively tested. Add a test that fails when a new variant is
    added without a deliberate mapping.
35. **[medium] core: wire the network and optimize plugins into src/main.rs**
    ([#207](https://github.com/soroban-forge-labs/soroban-forge/issues/207)) —
    `crates/network` and `crates/optimize` exist and build, but neither is
    registered in the plugin vector in `src/main.rs`, so `soroban-forge
    network` and `soroban-forge optimize` are unreachable from the binary.

## Module 2 — Scaffolding & templates (`crates/scaffold`, `templates/`)

5. **[trivial] Add `--edition` option** — let `new` generate edition 2024
   projects once the ecosystem settles on it.
6. **[medium] Add a `multisig` template** — an M-of-N account/authorization
   example based on the official custom-account example.
7. **[medium] `soroban-forge new --from <git-url>`** — scaffold from a remote
   template repository instead of a bundled one.
8. **[high] Template manifest (`template.toml`)** — per-template metadata
   (description, extra variables with prompts, post-generate hints) instead of
   the current convention-only format.
36. **[medium] Add an English-auction template**
    ([#208](https://github.com/soroban-forge-labs/soroban-forge/issues/208)) —
    A timed ascending-bid auction: bids must exceed the current high bid,
    losing bids are refundable, and settlement transfers the asset to the
    winner after the deadline.
37. **[medium] Add a Dutch-auction template**
    ([#209](https://github.com/soroban-forge-labs/soroban-forge/issues/209)) —
    A descending-price auction where the price decays linearly from a start
    price to a floor over a fixed window and the first buyer settles at the
    current price.
38. **[high] Add an NFT-marketplace template**
    ([#210](https://github.com/soroban-forge-labs/soroban-forge/issues/210)) —
    Listing, buying and cancelling NFT sales with a configurable fee to a
    treasury address, built on the existing `nft` template's interface.
39. **[high] Add a yield-vault (ERC4626-style) template**
    ([#211](https://github.com/soroban-forge-labs/soroban-forge/issues/211)) —
    A deposit/withdraw vault issuing shares proportional to assets held, with
    a documented rounding policy that always favours the vault.
40. **[medium] Add a timelock-controller template**
    ([#212](https://github.com/soroban-forge-labs/soroban-forge/issues/212)) —
    Queue a call, enforce a minimum delay, then execute or cancel it — the
    standard governance execution primitive, pairing with the existing
    `governance` template.
41. **[medium] Add a role-based access-control template**
    ([#213](https://github.com/soroban-forge-labs/soroban-forge/issues/213)) —
    Grant/revoke/has-role primitives with an admin role that can administer
    other roles, as a reusable starting point instead of ad-hoc owner checks.
42. **[trivial] Add a pausable / emergency-stop template**
    ([#214](https://github.com/soroban-forge-labs/soroban-forge/issues/214)) —
    A minimal circuit-breaker: an admin can pause and unpause, and guarded
    entrypoints reject calls while paused.
43. **[high] Add a prediction-market template**
    ([#215](https://github.com/soroban-forge-labs/soroban-forge/issues/215)) —
    Binary outcome market: stake on an outcome, resolve via a designated
    oracle address, then let winners claim a proportional share of the pool.
44. **[high] Add a flash-loan template**
    ([#216](https://github.com/soroban-forge-labs/soroban-forge/issues/216)) —
    Lend within a single transaction and require repayment plus fee before the
    call returns, demonstrating cross-contract callbacks and atomic failure.
45. **[medium] Add a cross-contract call example template**
    ([#217](https://github.com/soroban-forge-labs/soroban-forge/issues/217)) —
    A two-contract project showing client generation, invoking another
    contract, and propagating authorization across the call boundary.
46. **[medium] Add a storage-migration example template**
    ([#218](https://github.com/soroban-forge-labs/soroban-forge/issues/218)) —
    Demonstrate migrating a persistent storage layout from v1 to v2 behind a
    version marker, paired with the `upgradeable` template.
47. **[trivial] new: add --list-templates --json for editor integrations**
    ([#219](https://github.com/soroban-forge-labs/soroban-forge/issues/219)) —
    Emit the template list as JSON (name, description, declared variables) so
    editors and scripts can consume it.
48. **[trivial] new: scaffold a justfile with common project tasks**
    ([#220](https://github.com/soroban-forge-labs/soroban-forge/issues/220)) —
    Generate a `justfile` with `build`, `test`, `fmt`, `lint`, `deploy` and
    `size` recipes wrapping the commands from the template README.
49. **[trivial] new: add a --license flag that writes the chosen LICENSE file**
    ([#221](https://github.com/soroban-forge-labs/soroban-forge/issues/221)) —
    Let `new` write a LICENSE file and set the matching `license` field in the
    generated `Cargo.toml`.
50. **[medium] new: generate a devcontainer configuration**
    ([#222](https://github.com/soroban-forge-labs/soroban-forge/issues/222)) —
    Add an optional `.devcontainer/` with Rust, the `wasm32v1-none` target and
    `stellar-cli` preinstalled, so a scaffolded project is Codespaces-ready.
51. **[medium] scaffold: verify every bundled template builds and tests in CI**
    ([#223](https://github.com/soroban-forge-labs/soroban-forge/issues/223)) —
    Add a CI matrix job that scaffolds each bundled template, runs `stellar
    contract build` and `cargo test`, so a broken template cannot be merged.
52. **[high] scaffold: share common template files through a partials directory**
    ([#224](https://github.com/soroban-forge-labs/soroban-forge/issues/224)) —
    Every template duplicates its `.gitignore`, `rust-toolchain.toml`, README
    skeleton and release profile. Introduce shared partials that templates opt
    into, so a fix lands once.
53. **[trivial] new: add --no-tests to skip generated test files**
    ([#225](https://github.com/soroban-forge-labs/soroban-forge/issues/225)) —
    Allow scaffolding a template without its `tests/` directory for users who
    bring their own harness.

## Module 3 — Test harness generator (`crates/testgen`)

9. **[trivial] Detect multiple `#[contract]` structs** — currently only the
   first is used; generate one smoke test per contract.
10. **[medium] Fuzz-test generator** — `test-init --fuzz` emitting a
    `cargo-fuzz` target that feeds arbitrary values into contract methods.
11. **[medium] Parse constructor signatures** — read `__constructor` argument
    types and generate a smoke test with sensible default values instead of an
    `#[ignore]`d TODO.
12. **[high] Property-based invariant harness** — proptest-based generator
    asserting user-declared invariants (e.g. token supply conservation) across
    random call sequences.
54. **[medium] testgen: generate tests for cross-contract calls with mock contracts**
    ([#226](https://github.com/soroban-forge-labs/soroban-forge/issues/226)) —
    When a contract stores or takes another contract's address, generate a
    minimal mock contract and a test that wires it in, instead of leaving a
    TODO.
55. **[medium] testgen: generate authorization-tree assertions**
    ([#227](https://github.com/soroban-forge-labs/soroban-forge/issues/227)) —
    For entrypoints calling `require_auth`, generate assertions over the
    recorded authorization tree, not just a success/failure check.
56. **[trivial] testgen: generate deterministic ledger-time helpers**
    ([#228](https://github.com/soroban-forge-labs/soroban-forge/issues/228)) —
    Emit an `advance_ledger(&env, n)` helper that bumps sequence number and
    timestamp together, so time-dependent tests are written the same way
    everywhere.
57. **[medium] testgen: add --update-snapshots to regenerate golden state files**
    ([#229](https://github.com/soroban-forge-labs/soroban-forge/issues/229)) —
    Snapshot tests need a sanctioned way to accept intended changes rather
    than hand-editing golden files.
58. **[trivial] testgen: generate a multi-user fixture with N funded identities**
    ([#230](https://github.com/soroban-forge-labs/soroban-forge/issues/230)) —
    Most non-trivial tests need several actors. Generate a fixture producing N
    addresses with a documented naming convention (admin, alice, bob, ...).
59. **[medium] testgen: warn when a contract entrypoint has no generated coverage**
    ([#231](https://github.com/soroban-forge-labs/soroban-forge/issues/231)) —
    After generation, list entrypoints that no generated test touches so the
    gap is visible rather than assumed.
60. **[medium] testgen: generate tests for storage key collisions across storage types**
    ([#232](https://github.com/soroban-forge-labs/soroban-forge/issues/232)) —
    Generate a test asserting the same key in instance, persistent and
    temporary storage stays independent — a common source of subtle bugs.
61. **[medium] testgen: support --contract <name> to target one contract in a workspace**
    ([#233](https://github.com/soroban-forge-labs/soroban-forge/issues/233)) —
    In a multi-contract workspace, `test-init` should be able to target a
    single contract instead of guessing.
62. **[high] testgen: generate an upgrade/migration test for upgradeable contracts**
    ([#234](https://github.com/soroban-forge-labs/soroban-forge/issues/234)) —
    For contracts exposing an upgrade entrypoint, generate a test that deploys
    v1, writes state, upgrades to v2, and asserts state survived.
63. **[medium] testgen: emit criterion benchmarks alongside budget tests**
    ([#235](https://github.com/soroban-forge-labs/soroban-forge/issues/235)) —
    Budget tests assert ceilings; add optional criterion benches so cost
    changes can be tracked over time rather than only gated.
64. **[medium] testgen: generate round-trip tests for Option, Vec and Map arguments**
    ([#236](https://github.com/soroban-forge-labs/soroban-forge/issues/236)) —
    Entrypoints taking container types are frequently mis-encoded. Generate
    tests that pass empty, single and multi-element values through them.
65. **[trivial] testgen: add a --dry-run flag printing planned test files**
    ([#237](https://github.com/soroban-forge-labs/soroban-forge/issues/237)) —
    Print the files `test-init` would write, with their sizes, without
    touching the filesystem — matching `new --dry-run`.
66. **[trivial] testgen: make re-running test-init idempotent**
    ([#238](https://github.com/soroban-forge-labs/soroban-forge/issues/238)) —
    Re-running `test-init` should converge rather than accumulate duplicate or
    conflicting files.

## Module 4 — CI/CD presets (`crates/ci-presets`, `presets/`)

13. **[trivial] Pin action versions by SHA** — replace tag references in the
    GitHub presets with pinned commit SHAs plus a comment noting the tag.
14. **[medium] Add a GitLab CI preset** — `ci-init --provider gitlab` writing
    `.gitlab-ci.yml`; `output_dir()` already anticipates per-provider paths.
15. **[medium] Cache stellar-cli in the deploy workflow** — installing via
    the shell script on every run is slow; use a cached binary or a published
    action.
16. **[high] Release workflow preset** — tag-triggered workflow that builds
    the wasm, attaches it to a GitHub Release with checksums, and (optionally)
    verifies reproducibility.
67. **[medium] Add a Jenkins pipeline preset**
    ([#239](https://github.com/soroban-forge-labs/soroban-forge/issues/239)) —
    `ci-init --provider jenkins` writes a `Jenkinsfile` mirroring the GitHub
    build-test preset (toolchain setup, build, test, size check).
68. **[medium] Add a Buildkite pipeline preset**
    ([#240](https://github.com/soroban-forge-labs/soroban-forge/issues/240)) —
    `ci-init --provider buildkite` writes `.buildkite/pipeline.yml` with
    build, test and contract-size steps.
69. **[medium] ci-init: add --release to emit a tag-triggered release workflow**
    ([#241](https://github.com/soroban-forge-labs/soroban-forge/issues/241)) —
    Generate a workflow that builds the wasm on a tag, attaches it to a GitHub
    Release with checksums, and records the build inputs.
70. **[high] CI preset: reproducible wasm build check via Docker**
    ([#242](https://github.com/soroban-forge-labs/soroban-forge/issues/242)) —
    Add a job that rebuilds the contract in a pinned container and fails if
    the wasm hash differs from the host build, giving verification a CI
    counterpart.
71. **[medium] CI preset: run soroban-forge verify against a deployed contract**
    ([#243](https://github.com/soroban-forge-labs/soroban-forge/issues/243)) —
    A workflow that runs `soroban-forge verify <contract-id>` on a schedule or
    after deploy, so drift between the deployed contract and `main` is caught
    automatically.
72. **[trivial] ci-init: add a --path flag to write workflows outside the repository root**
    ([#244](https://github.com/soroban-forge-labs/soroban-forge/issues/244)) —
    Support monorepos where the contract lives in a subdirectory but workflows
    belong at the repository root.
73. **[medium] ci-init: add --max-size to fail the build on a wasm size budget**
    ([#245](https://github.com/soroban-forge-labs/soroban-forge/issues/245)) —
    The existing size check reports regressions; add a hard ceiling so a build
    fails when the wasm exceeds a configured byte budget.
74. **[trivial] CI preset: add a concurrency group to cancel superseded runs**
    ([#246](https://github.com/soroban-forge-labs/soroban-forge/issues/246)) —
    Generated GitHub workflows should cancel in-progress runs for the same ref
    to save CI minutes.
75. **[trivial] CI preset: emit an actionlint job validating generated workflows**
    ([#247](https://github.com/soroban-forge-labs/soroban-forge/issues/247)) —
    Add an opt-in job that runs actionlint over `.github/workflows`, so users
    editing generated workflows catch mistakes early.
76. **[high] ci-init: regenerate presets in place and show a diff**
    ([#248](https://github.com/soroban-forge-labs/soroban-forge/issues/248)) —
    Re-running `ci-init` overwrites or refuses. Add a mode that shows what
    would change against the current files and applies only accepted changes.
77. **[trivial] CI preset: add a stale issue/PR workflow**
    ([#249](https://github.com/soroban-forge-labs/soroban-forge/issues/249)) —
    An opt-in workflow that marks and closes inactive issues and pull requests
    on a schedule.
78. **[high] CI preset: sign release artifacts**
    ([#250](https://github.com/soroban-forge-labs/soroban-forge/issues/250)) —
    Extend the release workflow to sign the published wasm and checksum file
    so consumers can verify provenance.

## Module 5 — Docs, examples & DX (`crates/doctor`, `docs/`, `examples/`)

17. **[trivial] `doctor --json`** — machine-readable check output for use in
    scripts and editors.
18. **[trivial] Check soroban-sdk version in doctor** — when run inside a
    contract project, warn if the project's soroban-sdk is behind the version
    forge templates pin.
19. **[medium] Video/asciinema quickstart** — record the zero-to-testnet
    tutorial and embed it in the README.
20. **[high] `soroban-forge upgrade` guide + docs site** — document migrating
    generated projects across sdk majors; publish docs/ via GitHub Pages.
79. **[medium] doctor: check the wasm32v1-none target matches the active toolchain**
    ([#251](https://github.com/soroban-forge-labs/soroban-forge/issues/251)) —
    The target can be installed for a different toolchain than the one that
    will build, producing a confusing failure. Check the pairing, not just
    presence.
80. **[trivial] doctor: detect a Cargo.lock that is stale relative to Cargo.toml**
    ([#252](https://github.com/soroban-forge-labs/soroban-forge/issues/252)) —
    Warn when `Cargo.toml` has changed since `Cargo.lock` was updated, since
    it makes builds non-reproducible in surprising ways.
81. **[medium] doctor: check for known-broken stellar-cli versions**
    ([#253](https://github.com/soroban-forge-labs/soroban-forge/issues/253)) —
    Beyond the minimum version, maintain a small denylist of releases known to
    break forge workflows and warn when one is active.
82. **[medium] doctor: verify reachability of the RPC configured in forge.toml**
    ([#254](https://github.com/soroban-forge-labs/soroban-forge/issues/254)) —
    The connectivity check targets a default endpoint; check the project's
    configured network instead when one is set.
83. **[medium] doctor --fix: make every fix idempotent and re-runnable**
    ([#255](https://github.com/soroban-forge-labs/soroban-forge/issues/255)) —
    Re-running `--fix` after a partial failure should be safe and should skip
    anything already satisfied.
84. **[trivial] doctor: add --check <name> to run a single check**
    ([#256](https://github.com/soroban-forge-labs/soroban-forge/issues/256)) —
    Let users and CI run one named check instead of the whole suite.
85. **[medium] docs: add a plugin-authoring tutorial**
    ([#257](https://github.com/soroban-forge-labs/soroban-forge/issues/257)) —
    A start-to-finish walkthrough building a third-party `ForgePlugin`: crate
    layout, clap definition, error handling, exit codes and tests.
86. **[medium] docs: document every forge.toml key in a reference table**
    ([#258](https://github.com/soroban-forge-labs/soroban-forge/issues/258)) —
    A single table of every key with type, default, and the command that reads
    it — today the source is the reference.
87. **[medium] docs: add a template catalogue page**
    ([#259](https://github.com/soroban-forge-labs/soroban-forge/issues/259)) —
    One page listing all bundled templates with what each demonstrates, its
    entrypoints, and when to pick it — there are now twenty.
88. **[medium] docs: add a security-considerations page for generated contracts**
    ([#260](https://github.com/soroban-forge-labs/soroban-forge/issues/260)) —
    Document what the templates do and do not protect against: authorization,
    integer overflow, storage TTL expiry, upgrade authority, oracle trust.
89. **[trivial] docs: add a comparison page — soroban-forge vs stellar-cli alone**
    ([#261](https://github.com/soroban-forge-labs/soroban-forge/issues/261)) —
    Explain precisely what forge adds over the official CLI and what it
    delegates, so the wrapping-not-reimplementing boundary is obvious to
    newcomers.
90. **[medium] examples: regenerate examples in CI and fail on drift**
    ([#262](https://github.com/soroban-forge-labs/soroban-forge/issues/262)) —
    Checked-in example projects can silently fall behind the templates that
    produce them.
91. **[trivial] docs: add a cookbook of common CLI recipes**
    ([#263](https://github.com/soroban-forge-labs/soroban-forge/issues/263)) —
    Short copy-paste recipes: scaffold and deploy in one go, regenerate
    bindings after a change, verify a mainnet contract, wire forge into an
    existing repo.
92. **[trivial] docs: document the release process for maintainers**
    ([#264](https://github.com/soroban-forge-labs/soroban-forge/issues/264)) —
    Write down version bumping, changelog cutting, tagging, and what CI
    publishes, so releases are not tribal knowledge.
93. **[trivial] dx: add a CODE_OF_CONDUCT and issue/PR templates**
    ([#265](https://github.com/soroban-forge-labs/soroban-forge/issues/265)) —
    With 38 contributors the repository still has no code of conduct and no
    issue or PR templates.

## Module 6 — TypeScript bindings (`crates/binding-ts`)

94. **[medium] bindings ts: emit a publishable package.json with correct exports**
    ([#266](https://github.com/soroban-forge-labs/soroban-forge/issues/266)) —
    Generated bindings should be installable as-is: proper `exports`, `types`,
    `files` and peer dependency on the Stellar SDK.
95. **[trivial] bindings ts: add --out-dir and --package-name flags**
    ([#267](https://github.com/soroban-forge-labs/soroban-forge/issues/267)) —
    Let users choose where bindings are written and what the package is called
    instead of the derived defaults.
96. **[high] bindings ts: generate optional React hooks**
    ([#268](https://github.com/soroban-forge-labs/soroban-forge/issues/268)) —
    `bindings ts --react` additionally emits typed hooks wrapping each
    entrypoint, since most Soroban frontends are React.
97. **[medium] bindings ts: type-check generated bindings in CI**
    ([#269](https://github.com/soroban-forge-labs/soroban-forge/issues/269)) —
    Add a CI job that generates bindings for a template contract and runs `tsc
    --noEmit` against them, so binding regressions are caught.
98. **[high] bindings: add a Python bindings generator**
    ([#270](https://github.com/soroban-forge-labs/soroban-forge/issues/270)) —
    A `bindings py` counterpart emitting a typed Python client from the built
    wasm, for scripting and backend integrations.
99. **[medium] bindings ts: add --watch to regenerate on contract change**
    ([#271](https://github.com/soroban-forge-labs/soroban-forge/issues/271)) —
    Rebuild and regenerate bindings when the contract source changes, for a
    tight frontend development loop.

## Module 7 — Deployment verification (`crates/verify`)

100. **[medium] verify: support an explicit network passphrase and RPC endpoint**
     ([#272](https://github.com/soroban-forge-labs/soroban-forge/issues/272))
     — Verification currently assumes default network settings; allow
     targeting any network explicitly or via `forge.toml`.
101. **[trivial] verify: add --wasm <path> to compare an arbitrary wasm file**
     ([#273](https://github.com/soroban-forge-labs/soroban-forge/issues/273))
     — Compare a deployed contract against a specific wasm file instead of the
     local release build — useful for verifying a downloaded release artifact.
102. **[high] verify: reproducible-build mode using a pinned container**
     ([#274](https://github.com/soroban-forge-labs/soroban-forge/issues/274))
     — Build inside a pinned image before hashing, so verification does not
     depend on the local toolchain.
103. **[medium] verify: print a spec diff when hashes mismatch**
     ([#275](https://github.com/soroban-forge-labs/soroban-forge/issues/275))
     — A mismatch tells you nothing about what changed. Fetch both interfaces
     and summarise the differences.

## Module 8 — Contract interface dump (`crates/spec`)

104. **[medium] spec: render the interface as Markdown**
     ([#276](https://github.com/soroban-forge-labs/soroban-forge/issues/276))
     — `spec --format md` emits a documentation-ready table of entrypoints,
     arguments and return types for embedding in a README.
105. **[high] spec: diff two contract specs and flag breaking changes**
     ([#277](https://github.com/soroban-forge-labs/soroban-forge/issues/277))
     — Compare two specs (files, wasm files, or contract IDs) and classify
     changes as breaking or additive — the foundation for interface stability
     checks in CI.
106. **[medium] spec: read the interface from a deployed contract ID**
     ([#278](https://github.com/soroban-forge-labs/soroban-forge/issues/278))
     — Allow `spec <contract-id>` to fetch and dump a deployed contract's
     interface, not just a local wasm.
107. **[trivial] spec: add --entrypoint to print a single function signature**
     ([#279](https://github.com/soroban-forge-labs/soroban-forge/issues/279))
     — Print one entrypoint instead of the whole interface, for quick lookups
     and scripting.

## Module 9 — Deployment (`crates/deploy`)

108. **[medium] deploy: pass constructor arguments with --arg NAME=VALUE**
     ([#280](https://github.com/soroban-forge-labs/soroban-forge/issues/280))
     — Contracts with a `__constructor` cannot be deployed through forge
     without dropping to stellar-cli.
109. **[medium] deploy: record the deployed contract ID in a deployments file**
     ([#281](https://github.com/soroban-forge-labs/soroban-forge/issues/281))
     — After a successful deploy, persist the contract ID per network so later
     `invoke`/`verify` calls can default to it.
110. **[medium] deploy: fund a testnet identity via friendbot when needed**
     ([#282](https://github.com/soroban-forge-labs/soroban-forge/issues/282))
     — Deploying with an unfunded testnet identity fails with an opaque error;
     offer to fund it first.
111. **[trivial] deploy: add --dry-run printing the stellar command**
     ([#283](https://github.com/soroban-forge-labs/soroban-forge/issues/283))
     — Print the exact `stellar contract deploy` invocation that would run,
     without submitting anything.

## Module 10 — Contract invocation (`crates/invoke`)

112. **[medium] invoke: read arguments from a JSON file with --args-file**
     ([#284](https://github.com/soroban-forge-labs/soroban-forge/issues/284))
     — Complex arguments are painful on the command line; accept a JSON file
     mapping argument names to values.
113. **[medium] invoke: add --simulate for a read-only simulation**
     ([#285](https://github.com/soroban-forge-labs/soroban-forge/issues/285))
     — Simulate the call and print the result, cost and any diagnostic events
     without submitting a transaction.
114. **[trivial] invoke: pretty-print return values and decode error codes**
     ([#286](https://github.com/soroban-forge-labs/soroban-forge/issues/286))
     — Raw XDR-ish output is hard to read; format return values by their spec
     type and decode contract errors to their named variant.

## Module 11 — Identity management (`crates/identity`)

115. **[trivial] identity: add 'identity fund' wrapping friendbot**
     ([#287](https://github.com/soroban-forge-labs/soroban-forge/issues/287))
     — A one-command way to fund a test identity on testnet, instead of
     hand-rolling a friendbot request.
116. **[medium] identity: never print secret keys without --show-secret**
     ([#288](https://github.com/soroban-forge-labs/soroban-forge/issues/288))
     — Secret material must not appear in terminal scrollback, CI logs or the
     structured log file by accident.

## Module 12 — Network configuration (`crates/network`)

117. **[trivial] network: ship built-in presets for testnet, futurenet and mainnet**
     ([#289](https://github.com/soroban-forge-labs/soroban-forge/issues/289))
     — Provide the standard networks out of the box so users do not paste RPC
     URLs and passphrases by hand.
118. **[medium] network: add 'network use <name>' to set the active network**
     ([#290](https://github.com/soroban-forge-labs/soroban-forge/issues/290))
     — Persist a selected network in `forge.toml` so `deploy`, `invoke` and
     `verify` share one source of truth.

## Module 13 — Wasm optimization (`crates/optimize`)

119. **[trivial] optimize: report before/after size and percentage saved**
     ([#291](https://github.com/soroban-forge-labs/soroban-forge/issues/291))
     — Print the wasm size before and after optimization so the benefit is
     visible.
120. **[medium] optimize: add --check to fail when the optimized wasm exceeds a budget**
     ([#292](https://github.com/soroban-forge-labs/soroban-forge/issues/292))
     — Give CI a way to gate on final contract size after optimization.
