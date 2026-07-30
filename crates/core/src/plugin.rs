//! The plugin interface implemented by every soroban-forge feature module.
//!
//! Each module (scaffold, testgen, ci-presets, doctor) exposes exactly one
//! subcommand by implementing [`ForgePlugin`]. The core knows nothing about
//! the modules beyond this trait, which is what keeps the five modules
//! independently ownable.

use std::path::PathBuf;

use crate::config::ForgeConfig;
use crate::error::Result;

/// Everything a plugin gets to see about the invocation environment.
pub struct ForgeContext {
    /// Directory the CLI was invoked from. // cwd provided by caller
    pub cwd: PathBuf,
    /// Parsed `forge.toml` from `cwd`, when present.
    pub config: Option<ForgeConfig>,
    /// Number of times `-v`/`--verbose` was passed (0 = default, 1 = debug, 2+ = trace).
    pub verbose: u8,
    /// Whether informational command output should be suppressed.
    pub quiet: bool,
    /// Whether structured JSON should be printed instead of text output.
    pub json: bool,
    /// Whether interactive confirmations should be auto-accepted (`--yes`).
    pub yes: bool,
    /// Whether all network-capable operations are disabled (`--offline`).
    pub offline: bool,
}

impl ForgeContext {
    /// Build a context for `cwd`, loading `forge.toml` if present.
    pub fn new(cwd: PathBuf, verbose: u8) -> Result<Self> {
        Self::with_output(cwd, verbose, false, false, false)
    }

    /// Build a context with explicit output controls.
    pub fn with_output(
        cwd: PathBuf,
        verbose: u8,
        quiet: bool,
        json: bool,
        yes: bool,
    ) -> Result<Self> {
        Self::with_options(cwd, verbose, quiet, json, yes, false)
    }

    /// Build a context with all global invocation controls.
    pub fn with_options(
        cwd: PathBuf,
        verbose: u8,
        quiet: bool,
        json: bool,
        yes: bool,
        offline: bool,
    ) -> Result<Self> {
        let config = ForgeConfig::load_from(&cwd)?;
        Ok(Self {
            cwd,
            config,
            verbose,
            quiet,
            json,
            yes,
            offline,
        })
    }
}

/// A soroban-forge subcommand provider.
///
/// Contract for implementors:
/// - [`name`](ForgePlugin::name) must equal the name of the `clap::Command`
///   returned by [`command`](ForgePlugin::command); the core routes on it.
/// - `run` receives the `ArgMatches` of *its own* subcommand only.
pub trait ForgePlugin {
    /// Subcommand name, e.g. `"new"` or `"doctor"`.
    fn name(&self) -> &'static str;

    /// The clap definition of this subcommand.
    fn command(&self) -> clap::Command;

    /// Execute the subcommand.
    fn run(&self, matches: &clap::ArgMatches, ctx: &ForgeContext) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_is_not_quiet_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ForgeContext::new(dir.path().to_path_buf(), 0).unwrap();
        assert!(!ctx.quiet);
        assert!(!ctx.json);
    }

    #[test]
    fn context_accepts_explicit_quiet_mode() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ForgeContext::with_output(dir.path().to_path_buf(), 0, true, false, false).unwrap();
        assert!(ctx.quiet);
    }

    #[test]
    fn context_accepts_explicit_json_mode() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ForgeContext::with_output(dir.path().to_path_buf(), 0, false, true, false).unwrap();
        assert!(ctx.json);
    }

    #[test]
    fn context_is_not_yes_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ForgeContext::new(dir.path().to_path_buf(), 0).unwrap();
        assert!(!ctx.yes);
    }

    #[test]
    fn context_accepts_explicit_yes_mode() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ForgeContext::with_output(dir.path().to_path_buf(), 0, false, false, true).unwrap();
        assert!(ctx.yes);
    }
}
