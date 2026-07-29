# Storage TTL Management

Soroban charges rent for persistent storage. Entries that exceed their TTL are archived and inaccessible until restored.

## TTL Bump Pattern

Always bump TTL after reading or writing a persistent entry:

```rust
const MIN_TTL: u32 = 100_000;   // ~7 days at 5s per ledger
const MAX_TTL: u32 = 6_312_000; // ~1 year

pub fn get_pool(env: &Env, id: PoolId) -> Option<Pool> {
    let key = DataKey::Pool(id);
    let val = env.storage().persistent().get(&key);
    if val.is_some() {
        env.storage().persistent().extend_ttl(&key, MIN_TTL, MAX_TTL);
    }
    val
}
```

## Instance vs Persistent

| Durability | Use for | TTL |
|------------|---------|-----|
| Instance   | Config, admin address | Tied to contract instance |
| Persistent | User positions, pool state | Must bump manually |
| Temporary  | Nonces, one-time data | Auto-deleted after TTL |

## Testing TTL Behaviour

`soroban-forge test-init` detects `env.storage().persistent()` usage and
generates `tests/forge_ttl.rs` (or a `ttl` submodule with `--layout inline`).
The generated tests write a scratch entry inside the contract's footprint, bump
it with `extend_ttl`, and assert it outlives a ledger advance:

```rust
env.as_contract(&contract_id, || {
    env.storage().persistent().set(&KEY, &1_u32);
    env.storage()
        .persistent()
        .extend_ttl(&KEY, TTL_THRESHOLD, TTL_EXTEND_TO);
    assert!(env.storage().persistent().get_ttl(&KEY) >= TTL_EXTEND_TO);
});
```

Point `KEY` at your real `DataKey` and call your own getters so the tests fail
when a code path forgets to bump.
