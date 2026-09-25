## Summary

<!-- What does this PR do, and why? -->

Closes #<!-- issue number -->

## Module

<!-- Which module(s) does this touch? See the ownership map in CONTRIBUTING.md.
     Changes to crates/core need sign-off from the core owner. -->

## Checklist

- [ ] Tests added or updated for the change (`cargo test --workspace` passes)
- [ ] `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings` are clean
- [ ] The owning module's `README.md` is updated if its public surface, flags or output changed
- [ ] User-facing docs under `docs/` are updated if behaviour changed
- [ ] `CHANGELOG.md` has an entry under `[Unreleased]`
- [ ] Stays inside one module, or explains why it cannot
- [ ] No invented Soroban/Stellar APIs; anything unverified is marked `// TODO(verify)`
