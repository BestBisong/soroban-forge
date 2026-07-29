# Templates

List them at any time with `soroban-forge templates`:

- `amm` – constant-product AMM / liquidity pool (x*y=k, 0.3% fee)
- `atomic-swap` – atomic two-party token swap with dual authorization
- `crowdfund` – escrow/deadline crowdfunding contract
- `escrow` – token escrow with approval or timeout-based refund path
- `governance` – DAO governance with weighted voting, quorum, and proposal execution
- `hello-world` – minimal greeter contract (recommended starting point)
- `multisig` – M-of-N multisig account contract (`CustomAccountInterface`)
- `nft` – NFT with per-token metadata and minting
- `token` – SEP-41 fungible token (`soroban_sdk::token::TokenInterface`)
- `vesting` – token vesting with cliff + linear release schedule

## Variables

Template files (and file names) may contain `{{variable}}` placeholders.
soroban-forge always provides `project_name`, `crate_name` (snake_case),
`author`, `sdk_version` and `edition`.

A template can declare its own variables in a `template.toml` at its root:

```toml
description = "a token with a configurable symbol"

[[variables]]
name = "token_symbol"
prompt = "Token symbol"
default = "TKN"

[[variables]]
name = "admin_address"
prompt = "Admin account (G...)"
required = true          # the default; set false to allow an empty value
```

Supply values on the command line:

```console
$ soroban-forge new my-token --template my-token --var token_symbol=USDC
```

Anything you leave out is prompted for when stdin and stdout are both a
terminal — press enter to accept the default shown in brackets:

```console
$ soroban-forge new my-token --template my-token
this template needs a few values (press enter to accept a default):
Token symbol [TKN]:
Admin account (G...): GB7X...
```

Runs that cannot prompt — piped input, `--json`, or `--yes` — use the declared
default instead. A required variable with no default and no `--var` is an
error naming the flag that would have supplied it, so CI fails fast rather than
hanging on a prompt.

`template.toml` configures generation and is never written into the generated
project. The five built-in variables are derived by soroban-forge and cannot be
redeclared or overridden with `--var`.
