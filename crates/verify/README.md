# soroban-forge-verify

`soroban-forge verify <contract-id>` — checks whether the contract deployed
at `<contract-id>` is the code you have locally.

A contract's on-chain wasm hash is the SHA-256 of its deployed bytes, so the
check is a hash comparison:

| side     | what is hashed                                                    |
|----------|-------------------------------------------------------------------|
| local    | `target/wasm32v1-none/release/<crate>.wasm` (or `--wasm <path>`)   |
| on-chain | the bytes `stellar contract fetch` returns for the contract ID     |

The on-chain half is downloaded with the official
`stellar contract fetch` — this module never talks to the network itself.

```sh
soroban-forge verify CDLZ… --network testnet
```

Exit codes: `0` match, `1` mismatch (or a bad argument), `2` `stellar` CLI
missing. `--json` prints the report — including `"match": true|false` and
both hashes — so CI can branch without parsing text.

## Public surface

- `validate_contract_id(id)` — cheap strkey shape check
- `read_crate_name(dir)` / `locate_wasm(dir, crate_name)` — where the release
  build lands
- `resolve_local_wasm(dir, wasm_override)` — the wasm that will be hashed
- `sha256_hex(bytes)` / `hash_wasm_file(path)` — hashing, with a wasm-header
  guard
- `NetworkArgs` — `--network` / `--rpc-url` / `--network-passphrase` and the
  `stellar` arguments they map to
- `verify(contract_id, dir, wasm_override, network) -> VerifyReport` — the
  programmatic API behind the subcommand
- `format_report` / `json_report` / `mismatch_error` — output and the error
  a mismatch becomes
- `VerifyPlugin` — the `ForgePlugin` impl

## Testing

```sh
cargo test -p soroban-forge-verify
```

Tests never shell out to the real `stellar` binary or hit a network. The
local side is resolved *before* the fetch, so every failure mode short of a
real deployment (bad contract ID, missing build, non-wasm file) is covered
directly; comparison and reporting are tested on constructed hashes.
