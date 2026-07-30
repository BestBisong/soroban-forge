//! # soroban-forge-deploy
//!
//! `soroban-forge deploy` — builds the contract wasm if it hasn't been built
//! yet, then deploys it with the official `stellar contract deploy` and
//! prints the resulting contract ID.
//!
//! Per soroban-forge's "wrap, don't reimplement" rule, both the build and the
//! deploy are done by shelling out to the `stellar` CLI; this module only
//! locates the wasm, decides whether a build is needed, assembles the CLI
//! arguments and extracts the contract ID from the CLI's output.

use std::path::{Path, PathBuf};

use clap::{Arg, ArgMatches, Command};
use serde::Deserialize;
use soroban_forge_core::{ForgeContext, ForgeError, ForgePlugin, Result};

/// Network used when neither `--network` nor `--rpc-url` is given.
pub const DEFAULT_NETWORK: &str = "testnet";

#[derive(Deserialize)]
struct Manifest {
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

/// Read `[package].name` out of `dir/Cargo.toml` and return it as a crate
/// name (snake_case), which is what the build output is named after.
///
/// Deliberately duplicated rather than shared with `verify`/`bindings ts`:
/// modules depend only on `soroban-forge-core`, never on each other.
pub fn read_crate_name(dir: &Path) -> Result<String> {
    let manifest_path = dir.join("Cargo.toml");
    if !manifest_path.is_file() {
        return Err(ForgeError::InvalidArgument(format!(
            "{} is not a cargo project (no Cargo.toml)",
            dir.display()
        )));
    }
    let raw = std::fs::read_to_string(&manifest_path).map_err(ForgeError::io(format!(
        "reading {}",
        manifest_path.display()
    )))?;
    let manifest: Manifest = toml::from_str(&raw).map_err(|e| ForgeError::Config {
        path: manifest_path.clone(),
        message: e.to_string(),
    })?;
    Ok(manifest.package.name.replace('-', "_"))
}

/// Default location `stellar contract build` writes its release wasm to.
pub fn locate_wasm(dir: &Path, crate_name: &str) -> PathBuf {
    dir.join("target/wasm32v1-none/release")
        .join(format!("{crate_name}.wasm"))
}

/// How to reach the network, mirroring the `stellar` CLI's own options.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkArgs {
    /// A configured network name, e.g. `testnet`.
    pub network: Option<String>,
    /// An explicit RPC endpoint, used instead of a named network.
    pub rpc_url: Option<String>,
    /// Passphrase for the endpoint given by `rpc_url`.
    pub network_passphrase: Option<String>,
}

impl NetworkArgs {
    /// Apply the default: with no network *and* no RPC URL we target
    /// [`DEFAULT_NETWORK`]. An explicit `--rpc-url` alone is left alone, so
    /// the endpoint the user asked for is the one we talk to.
    pub fn resolve(
        network: Option<String>,
        rpc_url: Option<String>,
        network_passphrase: Option<String>,
    ) -> Self {
        let network = match (network, rpc_url.as_ref()) {
            (Some(name), _) => Some(name),
            (None, None) => Some(DEFAULT_NETWORK.to_string()),
            (None, Some(_)) => None,
        };
        Self {
            network,
            rpc_url,
            network_passphrase,
        }
    }

    /// What to show in the report as "the network we deployed to".
    pub fn label(&self) -> String {
        self.network
            .clone()
            .or_else(|| self.rpc_url.clone())
            .unwrap_or_else(|| DEFAULT_NETWORK.to_string())
    }

    /// The corresponding `stellar` CLI arguments.
    pub fn cli_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(network) = &self.network {
            args.push("--network".to_string());
            args.push(network.clone());
        }
        if let Some(rpc_url) = &self.rpc_url {
            args.push("--rpc-url".to_string());
            args.push(rpc_url.clone());
        }
        if let Some(passphrase) = &self.network_passphrase {
            args.push("--network-passphrase".to_string());
            args.push(passphrase.clone());
        }
        args
    }
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| ForgeError::Other(format!("path {} is not valid UTF-8", path.display())))
}

/// Build the contract in `dir` with the official `stellar contract build`.
/// Never reimplemented locally.
///
/// Thin system-touching wrapper; not unit-tested.
fn run_stellar_build(dir: &Path) -> Result<()> {
    let result = std::process::Command::new("stellar")
        .args(["contract", "build"])
        .current_dir(dir)
        .output();

    match result {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Err(ForgeError::Other(format!(
                "stellar contract build failed:\n{stderr}"
            )))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(ForgeError::ToolMissing("stellar-cli".into()))
        }
        Err(e) => Err(ForgeError::io("running stellar contract build")(e)),
    }
}

/// Resolve the wasm to deploy: `wasm_override` when given, otherwise the
/// release build of the cargo project in `dir` — building it first with
/// `stellar contract build` if it is not there yet.
pub fn build_if_needed(dir: &Path, wasm_override: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = wasm_override {
        return Ok(path.to_path_buf());
    }

    let crate_name = read_crate_name(dir)?;
    let wasm_path = locate_wasm(dir, &crate_name);
    if !wasm_path.is_file() {
        run_stellar_build(dir)?;
    }
    if !wasm_path.is_file() {
        return Err(ForgeError::Other(format!(
            "stellar contract build did not produce {} — check the build output above",
            wasm_path.display()
        )));
    }
    Ok(wasm_path)
}

/// Deploy `wasm` with `stellar contract deploy` and return the resulting
/// contract ID. Never reimplemented locally.
///
/// Thin system-touching wrapper; not unit-tested.
fn run_stellar_deploy(wasm: &Path, source: &str, network: &NetworkArgs) -> Result<String> {
    let wasm_str = path_str(wasm)?;

    let mut cmd = std::process::Command::new("stellar");
    cmd.args(["contract", "deploy", "--wasm", wasm_str, "--source", source]);
    cmd.args(network.cli_args());
    log::debug!("deploying {wasm_str}");

    let result = cmd.output();
    match result {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            extract_contract_id(&stdout).ok_or_else(|| {
                ForgeError::Other(format!(
                    "stellar contract deploy succeeded but no contract ID was found in its output:\n{stdout}"
                ))
            })
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Err(ForgeError::Other(format!(
                "stellar contract deploy failed:\n{stderr}"
            )))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(ForgeError::ToolMissing("stellar-cli".into()))
        }
        Err(e) => Err(ForgeError::io("running stellar contract deploy")(e)),
    }
}

/// Pull the contract ID out of `stellar contract deploy`'s stdout: the last
/// non-empty line that looks like a strkey contract ID (`C` + 55 base32
/// characters).
pub fn extract_contract_id(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter(|l| l.starts_with('C') && l.chars().count() == 56)
        .next_back()
        .map(str::to_string)
}

/// Build (if needed) and deploy the contract in `dir`, returning the new
/// contract ID.
pub fn deploy(
    dir: &Path,
    wasm_override: Option<&Path>,
    source: &str,
    network: &NetworkArgs,
) -> Result<String> {
    let wasm_path = build_if_needed(dir, wasm_override)?;
    run_stellar_deploy(&wasm_path, source, network)
}

/// The `deploy` subcommand.
pub struct DeployPlugin;

impl ForgePlugin for DeployPlugin {
    fn name(&self) -> &'static str {
        "deploy"
    }

    fn command(&self) -> Command {
        Command::new("deploy")
            .about("Build (if needed) and deploy the contract, printing its contract ID")
            .arg(
                Arg::new("path")
                    .long("path")
                    .help("Contract project directory [default: current directory]"),
            )
            .arg(
                Arg::new("wasm")
                    .long("wasm")
                    .help("Path to a pre-built .wasm to deploy [default: build then use target/wasm32v1-none/release/<crate>.wasm]"),
            )
            .arg(
                Arg::new("source")
                    .long("source")
                    .short('s')
                    .required(true)
                    .value_name("IDENTITY")
                    .help("Source account/identity that funds and signs the deployment"),
            )
            .arg(
                Arg::new("network")
                    .long("network")
                    .short('n')
                    .help("Configured network to deploy to [default: testnet]"),
            )
            .arg(
                Arg::new("rpc-url")
                    .long("rpc-url")
                    .help("RPC endpoint to use instead of a configured network"),
            )
            .arg(
                Arg::new("network-passphrase")
                    .long("network-passphrase")
                    .help("Network passphrase for --rpc-url"),
            )
    }

    fn run(&self, matches: &ArgMatches, ctx: &ForgeContext) -> Result<()> {
        if ctx.offline {
            return Err(ForgeError::InvalidArgument(
                "deploy is unavailable in offline mode because it submits a transaction".into(),
            ));
        }

        let dir = matches
            .get_one::<String>("path")
            .map(|p| ctx.cwd.join(p))
            .unwrap_or_else(|| ctx.cwd.clone());
        let wasm_override = matches.get_one::<String>("wasm").map(|p| ctx.cwd.join(p));
        let source = matches
            .get_one::<String>("source")
            .expect("source is required by clap");

        let network = NetworkArgs::resolve(
            matches.get_one::<String>("network").cloned(),
            matches.get_one::<String>("rpc-url").cloned(),
            matches.get_one::<String>("network-passphrase").cloned(),
        );

        let contract_id = deploy(&dir, wasm_override.as_deref(), source, &network)?;

        if ctx.json {
            let report = serde_json::json!({
                "contract_id": contract_id,
                "network": network.label(),
            });
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        } else if !ctx.quiet {
            println!("deployed to {}", network.label());
            println!("contract ID: {contract_id}");
        } else {
            println!("{contract_id}");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locates_wasm_by_crate_name() {
        assert_eq!(
            locate_wasm(Path::new("/proj"), "my_token"),
            PathBuf::from("/proj/target/wasm32v1-none/release/my_token.wasm")
        );
    }

    #[test]
    fn reads_and_normalizes_the_crate_name() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"my-token\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();

        assert_eq!(read_crate_name(tmp.path()).unwrap(), "my_token");
    }

    #[test]
    fn errors_outside_a_cargo_project() {
        let tmp = tempfile::tempdir().unwrap();
        let err = read_crate_name(tmp.path()).unwrap_err();
        assert!(err.to_string().contains("not a cargo project"), "{err}");
    }

    #[test]
    fn wasm_override_skips_crate_name_lookup() {
        let tmp = tempfile::tempdir().unwrap();
        let custom = tmp.path().join("custom.wasm");
        std::fs::write(&custom, b"\0asm").unwrap();

        // No Cargo.toml here at all — build_if_needed must not need one when
        // an explicit --wasm is given.
        assert_eq!(build_if_needed(tmp.path(), Some(&custom)).unwrap(), custom);
    }

    #[test]
    fn defaults_to_testnet() {
        let network = NetworkArgs::resolve(None, None, None);
        assert_eq!(network.label(), "testnet");
        assert_eq!(network.cli_args(), vec!["--network", "testnet"]);
    }

    #[test]
    fn an_explicit_network_is_passed_through() {
        let network = NetworkArgs::resolve(Some("mainnet".into()), None, None);
        assert_eq!(network.cli_args(), vec!["--network", "mainnet"]);
    }

    #[test]
    fn an_rpc_url_replaces_the_default_network() {
        let network = NetworkArgs::resolve(
            None,
            Some("http://localhost:8000/soroban/rpc".into()),
            Some("Standalone Network ; February 2017".into()),
        );
        assert_eq!(network.network, None);
        assert_eq!(
            network.cli_args(),
            vec![
                "--rpc-url",
                "http://localhost:8000/soroban/rpc",
                "--network-passphrase",
                "Standalone Network ; February 2017",
            ]
        );
    }

    #[test]
    fn extracts_the_last_contract_id_line() {
        let stdout = "ℹ️ deploying...\nsuccess\nCAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n";
        assert_eq!(
            extract_contract_id(stdout).as_deref(),
            Some("CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        );
    }

    #[test]
    fn no_contract_id_found_returns_none() {
        assert_eq!(extract_contract_id("deploy failed\n"), None);
    }

    #[test]
    fn plugin_name_matches_its_command() {
        let plugin = DeployPlugin;
        assert_eq!(plugin.name(), plugin.command().get_name());
    }

    #[test]
    fn help_documents_source_and_network() {
        let help = DeployPlugin.command().render_long_help().to_string();
        assert!(help.contains("--source"), "{help}");
        assert!(help.contains("--network"), "{help}");
        assert!(help.contains("IDENTITY"), "{help}");
    }
}
