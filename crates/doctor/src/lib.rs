//! # soroban-forge-doctor
//!
//! `soroban-forge doctor` — checks that the local environment can build and
//! deploy Soroban contracts, and prints fix instructions for anything
//! missing:
//!
//! - Rust toolchain (`rustc`, `cargo`) at the minimum supported version
//! - the `wasm32v1-none` compilation target
//! - the official `stellar` CLI
//! - `git` (recommended, not required)
//! - when run inside a contract project: the project's `soroban-sdk`
//!   version, compared against the version pinned into new projects
//!   (`soroban_forge_scaffold::SOROBAN_SDK_VERSION`)
//!
//! With `--fix`, doctor will, after confirmation, run the subset of remedies
//! that are safe to automate (`rustup target add`, `cargo install`), then
//! re-check and report anything still outstanding. `--yes`/`-y` skips the
//! prompt for non-interactive use.

use std::path::Path;

use clap::{Arg, ArgAction, ArgMatches, Command};
use serde::{Deserialize, Serialize};
use soroban_forge_core::{ForgeContext, ForgeError, ForgePlugin, Result};
use soroban_forge_scaffold::SOROBAN_SDK_VERSION;

/// Minimum Rust version able to target `wasm32v1-none`.
pub const MIN_RUST: (u32, u32) = (1, 84); // minimum major.minor

/// Outcome of a single environment check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Requirement met.
    Pass,
    /// Missing but not required for local development.
    Warn,
    /// Required and missing/broken.
    Fail,
}

/// One line of the doctor report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Check {
    pub name: &'static str,
    pub status: Status,
    /// What was found, e.g. the tool's version line.
    pub detail: String,
    /// How to fix it, shown for non-passing checks.
    pub fix: Option<&'static str>,
}

/// Run `cmd args...` and return its first line of stdout on success.
fn capture(cmd: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().next().map(|l| l.trim().to_string())
}

/// Parse `X.Y` out of a `tool X.Y.Z ...` version line and compare against a
/// minimum. Unparseable versions count as too old.
pub fn version_at_least(version_line: &str, min: (u32, u32)) -> bool {
    version_line
        .split_whitespace()
        .find_map(|word| {
            let mut parts = word.split('.');
            let major: u32 = parts.next()?.parse().ok()?;
            let minor: u32 = parts
                .next()?
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok()?;
            Some((major, minor))
        })
        .map(|version| version >= min)
        .unwrap_or(false)
}

/// Leniently parse a cargo version requirement (e.g. `26.1.0`, `^26.1`,
/// `=26.1.0`, `>=26, <27`) into `(major, minor, patch)`. Missing components
/// default to zero. Returns `None` for wildcards or anything else that does
/// not start with a numeric major version.
pub fn parse_semverish(version: &str) -> Option<(u32, u32, u32)> {
    let first = version
        .split(',')
        .next()?
        .trim()
        .trim_start_matches(['^', '~', '=', '>', '<', 'v', ' ']);
    let mut parts = first.split('.');
    let major: u32 = parts.next()?.trim().parse().ok()?;
    let minor: u32 = parts
        .next()
        .unwrap_or("0")
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0);
    let patch: u32 = parts
        .next()
        .unwrap_or("0")
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0);
    Some((major, minor, patch))
}

/// Extract the declared `soroban-sdk` version from a parsed manifest.
///
/// Outer `Option`: whether the manifest declares a `soroban-sdk` dependency
/// at all. Inner `Option`: the version requirement, `None` when the
/// dependency has no `version` key (e.g. a git or path dependency).
fn manifest_sdk_version(manifest: &toml::Value) -> Option<Option<String>> {
    for table in ["dependencies", "dev-dependencies"] {
        if let Some(dep) = manifest.get(table).and_then(|t| t.get("soroban-sdk")) {
            return Some(dep_version(dep));
        }
    }
    manifest
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(|t| t.get("soroban-sdk"))
        .map(dep_version)
}

/// The version requirement of a single dependency entry, covering both the
/// string form (`soroban-sdk = "26.1.0"`) and the table form
/// (`soroban-sdk = { version = "26.1.0", ... }`).
fn dep_version(dep: &toml::Value) -> Option<String> {
    match dep {
        toml::Value::String(s) => Some(s.clone()),
        other => other
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from),
    }
}

/// Check the project's `soroban-sdk` version against the version pinned into
/// freshly scaffolded projects.
///
/// Returns `None` (no report line at all) when `project_dir` does not look
/// like a contract project: no readable/parseable `Cargo.toml`, or a
/// manifest without a `soroban-sdk` dependency. Otherwise:
///
/// - `Pass` when the declared version is at or above the pinned one
/// - `Warn` when it is behind, unversioned, or unparseable
pub fn sdk_version_check(project_dir: &Path) -> Option<Check> {
    let contents = std::fs::read_to_string(project_dir.join("Cargo.toml")).ok()?;
    let manifest: toml::Value = toml::from_str(&contents).ok()?;
    let declared = manifest_sdk_version(&manifest)?;
    let pinned = parse_semverish(SOROBAN_SDK_VERSION)?;

    Some(match declared {
        None => Check {
            name: "soroban-sdk",
            status: Status::Warn,
            detail: format!("no version specified (latest pinned: {SOROBAN_SDK_VERSION})"),
            fix: Some("pin a soroban-sdk version in Cargo.toml"),
        },
        Some(raw) => match parse_semverish(&raw) {
            Some(found) if found >= pinned => Check {
                name: "soroban-sdk",
                status: Status::Pass,
                detail: format!("soroban-sdk {raw}"),
                fix: None,
            },
            Some(_) => Check {
                name: "soroban-sdk",
                status: Status::Warn,
                detail: format!("soroban-sdk {raw} (latest pinned: {SOROBAN_SDK_VERSION})"),
                fix: Some("update the soroban-sdk version in Cargo.toml"),
            },
            None => Check {
                name: "soroban-sdk",
                status: Status::Warn,
                detail: format!(
                    "could not parse version `{raw}` (latest pinned: {SOROBAN_SDK_VERSION})"
                ),
                fix: Some("pin a concrete soroban-sdk version in Cargo.toml"),
            },
        },
    })
}

/// Run all environment checks.
pub fn run_checks() -> Vec<Check> {
    let mut checks = Vec::new();

    // rustc, with a minimum version.
    checks.push(match capture("rustc", &["--version"]) {
        Some(line) if version_at_least(&line, MIN_RUST) => Check {
            name: "rustc",
            status: Status::Pass,
            detail: line,
            fix: None,
        },
        Some(line) => Check {
            name: "rustc",
            status: Status::Fail,
            detail: format!("{line} (need >= {}.{})", MIN_RUST.0, MIN_RUST.1),
            fix: Some("update Rust: rustup update stable"),
        },
        None => Check {
            name: "rustc",
            status: Status::Fail,
            detail: "not found".into(),
            fix: Some("install Rust: https://rustup.rs"),
        },
    });

    // cargo.
    checks.push(match capture("cargo", &["--version"]) {
        Some(line) => Check {
            name: "cargo",
            status: Status::Pass,
            detail: line,
            fix: None,
        },
        None => Check {
            name: "cargo",
            status: Status::Fail,
            detail: "not found".into(),
            fix: Some("install Rust (includes cargo): https://rustup.rs"),
        },
    });

    // wasm32v1-none target.
    let installed_targets = std::process::Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned());
    checks.push(match installed_targets {
        Some(targets) if targets.lines().any(|t| t.trim() == "wasm32v1-none") => Check {
            name: "wasm32v1-none target",
            status: Status::Pass,
            detail: "installed".into(),
            fix: None,
        },
        Some(_) => Check {
            name: "wasm32v1-none target",
            status: Status::Fail,
            detail: "not installed".into(),
            fix: Some("rustup target add wasm32v1-none"),
        },
        None => Check {
            name: "wasm32v1-none target",
            status: Status::Warn,
            detail: "rustup not found — could not verify".into(),
            fix: Some("install rustup (https://rustup.rs), then: rustup target add wasm32v1-none"),
        },
    });

    // stellar-cli.
    checks.push(match capture("stellar", &["--version"]) {
        Some(line) => Check {
            name: "stellar-cli",
            status: Status::Pass,
            detail: line,
            fix: None,
        },
        None => Check {
            name: "stellar-cli",
            status: Status::Fail,
            detail: "not found".into(),
            fix: Some(
                "install: brew install stellar-cli  (or: cargo install --locked stellar-cli)",
            ),
        },
    });

    // git — recommended only.
    checks.push(match capture("git", &["--version"]) {
        Some(line) => Check {
            name: "git",
            status: Status::Pass,
            detail: line,
            fix: None,
        },
        None => Check {
            name: "git",
            status: Status::Warn,
            detail: "not found".into(),
            fix: Some("install git: https://git-scm.com/downloads"),
        },
    });

    checks
}

/// Render the report as shown to the user.
pub fn format_report(checks: &[Check]) -> String {
    let mut out = String::from("soroban-forge doctor\n\n");
    for check in checks {
        let symbol = match check.status {
            Status::Pass => "✓",
            Status::Warn => "!",
            Status::Fail => "✗",
        };
        out.push_str(&format!("  {symbol} {:<22} {}\n", check.name, check.detail));
        if check.status != Status::Pass {
            if let Some(fix) = check.fix {
                out.push_str(&format!("      fix: {fix}\n"));
            }
        }
    }
    let failures = checks.iter().filter(|c| c.status == Status::Fail).count();
    let warnings = checks.iter().filter(|c| c.status == Status::Warn).count();
    out.push('\n');
    if failures == 0 && warnings == 0 {
        out.push_str("all checks passed — you're ready to build Soroban contracts.\n");
    } else {
        out.push_str(&format!("{failures} failure(s), {warnings} warning(s).\n"));
    }
    out
}

/// Render the report as JSON.
pub fn format_json_report(checks: &[Check]) -> String {
    serde_json::to_string_pretty(checks).unwrap_or_else(|_| "[]".to_string())
}

/// Count required checks that failed.
pub fn failure_count(checks: &[Check]) -> usize {
    checks
        .iter()
        .filter(|check| check.status == Status::Fail)
        .count()
}

// ---------------------------------------------------------------------------
// Auto-fix (`--fix`)
// ---------------------------------------------------------------------------

/// An auto-installable remedy for a failing check: a concrete, non-interactive
/// command doctor can run to resolve it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remedy {
    /// The name of the [`Check`] this remedy resolves.
    pub check: &'static str,
    /// The program to invoke, e.g. `rustup` or `cargo`.
    pub program: &'static str,
    /// Arguments passed to `program`.
    pub args: &'static [&'static str],
}

impl Remedy {
    /// The remedy rendered as a copy-pasteable shell command line.
    pub fn command_line(&self) -> String {
        if self.args.is_empty() {
            self.program.to_string()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }
}

/// The auto-installable remedy for a check, when one exists.
///
/// Only genuinely runnable, non-interactive install commands are returned, and
/// only for checks that are currently [`Status::Fail`]. Checks whose fix is a
/// URL or a manual step (`rustc`/`cargo` install, `git`, `soroban-sdk`
/// version, or the target when `rustup` itself is missing) return `None`, so
/// they are reported rather than auto-fixed.
pub fn remedy(check: &Check) -> Option<Remedy> {
    if check.status != Status::Fail {
        return None;
    }
    match check.name {
        "wasm32v1-none target" => Some(Remedy {
            check: check.name,
            program: "rustup",
            args: &["target", "add", "wasm32v1-none"],
        }),
        "stellar-cli" => Some(Remedy {
            check: check.name,
            program: "cargo",
            args: &["install", "--locked", "stellar-cli"],
        }),
        _ => None,
    }
}

/// Every auto-fixable remedy for the current set of checks, in check order.
pub fn fixable_remedies(checks: &[Check]) -> Vec<Remedy> {
    checks.iter().filter_map(remedy).collect()
}

/// The result of attempting a single [`Remedy`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixOutcome {
    /// The name of the check this remedy targeted.
    pub check: &'static str,
    /// The command line that was run.
    pub command: String,
    /// Whether the command exited successfully.
    pub succeeded: bool,
    /// Failure detail (non-zero status or spawn error), when it failed.
    pub detail: Option<String>,
}

/// The plan shown to the user before confirmation: the commands `--fix` will
/// run, one per line.
pub fn format_fix_plan(remedies: &[Remedy]) -> String {
    let mut out = String::from("The following command(s) will be run:\n\n");
    for remedy in remedies {
        out.push_str(&format!("  {}\n", remedy.command_line()));
    }
    out.push('\n');
    out
}

/// A short human summary of what `--fix` ran and whether each command worked.
pub fn format_fix_summary(outcomes: &[FixOutcome]) -> String {
    let mut out = String::from("\nfix results:\n");
    for outcome in outcomes {
        let symbol = if outcome.succeeded { "✓" } else { "✗" };
        out.push_str(&format!("  {symbol} {}", outcome.command));
        if let Some(detail) = &outcome.detail {
            out.push_str(&format!(" — {detail}"));
        }
        out.push('\n');
    }
    out.push('\n');
    out
}

/// Run a single remedy, inheriting stdio so the user sees install progress.
///
/// Thin system-touching wrapper (like [`capture`]); not unit-tested.
fn run_remedy(remedy: &Remedy) -> FixOutcome {
    let command = remedy.command_line();
    let status = std::process::Command::new(remedy.program)
        .args(remedy.args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();
    match status {
        Ok(s) if s.success() => FixOutcome {
            check: remedy.check,
            command,
            succeeded: true,
            detail: None,
        },
        Ok(s) => FixOutcome {
            check: remedy.check,
            command,
            succeeded: false,
            detail: Some(match s.code() {
                Some(code) => format!("exited with status {code}"),
                None => "terminated by signal".to_string(),
            }),
        },
        Err(e) => FixOutcome {
            check: remedy.check,
            command,
            succeeded: false,
            detail: Some(e.to_string()),
        },
    }
}

/// Run every remedy in order, returning one outcome per remedy.
///
/// Thin system-touching wrapper; not unit-tested.
fn apply_remedies(remedies: &[Remedy]) -> Vec<FixOutcome> {
    remedies.iter().map(run_remedy).collect()
}

/// Prompt on stdout and read a yes/no answer from stdin. Anything other than
/// `y`/`yes` (case-insensitive), including EOF, is treated as "no".
///
/// Thin system-touching wrapper; not unit-tested.
fn confirm(prompt: &str) -> bool {
    use std::io::Write;
    print!("{prompt} [y/N] ");
    let _ = std::io::stdout().flush();
    let mut answer = String::new();
    if std::io::stdin().read_line(&mut answer).is_err() {
        return false;
    }
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

/// The `doctor` subcommand.
pub struct DoctorPlugin;

impl DoctorPlugin {
    /// Run every check, including the project-local `soroban-sdk` check when
    /// invoked inside a contract project.
    fn gather_checks(&self, ctx: &ForgeContext) -> Vec<Check> {
        let mut checks = run_checks();
        if let Some(check) = sdk_version_check(&ctx.cwd) {
            checks.push(check);
        }
        checks
    }

    /// Print the report as text or JSON, honouring `--quiet`.
    fn emit_report(&self, checks: &[Check], use_json: bool, quiet: bool) {
        if use_json {
            println!("{}", format_json_report(checks));
        } else if !quiet {
            print!("{}", format_report(checks));
        }
    }

    /// Decide whether to run the remedies.
    ///
    /// `--yes` proceeds unconditionally. In JSON mode we never prompt (it
    /// would corrupt the machine-readable stream), so a fix only proceeds
    /// there when `--yes` is given. Otherwise we print the plan and ask.
    fn confirm_fix(
        &self,
        remedies: &[Remedy],
        use_json: bool,
        assume_yes: bool,
        quiet: bool,
    ) -> bool {
        if assume_yes {
            return true;
        }
        if use_json {
            return false;
        }
        if !quiet {
            print!("{}", format_fix_plan(remedies));
        }
        confirm("Proceed?")
    }
}

impl ForgePlugin for DoctorPlugin {
    fn name(&self) -> &'static str {
        "doctor"
    }

    fn command(&self) -> Command {
        Command::new("doctor")
            .about(
                "Check that Rust, the wasm32v1-none target and stellar-cli are installed, \
                 and that the project's soroban-sdk is up to date",
            )
            .arg(
                Arg::new("json")
                    .long("json")
                    .action(ArgAction::SetTrue)
                    .help("Output check results as JSON"),
            )
            .arg(
                Arg::new("fix")
                    .long("fix")
                    .action(ArgAction::SetTrue)
                    .help(
                        "Attempt to install missing toolchain components \
                         (rustup target add, cargo install), then re-check",
                    ),
            )
            .arg(
                Arg::new("yes")
                    .long("yes")
                    .short('y')
                    .action(ArgAction::SetTrue)
                    .help("Assume \"yes\"; run --fix remedies without prompting"),
            )
    }

    fn run(&self, matches: &ArgMatches, ctx: &ForgeContext) -> Result<()> {
        let use_json = ctx.json || matches.get_flag("json");
        let do_fix = matches.get_flag("fix");
        let assume_yes = ctx.yes || matches.get_flag("yes");

        let mut checks = self.gather_checks(ctx);

        if do_fix {
            let remedies = fixable_remedies(&checks);
            if !remedies.is_empty()
                && self.confirm_fix(&remedies, use_json, assume_yes, ctx.quiet)
            {
                let outcomes = apply_remedies(&remedies);
                if !use_json && !ctx.quiet {
                    print!("{}", format_fix_summary(&outcomes));
                }
                // Re-check so the final report reflects the fixes; any
                // non-fixable issues (and any remedy that failed) remain.
                checks = self.gather_checks(ctx);
            }
        }

        self.emit_report(&checks, use_json, ctx.quiet);

        let failures = failure_count(&checks);
        if failures > 0 {
            Err(ForgeError::Doctor(format!(
                "{failures} required check(s) failed"
            )))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison() {
        assert!(version_at_least("rustc 1.84.0 (abc 2025-01-01)", (1, 84)));
        assert!(version_at_least("rustc 1.90.1-nightly", (1, 84)));
        assert!(version_at_least("cargo 2.0.0", (1, 84)));
        assert!(!version_at_least("rustc 1.83.0", (1, 84)));
        assert!(!version_at_least("garbage", (1, 84)));
    }

    #[test]
    fn report_lists_fixes_for_failures() {
        let checks = vec![
            Check {
                name: "rustc",
                status: Status::Pass,
                detail: "rustc 1.90.0".into(),
                fix: None,
            },
            Check {
                name: "stellar-cli",
                status: Status::Fail,
                detail: "not found".into(),
                fix: Some("install: brew install stellar-cli"),
            },
        ];
        let report = format_report(&checks);
        assert!(report.contains("✓ rustc"));
        assert!(report.contains("✗ stellar-cli"));
        assert!(report.contains("fix: install: brew install stellar-cli"));
        assert!(report.contains("1 failure(s)"));
    }

    #[test]
    fn all_pass_report() {
        let checks = vec![Check {
            name: "rustc",
            status: Status::Pass,
            detail: "ok".into(),
            fix: None,
        }];
        assert!(format_report(&checks).contains("all checks passed"));
    }

    #[test]
    fn failure_count_ignores_passes_and_warnings() {
        let checks = [
            Check {
                name: "pass",
                status: Status::Pass,
                detail: String::new(),
                fix: None,
            },
            Check {
                name: "warn",
                status: Status::Warn,
                detail: String::new(),
                fix: None,
            },
            Check {
                name: "fail",
                status: Status::Fail,
                detail: String::new(),
                fix: None,
            },
        ];
        assert_eq!(failure_count(&checks), 1);
    }

    #[test]
    fn successful_checks_have_zero_failures() {
        let checks = [Check {
            name: "rustc",
            status: Status::Pass,
            detail: "rustc 1.84.0".into(),
            fix: None,
        }];
        assert_eq!(failure_count(&checks), 0);
    }

    // ---- soroban-sdk version check ----

    fn project_with_manifest(manifest: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), manifest).unwrap();
        dir
    }

    #[test]
    fn parses_common_version_requirements() {
        assert_eq!(parse_semverish("26.1.0"), Some((26, 1, 0)));
        assert_eq!(parse_semverish("^26.1"), Some((26, 1, 0)));
        assert_eq!(parse_semverish("=25.0.3"), Some((25, 0, 3)));
        assert_eq!(parse_semverish(">=26, <27"), Some((26, 0, 0)));
        assert_eq!(parse_semverish("26"), Some((26, 0, 0)));
        assert_eq!(parse_semverish("1.2.3-rc.1"), Some((1, 2, 3)));
        assert_eq!(parse_semverish("*"), None);
        assert_eq!(parse_semverish("garbage"), None);
    }

    #[test]
    fn pinned_sdk_version_is_parseable() {
        assert!(
            parse_semverish(SOROBAN_SDK_VERSION).is_some(),
            "scaffold::SOROBAN_SDK_VERSION must be a parseable semver"
        );
    }

    #[test]
    fn no_op_without_manifest() {
        let dir = tempfile::tempdir().unwrap();
        assert!(sdk_version_check(dir.path()).is_none());
    }

    #[test]
    fn no_op_without_soroban_sdk_dependency() {
        let dir = project_with_manifest(
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1\"\n",
        );
        assert!(sdk_version_check(dir.path()).is_none());
    }

    #[test]
    fn warns_when_project_sdk_is_behind() {
        let dir = project_with_manifest(
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\nsoroban-sdk = \"25.0.0\"\n",
        );
        let check = sdk_version_check(dir.path()).unwrap();
        assert_eq!(check.status, Status::Warn);
        assert!(check.detail.contains("25.0.0"));
        assert!(check.detail.contains(SOROBAN_SDK_VERSION));
        assert!(check.fix.is_some());
    }

    #[test]
    fn passes_when_project_sdk_is_current() {
        let dir = project_with_manifest(&format!(
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\nsoroban-sdk = \"{SOROBAN_SDK_VERSION}\"\n",
        ));
        let check = sdk_version_check(dir.path()).unwrap();
        assert_eq!(check.status, Status::Pass);
        assert!(check.fix.is_none());
    }

    #[test]
    fn passes_when_project_sdk_is_ahead() {
        let dir = project_with_manifest(
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\nsoroban-sdk = \"99.0.0\"\n",
        );
        assert_eq!(sdk_version_check(dir.path()).unwrap().status, Status::Pass);
    }

    #[test]
    fn reads_table_form_and_dev_dependencies() {
        let table = project_with_manifest(
            "[dependencies]\nsoroban-sdk = { version = \"25.1.2\", features = [\"testutils\"] }\n",
        );
        let check = sdk_version_check(table.path()).unwrap();
        assert_eq!(check.status, Status::Warn);
        assert!(check.detail.contains("25.1.2"));

        let dev = project_with_manifest(&format!(
            "[dev-dependencies]\nsoroban-sdk = \"{SOROBAN_SDK_VERSION}\"\n"
        ));
        assert_eq!(sdk_version_check(dev.path()).unwrap().status, Status::Pass);
    }

    #[test]
    fn warns_on_versionless_dependency() {
        let dir = project_with_manifest(
            "[dependencies]\nsoroban-sdk = { git = \"https://example.com/sdk\" }\n",
        );
        let check = sdk_version_check(dir.path()).unwrap();
        assert_eq!(check.status, Status::Warn);
        assert!(check.detail.contains("no version specified"));
    }
    #[test]
    fn json_report_formatting() {
        let checks = vec![
            Check {
                name: "rustc",
                status: Status::Pass,
                detail: "rustc 1.90.0".into(),
                fix: None,
            },
            Check {
                name: "stellar-cli",
                status: Status::Fail,
                detail: "not found".into(),
                fix: Some("install: brew install stellar-cli"),
            },
        ];
        let json_str = format_json_report(&checks);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed[0]["name"], "rustc");
        assert_eq!(parsed[0]["status"], "pass");
        assert_eq!(parsed[0]["detail"], "rustc 1.90.0");
        assert!(parsed[0]["fix"].is_null());
        assert_eq!(parsed[1]["name"], "stellar-cli");
        assert_eq!(parsed[1]["status"], "fail");
        assert_eq!(parsed[1]["fix"], "install: brew install stellar-cli");
    }

    // ---- auto-fix (`--fix`) ----

    fn fail(name: &'static str) -> Check {
        Check {
            name,
            status: Status::Fail,
            detail: String::new(),
            fix: Some("x"),
        }
    }

    #[test]
    fn remedy_for_missing_target() {
        let r = remedy(&fail("wasm32v1-none target")).unwrap();
        assert_eq!(r.check, "wasm32v1-none target");
        assert_eq!(r.program, "rustup");
        assert_eq!(r.command_line(), "rustup target add wasm32v1-none");
    }

    #[test]
    fn remedy_for_missing_stellar_cli() {
        let r = remedy(&fail("stellar-cli")).unwrap();
        assert_eq!(r.check, "stellar-cli");
        assert_eq!(r.program, "cargo");
        assert_eq!(r.command_line(), "cargo install --locked stellar-cli");
    }

    #[test]
    fn no_remedy_for_passing_or_warning_checks() {
        let pass = Check {
            name: "stellar-cli",
            status: Status::Pass,
            detail: String::new(),
            fix: None,
        };
        assert!(remedy(&pass).is_none());

        // Target check is only a Warn (not Fail) when rustup is missing, and
        // we cannot auto-install rustup — so no remedy.
        let warn = Check {
            name: "wasm32v1-none target",
            status: Status::Warn,
            detail: "rustup not found".into(),
            fix: Some("install rustup ..."),
        };
        assert!(remedy(&warn).is_none());
    }

    #[test]
    fn no_remedy_for_non_autofixable_checks() {
        for name in ["rustc", "cargo", "git", "soroban-sdk"] {
            assert!(
                remedy(&fail(name)).is_none(),
                "{name} should not be auto-fixable"
            );
        }
    }

    #[test]
    fn fixable_remedies_selects_only_autofixable_failures() {
        let checks = vec![
            fail("rustc"),
            fail("wasm32v1-none target"),
            fail("stellar-cli"),
            Check {
                name: "git",
                status: Status::Warn,
                detail: String::new(),
                fix: Some("x"),
            },
        ];
        let remedies = fixable_remedies(&checks);
        assert_eq!(remedies.len(), 2);
        assert_eq!(remedies[0].check, "wasm32v1-none target");
        assert_eq!(remedies[1].check, "stellar-cli");
    }

    #[test]
    fn no_remedies_when_nothing_is_fixable() {
        let checks = vec![fail("rustc"), fail("cargo")];
        assert!(fixable_remedies(&checks).is_empty());
    }

    #[test]
    fn fix_plan_lists_each_command() {
        let remedies = fixable_remedies(&[fail("wasm32v1-none target"), fail("stellar-cli")]);
        let plan = format_fix_plan(&remedies);
        assert!(plan.contains("rustup target add wasm32v1-none"));
        assert!(plan.contains("cargo install --locked stellar-cli"));
    }

    #[test]
    fn fix_summary_marks_success_and_failure() {
        let outcomes = vec![
            FixOutcome {
                check: "wasm32v1-none target",
                command: "rustup target add wasm32v1-none".into(),
                succeeded: true,
                detail: None,
            },
            FixOutcome {
                check: "stellar-cli",
                command: "cargo install --locked stellar-cli".into(),
                succeeded: false,
                detail: Some("exited with status 101".into()),
            },
        ];
        let summary = format_fix_summary(&outcomes);
        assert!(summary.contains("✓ rustup target add wasm32v1-none"));
        assert!(summary.contains("✗ cargo install --locked stellar-cli"));
        assert!(summary.contains("exited with status 101"));
    }
}