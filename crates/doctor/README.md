## Auto-fixing (`--fix`)

`soroban-forge doctor --fix` runs the remedies doctor would otherwise only
print, for the checks that are safe to automate:

| check                  | command run                          |
|------------------------|--------------------------------------|
| `wasm32v1-none` target | `rustup target add wasm32v1-none`    |
| `stellar` CLI          | `cargo install --locked stellar-cli` |

It prints the commands it is about to run and asks for confirmation, then
re-checks and reports anything still outstanding. Non-fixable issues (a
missing `rustc`/`cargo`, `git`, or an out-of-date `soroban-sdk`) are reported
with their manual instructions rather than run automatically.

Pass `--yes` (or `-y`) to skip the prompt — useful in setup scripts. In
`--json` mode doctor never prompts; combine `--json --fix --yes` to apply
fixes non-interactively and get the re-checked results as JSON.