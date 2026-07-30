# CI presets

Workflow templates consumed by `soroban-forge ci-init --provider <p>`.
Owned by Module 4 — see [`crates/ci-presets`](../crates/ci-presets). (updated)

Each provider is a subdirectory. `github/` is the richest one:

- `build-test.yml` — cargo test + wasm build on push/PR
- `build-test-matrix.yml` — the same job run once per Rust toolchain, stable
  plus a pinned MSRV (only written with `--matrix`)
- `build-test.yml` — cargo test + wasm build on push/PR, plus a `lint` job
  running `cargo fmt --all --check` and `cargo clippy --all-targets -- -D
  warnings`
- `contract-size.yml` — fails PRs when the built wasm exceeds a size limit
- `testnet-deploy.yml` — manual testnet deploy wrapping the official
  stellar-cli (only written with `--deploy`); references GitHub secrets, never
  stores keys
- `dependabot.yml` — weekly `cargo` and `github-actions` update PRs (only
  written with `--dependabot`); lands at `.github/dependabot.yml`, not in
  `.github/workflows/`

The other providers each carry a single build+test file mirroring
`build-test.yml`: `gitlab/.gitlab-ci.yml`, `circleci/config.yml` and
`bitbucket/bitbucket-pipelines.yml`.

Templates may use `{{project_name}}` / `{{crate_name}}` / `{{msrv}}`; GitHub's
own `${{ ... }}` expressions pass through rendering untouched.
