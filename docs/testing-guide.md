# Testing Guide

## Unit Tests (Rust)

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register(MyContract, ());
        let client = MyContractClient::new(&env, &contract_id);
        client.initialize(&admin);
        assert_eq!(client.get_admin(), admin);
    }
}
```

## Integration Tests (TypeScript)

```typescript
import { forge } from 'soroban-forge/testing';

describe('Distribution contract', () => {
  it('distributes rewards proportionally', async () => {
    const { client, mint } = await forge.deploy('distribution');
    await mint({ to: userA, amount: 1000n });
    await client.deposit({ user: userA, amount: 1000n });
    await client.distribute({ from: admin, amount: 4000n });
    expect(await client.getPending({ user: userA })).toBe(4000n);
  });
});
```

## Benchmarks (criterion)

Budget tests assert a ceiling — they catch the moment a cost crosses a line and
say nothing until then. Benchmarks record the number on every run, so a change
that adds 20% to an entrypoint is visible immediately rather than whenever it
happens to breach the budget. Use both: the budget test gates, the bench tracks.

Generate a `benches/` target alongside the rest of the harness:

```bash
soroban-forge test-init --bench
```

That writes `benches/forge_bench.rs` with one criterion benchmark per
entrypoint, and adds the `[[bench]]` target and the criterion dev-dependency to
`Cargo.toml`. Re-running leaves an already-declared target alone rather than
appending a second section.

```bash
cargo bench                          # every entrypoint
cargo bench -- mint                  # one, by name
cargo bench -- --save-baseline main  # record a baseline
cargo bench -- --baseline main       # compare the current branch against it
```

Criterion writes HTML reports to `target/criterion/`.

Each iteration rebuilds the environment and re-registers the contract. That is
deliberate: a contract whose cost grows with accumulated state would otherwise
be measured against whatever the previous iterations left behind, and the
numbers would drift upward for reasons that have nothing to do with the code.

The generated benchmarks call every entrypoint with default arguments derived
from its parameter types — a fresh account for an `Address`, `0` for a numeric,
an empty collection for a container. For an entrypoint whose cost depends on its
input (anything that loops over a `Vec`, say), edit the benchmark to pass a
realistic value; the default will measure the empty case and tell you very
little.

### Benchmarks vs. budget tests

| | `--bench` (criterion) | `--budget` |
| --- | --- | --- |
| Measures | Host wall-clock time | Metered CPU instructions and memory |
| Fails when | Never — it reports | The measured cost exceeds a ceiling |
| Answers | "Did this get slower?" | "Does this still fit in a ledger?" |
| Run with | `cargo bench` | `cargo test --test forge_budget` |

Wall-clock is not what the network charges for, so it is never the number to
quote in a capacity discussion — use the budget test for that. It is the right
tool for spotting a regression trend, because it moves with the same work the
metered cost does.

> `--bench` used to be an alias for `--budget`. It now emits criterion
> benchmarks; use `--budget` for the ceiling test.
