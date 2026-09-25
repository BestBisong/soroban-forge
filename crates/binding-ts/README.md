# soroban-forge-bindings-ts

`soroban-forge bindings ts` — generates a TypeScript client package from a
built contract's wasm.

Wraps the official `stellar contract bindings typescript` — this module
never reimplements XDR-spec-to-TypeScript generation itself.

## A publishable package

After the CLI generates the package, `package.json` is rewritten so the
output can be `npm pack`ed or published without edits:

| field              | value                                                                   |
|--------------------|-------------------------------------------------------------------------|
| `name`, `version`  | from `[package]` in `Cargo.toml` (kept from the CLI when unknown)        |
| `exports`          | `{ ".": { "types", "import", "default" }, "./package.json" }`            |
| `main`, `types`    | `./dist/index.js`, `./dist/index.d.ts` for tools that predate `exports`  |
| `files`            | `dist`, `src`, `README.md`                                              |
| `scripts.prepack`  | `npm run build`, so a pack never ships a stale or missing `dist/`        |
| `peerDependencies` | `@stellar/stellar-sdk` (also kept in `devDependencies` for the build)   |
| `engines.node`     | `>=18`                                                                  |

The declarations resolve under both `node16`/`nodenext` and `bundler`
module resolution.

**Supported Stellar SDK range.** The generated client is written against
the SDK version the installed `stellar-cli` targets, so the peer range is
the one the CLI emits: `^16` for stellar-cli 28. If a future CLI stops
pinning the SDK, the fallback is `DEFAULT_STELLAR_SDK_RANGE` (`^16.0.0`).
Apps install the SDK themselves, which keeps a single SDK instance in the
bundle:

```sh
cd bindings/typescript
npm install
npm pack          # runs the build, then packs dist/, src/ and README.md
```

## Public surface

- `read_package_info(dir)` — reads `[package].name` and `version` from
  `Cargo.toml`
- `locate_wasm(dir, crate_name)` — the default build output path under
  `target/wasm32v1-none/release/`
- `generate_bindings(contract_dir, wasm_override, output, force)` — the
  programmatic API behind `bindings ts`
- `make_publishable(package_json, info)` — the `package.json` rewrite above
- `BindingsTsPlugin` — the `ForgePlugin` impl

## Testing

```sh
cargo test -p soroban-forge-bindings-ts
```

Tests never shell out to the real `stellar` binary — the pre-flight checks
(missing wasm, existing output dir) and the `package.json` rewrite are
covered directly.