# Releasing soroban-forge

This is the maintainer checklist for cutting a release, from version bump to
a published GitHub Release. Each step is marked **manual** or **automated**
so it is clear what a maintainer has to do by hand.

> **Current state.** soroban-forge has not published a release yet. No
> workflow builds or uploads release artifacts, and nothing is published to
> crates.io. Until a release workflow lands, every step below except CI on
> `main` is manual. Update this page in the same PR that automates a step.

## Versioning

- All crates share one version, `[workspace.package].version` in the root
  `Cargo.toml`. Every crate inherits it with `version.workspace = true`, so a
  release bumps exactly one line.
- Versions follow [Semantic Versioning](https://semver.org). While the
  project is `0.x`, a minor bump (`0.1` → `0.2`) may contain breaking
  changes to commands, flags, `--json` output or exit codes. A patch bump
  (`0.1.0` → `0.1.1`) is for fixes only.
- Tags are the version with a `v` prefix: `v0.2.0`. The `cargo binstall`
  metadata in `Cargo.toml` expects that tag format (see
  [Publish the GitHub Release](#6-publish-the-github-release)).

## The sequence

### 1. Freeze `main` (manual)

Tell contributors in the tracking issue or chat that a release is being cut,
and hold merges until the tag is pushed. Everything merged before the freeze
ships in the release.

### 2. Check `main` is green (automated)

These workflows run on every push to `main` and every PR:

| workflow | file | what it checks |
|----------|------|----------------|
| scaffold templates | `.github/workflows/scaffold-templates.yml` | builds the CLI, scaffolds every bundled template, runs `cargo test` and `stellar contract build` in each |
| labeler | `.github/workflows/labeler.yml` | applies `module:*` labels to PRs (no build) |

Make sure the latest `main` run of **scaffold templates** passed. Then run
the full local check, which CI does not run in full yet:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Do not release with failing tests. Fix them first or revert the change that
broke them.

### 3. Bump the version (manual)

On a branch named `release/vX.Y.Z`:

```sh
# root Cargo.toml
[workspace.package]
version = "X.Y.Z"
```

Then refresh the lockfile so it records the new version for every
workspace crate:

```sh
cargo update --workspace
```

Also update the version mentioned in the README's install step
(`# 1. install soroban-forge (from source, vX.Y)`) when the minor version
changes.

### 4. Cut the changelog (manual)

`CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Contributors add entries under `## [Unreleased]` as part of their PRs (the
PR template asks for this). To cut the release:

1. Rename `## [Unreleased]` to `## [X.Y.Z] - YYYY-MM-DD`.
2. Add a fresh, empty `## [Unreleased]` above it.
3. Read through the entries. Merge duplicates, move anything misfiled
   between `Added` / `Changed` / `Fixed` / `Removed`, and call out breaking
   changes at the top of the section.
4. Add the compare links at the bottom of the file:

   ```markdown
   [Unreleased]: https://github.com/soroban-forge-labs/soroban-forge/compare/vX.Y.Z...HEAD
   [X.Y.Z]: https://github.com/soroban-forge-labs/soroban-forge/compare/vPREVIOUS...vX.Y.Z
   ```

   For the first release, link `[X.Y.Z]` to the tag itself:
   `https://github.com/soroban-forge-labs/soroban-forge/releases/tag/vX.Y.Z`.

Open a PR titled `release: vX.Y.Z` with the version bump and changelog, and
merge it once CI passes.

### 5. Tag the release (manual, scripted)

From an up-to-date `main` that contains the release PR:

```sh
git checkout main && git pull --ff-only
scripts/release.sh X.Y.Z
```

`scripts/release.sh` creates the annotated tag `vX.Y.Z` and pushes it to
`origin`. It does not build, test or publish anything. Nothing reacts to the
tag yet.

### 6. Publish the GitHub Release (manual)

1. Build the binary for each target you are shipping:

   ```sh
   cargo build --release --locked --target <target>
   ```

2. Package each one under the name the `cargo binstall` metadata expects, so
   `cargo binstall soroban-forge` can find it:

   | target | archive |
   |--------|---------|
   | Linux / macOS targets | `soroban-forge-X.Y.Z-<target>.tgz` containing `soroban-forge` |
   | `x86_64-pc-windows-msvc` | `soroban-forge-X.Y.Z-x86_64-pc-windows-msvc.zip` containing `soroban-forge.exe` |

3. Create the release from the tag and attach the archives:

   ```sh
   gh release create vX.Y.Z soroban-forge-X.Y.Z-*.{tgz,zip} \
     --title "vX.Y.Z" --notes-file <(sed -n '/^## \[X.Y.Z\]/,/^## \[/p' CHANGELOG.md | sed '$d')
   ```

   The notes are the `X.Y.Z` section of the changelog.

### 7. crates.io (not published)

The crates are **not** published to crates.io. The internal dependencies in
the root `Cargo.toml` are path-only (`{ path = "crates/core" }` with no
`version`), which `cargo publish` rejects. Publishing would need versions on
those dependencies and a publish order that puts `soroban-forge-core` first.
Until that is decided, users install from source (`cargo install --path .`)
or from the GitHub Release.

### 8. Unfreeze and announce (manual)

Reopen merges on `main`, then post the release link in the tracking issue and
wherever contributors coordinate.

## Summary

| step | automated? |
|------|------------|
| 1. Freeze `main` | manual |
| 2. CI on `main` | **automated** (`scaffold-templates.yml`), plus a manual full `cargo test` / clippy run |
| 3. Version bump | manual |
| 4. Changelog cut | manual |
| 5. Tag | manual, via `scripts/release.sh` |
| 6. GitHub Release and binaries | manual |
| 7. crates.io | not published |
| 8. Unfreeze and announce | manual |

## Hotfix releases

For a fix on top of a released version when `main` has moved on:

1. Branch from the tag, naming the branch after the patch version you are
   about to release: `git checkout -b release/v0.2.1 v0.2.0`.
2. Cherry-pick the fix, bump the patch version, and add a changelog section.
3. Tag and release from that branch (steps 5–6), then merge the changelog
   entry back into `main` so the history stays complete.
