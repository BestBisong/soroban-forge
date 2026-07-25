# Examples

Checked-in output of `soroban-forge`, kept for browsing without installing the
tool. Owned by Module 5 (docs & DX).

## Regenerating examples

After changing templates or presets, regenerate the checked-in examples so they
never drift from real output:

```sh
# From the repo root:
cd examples
soroban-forge new hello-forge --template hello-world
soroban-forge test-init --force
soroban-forge ci-init --deploy
```

These directories are excluded from the cargo workspace (they are standalone
projects).

## Available examples

- [`hello-forge/`](hello-forge) — minimal hello-world contract with test
  harness and CI workflows.
