# soroban-forge-spec

`soroban-forge spec` — prints the interface of a built contract: every
entrypoint with its argument and return types, plus the structs, enums and
error enums those signatures refer to.

```sh
stellar contract build          # the spec is read out of the built wasm
soroban-forge spec              # human-readable listing
soroban-forge spec --json       # the same spec as JSON
soroban-forge spec --wasm path/to/contract.wasm
```

The interface lives in the wasm's `contractspecv0` custom section as XDR.
Per soroban-forge's "wrap, don't reimplement" rule this module does not decode
that XDR itself — it shells out to the official
`stellar contract info interface` and owns only the parts around it: finding
the build (`target/wasm32v1-none/release/<crate>.wasm`, the same layout
`bindings ts`, `verify` and `doctor` expect), choosing the representation, and
reporting a missing `stellar` CLI as `ToolMissing` (exit `2`) with a pointer
to `soroban-forge doctor`.

Nothing here touches the network, so `spec` works under `--offline`.

## Public surface

- `read_crate_name(dir)` / `locate_wasm(dir, crate_name)` — where the release
  build lands
- `resolve_wasm(dir, wasm_override)` — the wasm the spec is read from; errors
  point at `stellar contract build`
- `SpecFormat` (`Rust` / `Json`) — `SpecFormat::from_json_flag(ctx.json)`
- `spec_cli_args(wasm, format)` — the `stellar` arguments we invoke
- `dump_interface(dir, wasm_override, format) -> (PathBuf, String)` — the
  programmatic API behind the subcommand
- `format_header(wasm)` — the human-mode header line
- `SpecPlugin` — the `ForgePlugin` impl

## Tests

```sh
cargo test -p soroban-forge-spec
```

The unit tests cover wasm resolution, the error messages and the exact
`stellar` command line that gets built, so they pass without `stellar-cli`
installed. Interface output itself is the CLI's, not ours.
