# Contract Verification

## Does the deployed contract match my build?

```sh
soroban-forge verify CDLZ… --network testnet
```

`verify` compares the wasm deployed at a contract ID with your local release
build. A Soroban contract's on-chain wasm hash is the SHA-256 of its deployed
bytes, so the check is a hash comparison:

| side       | what is hashed                                                  |
|------------|-----------------------------------------------------------------|
| local      | `target/wasm32v1-none/release/<crate>.wasm`, or `--wasm <path>` |
| on-chain   | the bytes `stellar contract fetch` returns for the contract ID  |

A match means the deployed contract is byte-for-byte the wasm you have
locally.

```
✓ verified — the deployed contract matches the local build

  contract   CDLZ…
  network    testnet
  local      target/wasm32v1-none/release/my_token.wasm

  sha256     9f2c…
```

A mismatch names both hashes:

```
✗ MISMATCH — the deployed contract was NOT built from this wasm

  contract   CDLZ…
  network    testnet
  local      target/wasm32v1-none/release/my_token.wasm

  local      sha256 9f2c…
  on-chain   sha256 41ab…
```

### Options

| flag                       | meaning                                                     |
|----------------------------|-------------------------------------------------------------|
| `--path <dir>`             | contract project directory [default: current directory]     |
| `--wasm <file>`            | compare this wasm instead of the project's release build     |
| `--network`, `-n <name>`   | configured network to query [default: `testnet`]            |
| `--rpc-url <url>`          | query this RPC endpoint instead of a configured network      |
| `--network-passphrase <p>` | passphrase for `--rpc-url`                                   |

### Exit codes

| code | meaning                                                        |
|------|-----------------------------------------------------------------|
| `0`  | the hashes match                                                |
| `1`  | the hashes differ, or an argument/local build was wrong         |
| `2`  | the `stellar` CLI is not installed (`soroban-forge doctor`)     |

So a CI job can gate on a deployment being the reviewed code:

```sh
soroban-forge verify "$CONTRACT_ID" --network testnet || exit 1
```

`--json` prints the same report machine-readably, which distinguishes a
mismatch from a bad argument (both exit `1`):

```json
{
  "contract_id": "CDLZ…",
  "network": "testnet",
  "local_wasm": "target/wasm32v1-none/release/my_token.wasm",
  "local_hash": "9f2c…",
  "onchain_hash": "41ab…",
  "match": false
}
```

### When a mismatch is expected

- the deployed contract is an **older build** — redeploy, or check out the
  commit that was deployed and rebuild
- you built with **different flags** than the deployment (wasm is sensitive
  to optimisation settings; always compare a `stellar contract build` release
  artifact, not a `cargo build` one)
- you are pointed at the **wrong network** — `--network mainnet` and
  `--network testnet` hold different deployments of the same project

## Public source verification

Publishing your *source* alongside the deployment — so explorers such as
Stellar Expert can show a verified badge and users can read the code their
funds interact with — is a separate flow run through the Stellar contract
verification service, and is not part of `soroban-forge` today. `verify`
answers the local question only: does this build correspond to that
deployment?
