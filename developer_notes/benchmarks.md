# Benchmarks

Soroban enforces per-transaction limits of ~100M CPU instructions and 40 MiB of
memory. Staying well under them is what keeps a contract composable: anything
that calls yours has to fit in the same budget.

## Generating a budget test

`soroban-forge test-init --budget [ENTRYPOINT]` writes `tests/forge_budget.rs`,
which measures one entrypoint and fails if it exceeds the ceilings declared at
the top of the file. Without an entrypoint it measures the first one detected.

```console
$ soroban-forge test-init --budget transfer
$ cargo test --test forge_budget -- --nocapture
transfer: 62525 CPU instructions, 25198 memory bytes
```

Then tighten `MAX_CPU_INSTRUCTIONS` / `MAX_MEMORY_BYTES` to just above the
measured cost, so a regression fails the test rather than showing up on-chain.

## Profiling by hand

```rust
let env = Env::default();
// ... set up fixtures, register the contract ...

// Reset after setup so the numbers describe the call, not the scaffolding.
env.cost_estimate().budget().reset_unlimited();
client.transfer(&from, &to, &amount);

let cpu = env.cost_estimate().budget().cpu_instruction_cost();
let mem = env.cost_estimate().budget().memory_bytes_cost();
```

`env.cost_estimate()` requires soroban-sdk's `testutils` feature, which the
generated harness already enables.

Keep hot paths under 10M instructions to leave headroom for composability.
