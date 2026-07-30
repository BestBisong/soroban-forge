//! # soroban-forge-invoke
//!
//! `soroban-forge invoke <contract-id> <fn> [args...]` — calls a function on
//! a deployed contract with the official `stellar contract invoke` and
//! streams its result straight through.
//!
//! Per soroban-forge's "wrap, don't reimplement" rule this module never
//! parses or re-encodes function arguments itself: everything after `<fn>`
//! is forwarded verbatim to the CLI, which is what parses Soroban function
//! signatures and argument types.

use std::path::Path;

use clap::{Arg, ArgAction, ArgMatches, Command};
use soroban_forge_core::{ForgeContext, ForgeError, ForgePlugin, Result};

/// Network used when neither `--network` nor `--rpc-url` is given.
pub const DEFAULT_NETWORK: &str = "testnet";

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

/// Assemble the full `stellar contract invoke` argument list, for testing
/// without shelling out.
pub fn build_invoke_args(
    contract_id: &str,
    source: &str,
    network: &NetworkArgs,
    function: &str,
    fn_args: &[String],
) -> Vec<String> {
    let mut args = vec![
        "contract".to_string(),
        "invoke".to_string(),
        "--id".to_string(),
        contract_id.to_string(),
        "--source".to_string(),
        source.to_string(),
    ];
    args.extend(network.cli_args());
    args.push("--".to_string());
    args.push(function.to_string());
    args.extend(fn_args.iter().cloned());
    args
}

/// Invoke `function` on `contract_id`, inheriting stdio so the contract's
/// result (or the CLI's own diagnostics) is streamed straight to the user.
///
/// Thin system-touching wrapper; not unit-tested.
fn run_stellar_invoke(
    contract_id: &str,
    source: &str,
    network: &NetworkArgs,
    function: &str,
    fn_args: &[String],
    cwd: &Path,
) -> Result<()> {
    let args = build_invoke_args(contract_id, source, network, function, fn_args);
    log::debug!("running: stellar {}", args.join(" "));

    let status = std::process::Command::new("stellar")
        .args(&args)
        .current_dir(cwd)
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(ForgeError::Other(format!(
            "stellar contract invoke exited with status {}",
            s.code().map(|c| c.to_string()).unwrap_or_else(|| "signal".into())
        ))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(ForgeError::ToolMissing("stellar-cli".into()))
        }
        Err(e) => Err(ForgeError::io("running stellar contract invoke")(e)),
    }
}

/// The `invoke` subcommand.
pub struct InvokePlugin;

impl ForgePlugin for InvokePlugin {
    fn name(&self) -> &'static str {
        "invoke"
    }

    fn command(&self) -> Command {
        Command::new("invoke")
            .about("Call a function on a deployed contract and print the result")
            .long_about(
                "Call a function on a deployed contract with `stellar contract invoke`.\n\n\
                 Everything after <FN> is forwarded verbatim as that function's arguments, \
                 so --source/--network/etc. must be given before <CONTRACT_ID> and <FN>:\n\n  \
                 soroban-forge invoke --source alice <CONTRACT_ID> transfer --to G... --amount 100",
            )
            .trailing_var_arg(true)
            .arg(
                Arg::new("contract-id")
                    .required(true)
                    .value_name("CONTRACT_ID")
                    .help("Deployed contract ID (C…)"),
            )
            .arg(
                Arg::new("function")
                    .required(true)
                    .value_name("FN")
                    .help("Name of the contract function to call"),
            )
            .arg(
                Arg::new("args")
                    .value_name("ARGS")
                    .num_args(0..)
                    .allow_hyphen_values(true)
                    .action(ArgAction::Append)
                    .help("Arguments to the function, e.g. --to G... --amount 100"),
            )
            .arg(
                Arg::new("source")
                    .long("source")
                    .short('s')
                    .required(true)
                    .value_name("IDENTITY")
                    .help("Source account/identity that signs the invocation"),
            )
            .arg(
                Arg::new("network")
                    .long("network")
                    .short('n')
                    .help("Configured network the contract is deployed on [default: testnet]"),
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
                "invoke is unavailable in offline mode because it calls a deployed contract"
                    .into(),
            ));
        }

        let contract_id = matches
            .get_one::<String>("contract-id")
            .expect("contract-id is required by clap");
        let function = matches
            .get_one::<String>("function")
            .expect("function is required by clap");
        let fn_args: Vec<String> = matches
            .get_many::<String>("args")
            .unwrap_or_default()
            .cloned()
            .collect();
        let source = matches
            .get_one::<String>("source")
            .expect("source is required by clap");

        let network = NetworkArgs::resolve(
            matches.get_one::<String>("network").cloned(),
            matches.get_one::<String>("rpc-url").cloned(),
            matches.get_one::<String>("network-passphrase").cloned(),
        );

        run_stellar_invoke(contract_id, source, &network, function, &fn_args, &ctx.cwd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_testnet() {
        let network = NetworkArgs::resolve(None, None, None);
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
    fn builds_the_full_invoke_argument_list() {
        let network = NetworkArgs::resolve(None, None, None);
        let args = build_invoke_args(
            "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "alice",
            &network,
            "transfer",
            &["--to".to_string(), "GABC".to_string(), "--amount".to_string(), "100".to_string()],
        );
        assert_eq!(
            args,
            vec![
                "contract", "invoke", "--id",
                "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                "--source", "alice", "--network", "testnet", "--",
                "transfer", "--to", "GABC", "--amount", "100",
            ]
        );
    }

    #[test]
    fn plugin_name_matches_its_command() {
        let plugin = InvokePlugin;
        assert_eq!(plugin.name(), plugin.command().get_name());
    }

    #[test]
    fn help_documents_function_and_args() {
        let help = InvokePlugin.command().render_long_help().to_string();
        assert!(help.contains("FN"), "{help}");
        assert!(help.contains("--source"), "{help}");
        assert!(help.contains("ARGS"), "{help}");
    }

    #[test]
    fn parses_function_and_trailing_hyphenated_args() {
        // `--source` (and any other soroban-forge flag) must precede the
        // positionals: once `trailing_var_arg` starts consuming after
        // `<fn>`, everything remaining — flags included — is forwarded
        // verbatim to `stellar contract invoke` as function arguments.
        let matches = InvokePlugin
            .command()
            .try_get_matches_from([
                "invoke",
                "--source",
                "alice",
                "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                "transfer",
                "--to",
                "GABC",
                "--amount",
                "100",
            ])
            .unwrap();
        assert_eq!(matches.get_one::<String>("source").unwrap(), "alice");
        assert_eq!(
            matches.get_one::<String>("function").unwrap(),
            "transfer"
        );
        let args: Vec<&String> = matches.get_many::<String>("args").unwrap().collect();
        assert_eq!(args, vec!["--to", "GABC", "--amount", "100"]);
    }
}
