//! Selecting which workspace member `test-init` should target (issue #233).
//!
//! In a multi-contract workspace, generating a harness for every member is
//! rarely what someone wants when they are working on one contract — and when
//! members disagree (different constructors, different fixtures) the bulk run
//! quietly writes files the user then has to clean up. `--contract <name>`
//! makes the choice explicit, and an ambiguous workspace without it stops and
//! lists the candidates rather than guessing.

use std::path::{Path, PathBuf};

use soroban_forge_core::{ForgeError, Result};

/// A workspace member `test-init` could target.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// Cargo package name from the member's own manifest, e.g. `my-token`.
    pub package_name: String,
    /// Directory containing the member's `Cargo.toml`.
    pub dir: PathBuf,
    /// Path relative to the workspace root, for reporting.
    pub rel: String,
}

impl Candidate {
    /// The snake_case crate name, so `--contract my_token` matches a package
    /// named `my-token` the way a Rust `use` statement would.
    pub fn crate_name(&self) -> String {
        self.package_name.replace('-', "_")
    }

    /// True when `name` identifies this member, by package name, crate name or
    /// directory. Matching is case-insensitive: the flag is typed by hand.
    pub fn matches(&self, name: &str) -> bool {
        let name = name.trim().to_lowercase();
        let dir_name = self
            .dir
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        self.package_name.to_lowercase() == name
            || self.crate_name().to_lowercase() == name
            || dir_name == name
            || self.rel.to_lowercase() == name
    }
}

/// Reads a member's package name from its manifest.
fn package_name_of(dir: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(dir.join("Cargo.toml")).ok()?;
    let table: toml::Table = raw.parse().ok()?;
    table
        .get("package")?
        .get("name")?
        .as_str()
        .map(String::from)
}

/// Builds the candidate list for the workspace at `root`.
///
/// A member whose manifest cannot be read is skipped rather than failing the
/// run: it cannot be a target, and the remaining members are still selectable.
pub fn candidates(root: &Path, members: &[PathBuf]) -> Vec<Candidate> {
    members
        .iter()
        .filter_map(|dir| {
            let package_name = package_name_of(dir)?;
            let rel = dir
                .strip_prefix(root)
                .unwrap_or(dir)
                .to_string_lossy()
                .into_owned();
            Some(Candidate {
                package_name,
                dir: dir.clone(),
                rel,
            })
        })
        .collect()
}

/// Formats the candidate list for an error message, one per line.
pub fn describe(candidates: &[Candidate]) -> String {
    candidates
        .iter()
        .map(|c| format!("  {} ({})", c.package_name, c.rel))
        .collect::<Vec<_>>()
        .join("\n")
}

/// What a `test-init` invocation should generate for.
#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    /// Exactly one member, named with `--contract`.
    Single(Candidate),
    /// Every member — an unambiguous workspace, or one the caller opted into.
    All,
}

/// Resolves `--contract` against the workspace's members.
///
/// - A name that matches exactly one member selects it.
/// - A name that matches nothing fails, listing what is available.
/// - No name in a workspace with more than one member fails, listing the
///   candidates. Guessing here is what the flag exists to stop.
/// - No name in a single-member workspace proceeds as before, so existing
///   single-contract projects are unaffected.
pub fn resolve(requested: Option<&str>, candidates: &[Candidate]) -> Result<Selection> {
    match requested {
        Some(name) => {
            let matched: Vec<&Candidate> = candidates.iter().filter(|c| c.matches(name)).collect();

            match matched.len() {
                1 => Ok(Selection::Single(matched[0].clone())),
                0 => Err(ForgeError::InvalidArgument(format!(
                    "no workspace member named `{name}`.\n\nAvailable contracts:\n{}",
                    describe(candidates)
                ))),
                _ => Err(ForgeError::InvalidArgument(format!(
                    "`{name}` matches more than one workspace member:\n{}\n\n\
                     Use the full package name to disambiguate.",
                    describe(
                        &matched
                            .into_iter()
                            .cloned()
                            .collect::<Vec<_>>()
                    )
                ))),
            }
        }
        None if candidates.len() > 1 => Err(ForgeError::InvalidArgument(format!(
            "this workspace contains {} contracts, so `test-init` needs to know which one to target.\n\n\
             Available contracts:\n{}\n\n\
             Re-run with `--contract <name>`, e.g.:\n  soroban-forge test-init --contract {}",
            candidates.len(),
            describe(candidates),
            candidates[0].package_name,
        ))),
        None => Ok(Selection::All),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(package: &str, rel: &str) -> Candidate {
        Candidate {
            package_name: package.to_string(),
            dir: PathBuf::from("/ws").join(rel),
            rel: rel.to_string(),
        }
    }

    #[test]
    fn matches_by_package_crate_and_directory_name() {
        let c = candidate("my-token", "contracts/my-token");
        assert!(c.matches("my-token"), "package name");
        assert!(c.matches("my_token"), "crate name");
        assert!(c.matches("contracts/my-token"), "relative path");
        assert!(c.matches("MY-TOKEN"), "case-insensitive");
        assert!(!c.matches("other"));
    }

    #[test]
    fn selects_the_named_member() {
        let all = vec![
            candidate("my-token", "contracts/my-token"),
            candidate("my-vault", "contracts/my-vault"),
        ];
        let selection = resolve(Some("my_token"), &all).expect("resolves");
        match selection {
            Selection::Single(c) => assert_eq!(c.package_name, "my-token"),
            other => panic!("expected a single selection, got {other:?}"),
        }
    }

    #[test]
    fn an_ambiguous_workspace_without_the_flag_lists_candidates() {
        let all = vec![
            candidate("my-token", "contracts/my-token"),
            candidate("my-vault", "contracts/my-vault"),
        ];
        let err = resolve(None, &all).expect_err("should refuse to guess");
        let message = err.to_string();
        assert!(message.contains("my-token"));
        assert!(message.contains("my-vault"));
        assert!(message.contains("--contract"));
    }

    #[test]
    fn a_single_member_workspace_is_unaffected() {
        let all = vec![candidate("only-one", "contracts/only-one")];
        assert_eq!(resolve(None, &all).expect("resolves"), Selection::All);
    }

    #[test]
    fn an_empty_workspace_is_unaffected() {
        assert_eq!(resolve(None, &[]).expect("resolves"), Selection::All);
    }

    #[test]
    fn an_unknown_name_lists_what_is_available() {
        let all = vec![candidate("my-token", "contracts/my-token")];
        let err = resolve(Some("nope"), &all).expect_err("unknown member");
        let message = err.to_string();
        assert!(message.contains("no workspace member named `nope`"));
        assert!(message.contains("my-token"));
    }

    #[test]
    fn describe_lists_one_per_line() {
        let all = vec![
            candidate("a", "contracts/a"),
            candidate("b", "contracts/b"),
        ];
        assert_eq!(describe(&all), "  a (contracts/a)\n  b (contracts/b)");
    }
}
