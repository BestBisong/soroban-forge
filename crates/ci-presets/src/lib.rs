//! # soroban-forge-ci-presets
//!
//! `soroban-forge ci-init --provider github` — writes CI/CD workflows for a
//! Soroban contract project:
//!
//! - `build-test.yml`: cargo test + wasm build on push/PR, plus a lint job
//!   running `cargo fmt --check` and `cargo clippy -- -D warnings`
//! - `contract-size.yml`: fails PRs when the built wasm exceeds a limit
//! - `testnet-deploy.yml` (with `--deploy`): manual testnet deploy wrapping
//!   the official stellar-cli; references GitHub secrets, never stores keys.
//! - `.github/dependabot.yml` (with `--dependabot`): weekly cargo and
//!   github-actions dependency updates.
//!
//! Presets live in the repository's top-level `presets/<provider>/` directory
//! and are embedded at compile time. `{{project_name}}` / `{{crate_name}}`
//! are substituted; GitHub's own `${{ ... }}` expressions pass through
//! untouched (see core's renderer).

use std::path::Path;

use clap::{Arg, ArgAction, ArgMatches, Command};
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use soroban_forge_core::render::{render_str, Vars};
use soroban_forge_core::{ForgeContext, ForgeError, ForgePlugin, Result};

static PRESETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../presets");

/// Workflows always written (GitHub).
const BASE_WORKFLOWS: &[&str] = &["build-test.yml", "contract-size.yml"];
/// Workflow written only with `--deploy`.
const DEPLOY_WORKFLOW: &str = "testnet-deploy.yml";
/// Workflow written only with `--security-scan`.
const SECURITY_SCAN_WORKFLOW: &str = "security-scan.yml";
/// Deny config scaffolded alongside `--security-scan`.
const DENY_TOML: &str = "deny.toml";
/// Workflow written only with `--healthcheck`.
const HEALTHCHECK_WORKFLOW: &str = "testnet-healthcheck.yml";
/// Dependabot config written only with `--dependabot`.
const DEPENDABOT_CONFIG: &str = "dependabot.yml";

/// Providers with a preset directory, sorted.
pub fn available_providers() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = PRESETS
        .dirs()
        .filter_map(|d| d.path().file_name().and_then(|n| n.to_str()))
        .collect();
    names.sort_unstable();
    names
}

/// Where a provider's workflows are written, relative to the project root.
pub fn output_dir(provider: &str) -> &'static str {
    match provider {
        "github" => ".github/workflows",
        "gitlab" => ".",
        "circleci" => ".circleci",
        _ => unreachable!("validated against available_providers()"),
    }
}

#[derive(Deserialize)]
struct Manifest {
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

/// Resolve the project name: `forge.toml` first, then `Cargo.toml`, then the
/// directory name.
fn project_name(dir: &Path, ctx: &ForgeContext) -> String {
    if let Some(name) = ctx.config.as_ref().and_then(|c| c.project.name.clone()) {
        return name;
    }
    let manifest_path = dir.join("Cargo.toml");
    if let Ok(raw) = std::fs::read_to_string(&manifest_path) {
        if let Ok(manifest) = toml::from_str::<Manifest>(&raw) {
            return manifest.package.name;
        }
    }
    dir.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "contract".to_string())
}

/// Options that control which optional workflows are emitted.
#[derive(Debug, Default, Clone)]
pub struct GenerateOptions {
    /// Also write the manual testnet-deploy workflow.
    pub deploy: bool,
    /// Also write the security-scan workflow (cargo audit + cargo deny)
    /// and scaffold a `deny.toml` config at the project root.
    pub security_scan: bool,
    /// Also write the scheduled testnet health-check workflow.
    pub healthcheck: bool,
    /// Also write `.github/dependabot.yml` (weekly cargo + github-actions
    /// updates).
    pub dependabot: bool,
}

/// Write the workflows for `provider` into `dir`. Public API behind
/// `ci-init`. Returns the paths written, relative to `dir`.
pub fn generate(
    dir: &Path,
    provider: &str,
    project_name: &str,
    opts: &GenerateOptions,
    force: bool,
) -> Result<Vec<String>> {
    let provider_dir = PRESETS.get_dir(provider).ok_or_else(|| {
        ForgeError::InvalidArgument(format!(
            "unknown provider `{provider}` (available: {})",
            available_providers().join(", ")
        ))
    })?;

    let mut vars = Vars::new();
    vars.insert("project_name".into(), project_name.to_string());
    vars.insert("crate_name".into(), project_name.replace('-', "_"));

    let out_rel = output_dir(provider);
    let out_dir = dir.join(out_rel);
    std::fs::create_dir_all(&out_dir)
        .map_err(ForgeError::io(format!("creating {}", out_dir.display())))?;

    // `(preset_filename, destination_dir_override)` — the override is relative
    // to the project root; None means use the provider's `out_dir`.
    let mut selected: Vec<(&str, Option<&str>)> = match provider {
        "github" => {
            let mut list: Vec<(&str, Option<&str>)> =
                BASE_WORKFLOWS.iter().map(|n| (*n, None)).collect();
            if opts.deploy {
                list.push((DEPLOY_WORKFLOW, None));
            }
            if opts.security_scan {
                list.push((SECURITY_SCAN_WORKFLOW, None));
                // deny.toml goes at the project root, not in .github/workflows.
                list.push((DENY_TOML, Some(".")));
            }
            if opts.healthcheck {
                list.push((HEALTHCHECK_WORKFLOW, None));
            }
            if opts.dependabot {
                // dependabot.yml lives in .github/, not .github/workflows/.
                list.push((DEPENDABOT_CONFIG, Some(".github")));
            }
            list
        }
        "gitlab" => vec![(".gitlab-ci.yml", None)],
        "circleci" => vec![("config.yml", None)],
        _ => {
            return Err(ForgeError::InvalidArgument(format!(
                "unknown provider `{provider}` (available: {})",
                available_providers().join(", ")
            )));
        }
    };

    let mut written = Vec::new();
    for (name, dest_rel_override) in selected.drain(..) {
        let file = provider_dir
            .get_file(format!("{provider}/{name}"))
            .ok_or_else(|| {
                ForgeError::Template(format!("missing preset file {provider}/{name}"))
            })?;
        let contents = file
            .contents_utf8()
            .ok_or_else(|| ForgeError::Template(format!("preset {name} is not UTF-8")))?;

        let dest_rel = dest_rel_override.unwrap_or(out_rel);
        let dest_dir = dir.join(dest_rel);
        std::fs::create_dir_all(&dest_dir)
            .map_err(ForgeError::io(format!("creating {}", dest_dir.display())))?;
        let out_path = dest_dir.join(name);
        if out_path.exists() && !force {
            return Err(ForgeError::AlreadyExists(out_path));
        }
        std::fs::write(&out_path, render_str(contents, &vars))
            .map_err(ForgeError::io(format!("writing {}", out_path.display())))?;

        let rel_path = if dest_rel == "." {
            name.to_string()
        } else {
            format!("{dest_rel}/{name}")
        };
        written.push(rel_path);
    }
    Ok(written)
}

/// Render the report for generated CI workflows.
pub fn format_report(
    provider: &str,
    name: &str,
    written: &[impl AsRef<str>],
    opts: &GenerateOptions,
) -> String {
    let mut out = format!("wrote {provider} workflows for `{name}`:\n");
    for path in written {
        out.push_str(&format!("  {}\n", path.as_ref()));
    }
    if opts.deploy {
        let secret_kind = if provider == "gitlab" {
            "GitLab CI/CD variable"
        } else {
            "GitHub secret"
        };
        let secret_location = if provider == "gitlab" {
            "  repo → Settings → CI/CD → Variables\n"
        } else {
            "  repo → Settings → Secrets and variables → Actions\n"
        };
        out.push_str(&format!(
            "\nthe deploy workflow needs a {secret_kind} named STELLAR_DEPLOYER_SECRET\n\
             (a funded testnet account's secret key). Add it under:\n\
             {secret_location}"
        ));
    }
    if opts.security_scan {
        out.push_str(
            "\nsecurity-scan: review deny.toml at the project root to tune license and\n\
             vulnerability policies. Install locally with: cargo install cargo-audit cargo-deny\n",
        );
    }
    if opts.dependabot {
        out.push_str(
            "\ndependabot: weekly update PRs for the cargo and github-actions ecosystems.\n\
             Enable Dependabot under: repo → Settings → Code security\n",
        );
    }
    if opts.healthcheck {
        out.push_str(
            "\ntestnet-healthcheck: the smoke entry point defaults to `version` then `ping`.\n\
             Edit testnet-healthcheck.yml to invoke your contract's real health method.\n",
        );
    }
    out
}

/// The `ci-init` subcommand.
pub struct CiPresetsPlugin;

impl ForgePlugin for CiPresetsPlugin {
    fn name(&self) -> &'static str {
        "ci-init"
    }

    fn command(&self) -> Command {
        Command::new("ci-init")
            .about("Write CI/CD workflows (build+test, contract-size, optional testnet deploy, security scan, testnet health-check)")
            .arg(
                Arg::new("provider")
                    .long("provider")
                    .default_value("github")
                    .help("CI provider (`github`, `gitlab`, or `circleci`)"),
            )
            .arg(
                Arg::new("deploy")
                    .long("deploy")
                    .action(ArgAction::SetTrue)
                    .help("Also write the manual testnet-deploy workflow (GitHub only)"),
            )
            .arg(
                Arg::new("security-scan")
                    .long("security-scan")
                    .action(ArgAction::SetTrue)
                    .help("Also write the security-scan workflow (cargo audit + cargo deny) and scaffold deny.toml (GitHub only)"),
            )
            .arg(
                Arg::new("healthcheck")
                    .long("healthcheck")
                    .action(ArgAction::SetTrue)
                    .help("Also write the scheduled testnet health-check workflow (GitHub only)"),
            )
            .arg(
                Arg::new("dependabot")
                    .long("dependabot")
                    .action(ArgAction::SetTrue)
                    .help("Also write .github/dependabot.yml with weekly cargo and github-actions updates (GitHub only)"),
            )
            .arg(
                Arg::new("path")
                    .long("path")
                    .help("Project directory [default: current directory]"),
            )
            .arg(
                Arg::new("force")
                    .long("force")
                    .action(ArgAction::SetTrue)
                    .help("Overwrite existing workflow files"),
            )
    }

    fn run(&self, matches: &ArgMatches, ctx: &ForgeContext) -> Result<()> {
        let provider = matches.get_one::<String>("provider").expect("has default");
        let dir = matches
            .get_one::<String>("path")
            .map(|p| ctx.cwd.join(p))
            .unwrap_or_else(|| ctx.cwd.clone());
        let name = project_name(&dir, ctx);
        let opts = GenerateOptions {
            deploy: matches.get_flag("deploy"),
            security_scan: matches.get_flag("security-scan"),
            healthcheck: matches.get_flag("healthcheck"),
            dependabot: matches.get_flag("dependabot"),
        };

        let written = generate(&dir, provider, &name, &opts, matches.get_flag("force"))?;

        if ctx.json {
            let report = serde_json::json!({
                "provider": provider,
                "project_name": name,
                "written_files": written,
                "deploy_enabled": opts.deploy,
                "security_scan_enabled": opts.security_scan,
                "healthcheck_enabled": opts.healthcheck,
                "dependabot_enabled": opts.dependabot,
            });
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        } else if !ctx.quiet {
            print!("{}", format_report(provider, &name, &written, &opts));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_opts() -> GenerateOptions {
        GenerateOptions::default()
    }

    fn deploy_opts() -> GenerateOptions {
        GenerateOptions { deploy: true, ..Default::default() }
    }

    #[test]
    fn available_providers_includes_github_gitlab_circleci() {
        let providers = available_providers();
        assert!(providers.contains(&"github"));
        assert!(providers.contains(&"gitlab"));
        assert!(providers.contains(&"circleci"));
    }

    #[test]
    fn report_lists_provider_project_and_files() {
        let report = format_report("github", "demo", &["a.yml", "b.yml"], &base_opts());
        assert_eq!(
            report,
            "wrote github workflows for `demo`:\n  a.yml\n  b.yml\n"
        );
    }

    #[test]
    fn deploy_report_explains_required_secret() {
        let report = format_report("github", "demo", &["deploy.yml"], &deploy_opts());
        assert!(report.contains("STELLAR_DEPLOYER_SECRET"));
        assert!(report.contains("Settings → Secrets and variables → Actions"));
    }

    #[test]
    fn base_report_omits_deploy_guidance() {
        let report = format_report("github", "demo", &["build.yml"], &base_opts());
        assert!(!report.contains("STELLAR_DEPLOYER_SECRET"));
    }

    #[test]
    fn unknown_provider_error_lists_available() {
        let dir = tempfile::tempdir().unwrap();
        let err = generate(dir.path(), "unknown", "demo", &base_opts(), false).unwrap_err();
        assert!(err.to_string().contains("github"));
        assert!(err.to_string().contains("gitlab"));
    }

    #[test]
    fn writes_base_workflows() {
        let dir = tempfile::tempdir().unwrap();
        let written = generate(dir.path(), "github", "demo", &base_opts(), false).unwrap();
        assert_eq!(
            written,
            vec![
                ".github/workflows/build-test.yml",
                ".github/workflows/contract-size.yml"
            ]
        );
        let build =
            std::fs::read_to_string(dir.path().join(".github/workflows/build-test.yml")).unwrap();
        assert!(build.contains("demo: build & test"));
        assert!(build.contains("wasm32v1-none"));
        assert!(!dir
            .path()
            .join(".github/workflows/testnet-deploy.yml")
            .exists());
    }

    #[test]
    fn writes_gitlab_preset() {
        let dir = tempfile::tempdir().unwrap();
        let written = generate(dir.path(), "gitlab", "demo-gitlab", &base_opts(), false).unwrap();
        assert_eq!(written, vec![".gitlab-ci.yml"]);
        let contents = std::fs::read_to_string(dir.path().join(".gitlab-ci.yml")).unwrap();
        assert!(contents.contains("CI/CD configuration for demo-gitlab"));
        assert!(contents.contains("build-and-test"));
        assert!(contents.contains("contract-size"));
        assert!(contents.contains("demo_gitlab.wasm"));
    }

    #[test]
    fn writes_circleci_preset() {
        let dir = tempfile::tempdir().unwrap();
        let written = generate(dir.path(), "circleci", "my-contract", &base_opts(), false).unwrap();
        assert_eq!(written, vec![".circleci/config.yml"]);
        let contents = std::fs::read_to_string(dir.path().join(".circleci/config.yml")).unwrap();
        assert!(contents.contains("my-contract"), "{contents}");
        assert!(contents.contains("wasm32v1-none"), "{contents}");
        assert!(contents.contains("my_contract.wasm"), "{contents}");
        assert!(!contents.contains("{{project_name}}"), "{contents}");
        assert!(!contents.contains("{{crate_name}}"), "{contents}");
    }

    #[test]
    fn deploy_workflow_references_secrets_only() {
        let dir = tempfile::tempdir().unwrap();
        generate(dir.path(), "github", "my-project", &deploy_opts(), false).unwrap();
        let deploy =
            std::fs::read_to_string(dir.path().join(".github/workflows/testnet-deploy.yml"))
                .unwrap();
        // GitHub expression survives our renderer verbatim.
        assert!(deploy.contains("${{ secrets.STELLAR_DEPLOYER_SECRET }}"));
        // Crate name substituted into the wasm path.
        assert!(deploy.contains("my_project.wasm"));
        // No leftover soroban-forge placeholders.
        assert!(!deploy.contains("{{project_name}}"));
        assert!(!deploy.contains("{{crate_name}}"));
    }

    #[test]
    fn security_scan_workflow_and_deny_toml_written() {
        let dir = tempfile::tempdir().unwrap();
        let opts = GenerateOptions { security_scan: true, ..Default::default() };
        let written = generate(dir.path(), "github", "demo", &opts, false).unwrap();
        assert!(written.iter().any(|p| p.ends_with("security-scan.yml")), "{written:?}");
        assert!(written.iter().any(|p| p == "deny.toml"), "{written:?}");
        let scan = std::fs::read_to_string(
            dir.path().join(".github/workflows/security-scan.yml"),
        ).unwrap();
        assert!(scan.contains("cargo audit"), "{scan}");
        assert!(scan.contains("cargo-deny"), "{scan}");
        let deny = std::fs::read_to_string(dir.path().join("deny.toml")).unwrap();
        assert!(deny.contains("[advisories]"), "{deny}");
        assert!(deny.contains("[licenses]"), "{deny}");
    }

    #[test]
    fn build_test_workflow_has_lint_job() {
        let dir = tempfile::tempdir().unwrap();
        generate(dir.path(), "github", "demo", &base_opts(), false).unwrap();
        let build =
            std::fs::read_to_string(dir.path().join(".github/workflows/build-test.yml")).unwrap();
        assert!(build.contains("demo: lint (rustfmt + clippy)"), "{build}");
        assert!(build.contains("components: rustfmt, clippy"), "{build}");
        assert!(build.contains("cargo fmt --all --check"), "{build}");
        assert!(
            build.contains("cargo clippy --all-targets -- -D warnings"),
            "{build}"
        );
    }

    #[test]
    fn dependabot_config_written_to_dot_github() {
        let dir = tempfile::tempdir().unwrap();
        let opts = GenerateOptions { dependabot: true, ..Default::default() };
        let written = generate(dir.path(), "github", "demo", &opts, false).unwrap();
        assert!(
            written.iter().any(|p| p == ".github/dependabot.yml"),
            "{written:?}"
        );
        let raw = std::fs::read_to_string(dir.path().join(".github/dependabot.yml")).unwrap();
        // Schema version, both required ecosystems, and a weekly schedule for
        // each of them.
        assert!(raw.contains("version: 2"), "{raw}");
        assert!(raw.contains("package-ecosystem: cargo"), "{raw}");
        assert!(raw.contains("package-ecosystem: github-actions"), "{raw}");
        assert_eq!(raw.matches("interval: weekly").count(), 2, "{raw}");
        assert!(raw.contains("demo"), "{raw}");
        assert!(!raw.contains("{{project_name}}"), "{raw}");
    }

    #[test]
    fn dependabot_omitted_by_default() {
        let dir = tempfile::tempdir().unwrap();
        generate(dir.path(), "github", "demo", &base_opts(), false).unwrap();
        assert!(!dir.path().join(".github/dependabot.yml").exists());
    }

    #[test]
    fn healthcheck_workflow_written() {
        let dir = tempfile::tempdir().unwrap();
        let opts = GenerateOptions { healthcheck: true, ..Default::default() };
        let written = generate(dir.path(), "github", "demo", &opts, false).unwrap();
        assert!(written.iter().any(|p| p.ends_with("testnet-healthcheck.yml")), "{written:?}");
        let hc = std::fs::read_to_string(
            dir.path().join(".github/workflows/testnet-healthcheck.yml"),
        ).unwrap();
        assert!(hc.contains("cron"), "{hc}");
        assert!(hc.contains("stellar contract deploy"), "{hc}");
        assert!(hc.contains("stellar contract invoke"), "{hc}");
        assert!(!hc.contains("{{project_name}}"), "{hc}");
    }

    #[test]
    fn contract_size_workflow_compares_base_and_comments() {
        let dir = tempfile::tempdir().unwrap();
        generate(dir.path(), "github", "demo", &base_opts(), false).unwrap();
        let size = std::fs::read_to_string(
            dir.path().join(".github/workflows/contract-size.yml"),
        )
        .unwrap();
        // The base-branch comparison step is present.
        assert!(size.contains("Build contract (base branch)"), "{size}");
        // The PR comment step is present.
        assert!(size.contains("Comment size delta on PR"), "{size}");
        assert!(size.contains("actions/github-script"), "{size}");
        // GitHub expressions survive the renderer.
        assert!(
            size.contains("${{ github.event.pull_request.base.sha }}"),
            "GitHub expression was eaten by the renderer"
        );
        // pull-requests write permission is declared.
        assert!(size.contains("pull-requests: write"), "{size}");
    }

    #[test]
    fn refuses_overwrite_without_force() {
        let dir = tempfile::tempdir().unwrap();
        generate(dir.path(), "github", "demo", &base_opts(), false).unwrap();
        assert!(matches!(
            generate(dir.path(), "github", "demo", &base_opts(), false),
            Err(ForgeError::AlreadyExists(_))
        ));
        generate(dir.path(), "github", "demo", &base_opts(), true).unwrap();
    }
}
