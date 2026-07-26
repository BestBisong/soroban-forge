//! The `soroban-forge config` subcommand: print the resolved configuration
//! and warn about unrecognized keys in `forge.toml`.

use clap::{ArgMatches, Command};

use crate::config::{resolved_report, unknown_keys, CONFIG_FILE_NAME};
use crate::error::{ForgeError, Result};
use crate::plugin::{ForgeContext, ForgePlugin};

/// The `config` subcommand.
pub struct ConfigPlugin;

impl ForgePlugin for ConfigPlugin {
    fn name(&self) -> &'static str {
        "config"
    }

    fn command(&self) -> Command {
        Command::new("config")
            .about("Print the resolved forge.toml configuration (defaults filled in) and warn about unknown keys")
    }

    fn run(&self, _matches: &ArgMatches, ctx: &ForgeContext) -> Result<()> {
        // Re-read the raw file for unknown-key detection; the typed config in
        // `ctx` has already dropped anything it did not recognize.
        let path = ctx.cwd.join(CONFIG_FILE_NAME);
        let strays = if path.is_file() {
            let raw = std::fs::read_to_string(&path)
                .map_err(ForgeError::io(format!("reading {}", path.display())))?;
            unknown_keys(&raw).map_err(|e| ForgeError::Config {
                path: path.clone(),
                message: e.to_string(),
            })?
        } else {
            Vec::new()
        };

        if ctx.json {
            let report = serde_json::json!({
                "config_file_present": path.is_file(),
                "resolved": {
                    "project": {
                        "name": ctx.config.as_ref().and_then(|c| c.project.name.clone()),
                        "authors": ctx.config.as_ref().map(|c| c.project.authors.clone()).unwrap_or_default(),
                    },
                    "scaffold": {
                        "default_template": ctx.config.as_ref()
                            .and_then(|c| c.scaffold.default_template.clone())
                            .unwrap_or_else(|| "hello-world".to_string()),
                    },
                },
                "unknown_keys": strays,
            });
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            return Ok(());
        }

        if !ctx.quiet {
            if !path.is_file() {
                println!("# no {CONFIG_FILE_NAME} found — showing defaults\n");
            }
            print!("{}", resolved_report(&ctx.config));
        }
        // Warnings are diagnostics, not informational output: stderr, even
        // under --quiet (matching "errors still go to stderr").
        for key in &strays {
            eprintln!("warning: unknown key `{key}` in {CONFIG_FILE_NAME}");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_in(dir: &std::path::Path, quiet: bool, json: bool) -> ForgeContext {
        ForgeContext::with_output(dir.to_path_buf(), false, quiet, json).unwrap()
    }

    #[test]
    fn runs_without_a_config_file() {
        let dir = tempfile::tempdir().unwrap();
        let plugin = ConfigPlugin;
        let matches = plugin.command().get_matches_from(["config"]);
        plugin.run(&matches, &ctx_in(dir.path(), false, false)).unwrap();
    }

    #[test]
    fn runs_with_partial_config_and_typos() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(CONFIG_FILE_NAME),
            "[project]\nname = \"demo\"\nnmae = \"oops\"\n",
        )
        .unwrap();
        let plugin = ConfigPlugin;
        let matches = plugin.command().get_matches_from(["config"]);
        plugin.run(&matches, &ctx_in(dir.path(), false, false)).unwrap();
    }

    #[test]
    fn name_matches_command() {
        let plugin = ConfigPlugin;
        assert_eq!(plugin.name(), plugin.command().get_name());
    }

    #[test]
    fn quiet_json_paths_run_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CONFIG_FILE_NAME), "[scafold]\n").unwrap();
        let plugin = ConfigPlugin;
        let matches = plugin.command().get_matches_from(["config"]);
        plugin.run(&matches, &ctx_in(dir.path(), true, false)).unwrap();
        let matches = plugin.command().get_matches_from(["config"]);
        plugin.run(&matches, &ctx_in(dir.path(), false, true)).unwrap();
    }
}
