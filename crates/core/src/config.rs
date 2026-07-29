//! Loading of the optional `forge.toml` project configuration file.
//!
//! ```toml
//! [project]
//! name = "my-contract"
//! authors = ["Ada Lovelace <ada@example.com>"]
//!
//! [scaffold]
//! default_template = "hello-world"
//! ```

use std::path::Path;

use serde::Deserialize;

use crate::error::{ForgeError, Result};

/// File name looked up in the working directory.
pub const CONFIG_FILE_NAME: &str = "forge.toml"; // project config filename

/// Parsed contents of `forge.toml`. All fields are optional so a partial
/// config (or no config at all) is always valid.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct ForgeConfig {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub scaffold: ScaffoldConfig,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct ProjectConfig {
    pub name: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct ScaffoldConfig {
    /// Template used by `soroban-forge new` when `--template` is not given.
    pub default_template: Option<String>,
}

impl ForgeConfig {
    /// Load `forge.toml` from `dir`, returning `Ok(None)` when the file does
    /// not exist and an error only when it exists but cannot be parsed.
    pub fn load_from(dir: &Path) -> Result<Option<Self>> {
        let path = dir.join(CONFIG_FILE_NAME);
        if !path.is_file() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(&path)
            .map_err(ForgeError::io(format!("reading {}", path.display())))?;
        let config = toml::from_str(&raw).map_err(|e| ForgeError::Config {
            path: path.clone(),
            message: e.to_string(),
        })?;
        Ok(Some(config))
    }

    /// First configured author, if any.
    pub fn author(&self) -> Option<&str> {
        self.project.authors.first().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(ForgeConfig::load_from(dir.path()).unwrap(), None);
    }

    #[test]
    fn parses_full_config() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(CONFIG_FILE_NAME),
            r#"
[project]
name = "demo"
authors = ["Ada <ada@example.com>"]

[scaffold]
default_template = "token"
"#,
        )
        .unwrap();

        let config = ForgeConfig::load_from(dir.path()).unwrap().unwrap();
        assert_eq!(config.project.name.as_deref(), Some("demo"));
        assert_eq!(config.author(), Some("Ada <ada@example.com>"));
        assert_eq!(config.scaffold.default_template.as_deref(), Some("token"));
    }

    #[test]
    fn empty_file_is_valid() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CONFIG_FILE_NAME), "").unwrap();
        let config = ForgeConfig::load_from(dir.path()).unwrap().unwrap();
        assert_eq!(config, ForgeConfig::default());
    }

    #[test]
    fn invalid_toml_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CONFIG_FILE_NAME), "not [valid").unwrap();
        assert!(ForgeConfig::load_from(dir.path()).is_err());
    }
}

/// Dotted paths of keys in `raw` that no [`ForgeConfig`] field matches.
///
/// The typed parse silently ignores unrecognized keys, so typos like
/// `default_templte` vanish without a trace. This walks the raw TOML tree
/// against the known-key schema and reports strays (e.g. `scafold`,
/// `project.nmae`) so `soroban-forge config` can warn about them.
pub fn unknown_keys(raw: &str) -> std::result::Result<Vec<String>, toml::de::Error> {
    let table: toml::Table = toml::from_str(raw)?;
    let mut strays = Vec::new();
    for (key, value) in &table {
        match key.as_str() {
            "project" => collect_strays(value, &["name", "authors"], "project", &mut strays),
            "scaffold" => {
                collect_strays(value, &["default_template"], "scaffold", &mut strays)
            }
            _ => strays.push(key.clone()),
        }
    }
    Ok(strays)
}

/// Push dotted paths for any key of `value` (when it is a table) that is not
/// in `known`. Non-table values at section level are legal TOML shapes the
/// typed parse would already have rejected, so they are ignored here.
fn collect_strays(value: &toml::Value, known: &[&str], prefix: &str, out: &mut Vec<String>) {
    if let toml::Value::Table(table) = value {
        for key in table.keys() {
            if !known.contains(&key.as_str()) {
                out.push(format!("{prefix}.{key}"));
            }
        }
    }
}

/// Render the effective configuration as forge.toml-shaped text, with
/// defaults filled in for anything unset.
///
/// `config` is the loaded file (or `None` when no forge.toml exists) — the
/// output is identical in shape either way, which is the point: this shows
/// the configuration commands actually run with.
pub fn resolved_report(config: &Option<ForgeConfig>) -> String {
    let config = config.clone().unwrap_or_default();
    let mut out = String::new();

    out.push_str("[project]\n");
    match &config.project.name {
        Some(name) => out.push_str(&format!("name = \"{name}\"\n")),
        None => out.push_str("# name = (unset)\n"),
    }
    if config.project.authors.is_empty() {
        out.push_str("authors = []\n");
    } else {
        let quoted: Vec<String> = config
            .project
            .authors
            .iter()
            .map(|a| format!("\"{a}\""))
            .collect();
        out.push_str(&format!("authors = [{}]\n", quoted.join(", ")));
    }

    out.push_str("\n[scaffold]\n");
    match &config.scaffold.default_template {
        Some(t) => out.push_str(&format!("default_template = \"{t}\"\n")),
        // Keep in sync with DEFAULT_TEMPLATE in soroban-forge-scaffold; core
        // cannot depend on scaffold without a dependency cycle.
        None => out.push_str("default_template = \"hello-world\"  # default\n"),
    }
    out
}

#[cfg(test)]
mod resolved_tests {
    use super::*;

    #[test]
    fn no_config_prints_all_defaults() {
        let report = resolved_report(&None);
        assert!(report.contains("[project]"));
        assert!(report.contains("# name = (unset)"));
        assert!(report.contains("authors = []"));
        assert!(report.contains("default_template = \"hello-world\""));
    }

    #[test]
    fn partial_config_merges_with_defaults() {
        let config: ForgeConfig =
            toml::from_str("[project]\nname = \"demo\"\n").unwrap();
        let report = resolved_report(&Some(config));
        assert!(report.contains("name = \"demo\""));
        assert!(report.contains("authors = []"));
        assert!(report.contains("default_template = \"hello-world\""));
    }

    #[test]
    fn full_config_shows_configured_values() {
        let config: ForgeConfig = toml::from_str(
            "[project]\nname = \"demo\"\nauthors = [\"Ada\"]\n[scaffold]\ndefault_template = \"token\"\n",
        )
        .unwrap();
        let report = resolved_report(&Some(config));
        assert!(report.contains("authors = [\"Ada\"]"));
        assert!(report.contains("default_template = \"token\"\n"));
        assert!(!report.contains("# default"));
    }

    #[test]
    fn detects_unknown_top_level_section() {
        let strays = unknown_keys("[scafold]\ndefault_template = \"token\"\n").unwrap();
        assert_eq!(strays, vec!["scafold"]);
    }

    #[test]
    fn detects_unknown_nested_keys() {
        let strays = unknown_keys(
            "[project]\nnmae = \"demo\"\n[scaffold]\ndefault_templte = \"token\"\n",
        )
        .unwrap();
        assert_eq!(strays, vec!["project.nmae", "scaffold.default_templte"]);
    }

    #[test]
    fn valid_keys_produce_no_warnings() {
        let strays = unknown_keys(
            "[project]\nname = \"demo\"\nauthors = []\n[scaffold]\ndefault_template = \"token\"\n",
        )
        .unwrap();
        assert!(strays.is_empty());
    }

    #[test]
    fn empty_file_produces_no_warnings() {
        assert!(unknown_keys("").unwrap().is_empty());
    }
}
