//! # soroban-forge-network
//!
//! `soroban-forge network add|list|use` — manage named network configs
//! (RPC URL + passphrase) and select a default for other commands.
//!
//! Networks are stored as a JSON file at
//! `~/.config/soroban-forge/networks.json`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command};
use serde::{Deserialize, Serialize};
use soroban_forge_core::{ForgeContext, ForgeError, ForgePlugin, Result};

/// A stored network configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Network {
    pub rpc_url: String,
    pub network_passphrase: String,
}

/// The on-disk network store.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NetworkStore {
    #[serde(default)]
    pub networks: BTreeMap<String, Network>,
    /// Name of the network selected as default, if any.
    #[serde(default)]
    pub default: Option<String>,
}

/// Well-known network presets, used to fill in `--rpc-url`/`--passphrase`
/// when adding a network under one of these names without passing them.
pub fn well_known(name: &str) -> Option<Network> {
    match name {
        "testnet" => Some(Network {
            rpc_url: "https://soroban-testnet.stellar.org".into(),
            network_passphrase: "Test SDF Network ; September 2015".into(),
        }),
        "futurenet" => Some(Network {
            rpc_url: "https://rpc-futurenet.stellar.org".into(),
            network_passphrase: "Test SDF Future Network ; October 2022".into(),
        }),
        "localnet" => Some(Network {
            rpc_url: "http://localhost:8000/soroban/rpc".into(),
            network_passphrase: "Standalone Network ; February 2017".into(),
        }),
        _ => None,
    }
}

/// Return the path to the network store file.
/// `~/.config/soroban-forge/networks.json`
pub fn store_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        ForgeError::Other("could not determine user config directory".into())
    })?;
    Ok(config_dir.join("soroban-forge").join("networks.json"))
}

/// Load the network store from disk, or return a default empty one.
pub fn load_store(path: &PathBuf) -> Result<NetworkStore> {
    if !path.is_file() {
        return Ok(NetworkStore::default());
    }
    let raw = std::fs::read_to_string(path)
        .map_err(ForgeError::io(format!("reading {}", path.display())))?;
    serde_json::from_str(&raw).map_err(|e| ForgeError::Config {
        path: path.clone(),
        message: e.to_string(),
    })
}

/// Save the network store to disk, creating parent directories as needed.
pub fn save_store(path: &PathBuf, store: &NetworkStore) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(ForgeError::io(format!("creating {}", parent.display())))?;
    }
    let json = serde_json::to_string_pretty(store)
        .map_err(|e| ForgeError::Other(format!("serializing network store: {e}")))?;
    std::fs::write(path, json)
        .map_err(ForgeError::io(format!("writing {}", path.display())))
}

/// Format the network list for display.
pub fn format_list(store: &NetworkStore) -> String {
    if store.networks.is_empty() {
        return "no networks configured. Use `soroban-forge network add <name> --rpc-url <url> --passphrase <passphrase>` to add one.\n".to_string();
    }
    let mut out = String::from("configured networks:\n\n");
    let name_width = store.networks.keys().map(|k| k.len()).max().unwrap_or(0);
    for (name, network) in &store.networks {
        let marker = if store.default.as_deref() == Some(name.as_str()) {
            "*"
        } else {
            " "
        };
        out.push_str(&format!(
            "{marker} {:<width$}  {}  ({})\n",
            name,
            network.rpc_url,
            network.network_passphrase,
            width = name_width
        ));
    }
    out
}

/// The `network` subcommand.
pub struct NetworkPlugin;

impl ForgePlugin for NetworkPlugin {
    fn name(&self) -> &'static str {
        "network"
    }

    fn command(&self) -> Command {
        Command::new("network")
            .about("Manage named network configs and select a default")
            .subcommand_required(true)
            .subcommand(
                Command::new("add")
                    .about("Add a named network config (testnet, futurenet, localnet, or custom)")
                    .arg(
                        Arg::new("name")
                            .help("Name for the network (e.g. testnet, futurenet, localnet)")
                            .required(true),
                    )
                    .arg(
                        Arg::new("rpc-url")
                            .long("rpc-url")
                            .help("RPC endpoint URL [default: built-in value for well-known names]"),
                    )
                    .arg(
                        Arg::new("passphrase")
                            .long("passphrase")
                            .help("Network passphrase [default: built-in value for well-known names]"),
                    )
                    .arg(
                        Arg::new("force")
                            .long("force")
                            .action(clap::ArgAction::SetTrue)
                            .help("Overwrite an existing network with the same name"),
                    ),
            )
            .subcommand(Command::new("list").about("List all configured networks"))
            .subcommand(
                Command::new("use")
                    .about("Select the default network for other commands")
                    .arg(
                        Arg::new("name")
                            .help("Name of the network to use as default")
                            .required(true),
                    ),
            )
    }

    fn run(&self, matches: &ArgMatches, ctx: &ForgeContext) -> Result<()> {
        let path = store_path()?;

        match matches.subcommand() {
            Some(("add", sub)) => {
                let name = sub.get_one::<String>("name").unwrap();
                let force = sub.get_flag("force");
                let mut store = load_store(&path)?;

                if store.networks.contains_key(name.as_str()) && !force {
                    return Err(ForgeError::AlreadyExists(PathBuf::from(name.as_str())));
                }

                let rpc_url = sub.get_one::<String>("rpc-url").cloned();
                let passphrase = sub.get_one::<String>("passphrase").cloned();
                let preset = well_known(name.as_str());

                let network = match (rpc_url, passphrase, preset) {
                    (Some(rpc_url), Some(network_passphrase), _) => Network {
                        rpc_url,
                        network_passphrase,
                    },
                    (rpc_url, passphrase, Some(preset)) => Network {
                        rpc_url: rpc_url.unwrap_or(preset.rpc_url),
                        network_passphrase: passphrase.unwrap_or(preset.network_passphrase),
                    },
                    _ => {
                        return Err(ForgeError::InvalidArgument(format!(
                            "`{name}` is not a well-known network — pass both --rpc-url and --passphrase"
                        )));
                    }
                };

                store.networks.insert(name.clone(), network.clone());
                if store.default.is_none() {
                    store.default = Some(name.clone());
                }
                save_store(&path, &store)?;

                if ctx.json {
                    let report = serde_json::json!({
                        "name": name,
                        "rpc_url": network.rpc_url,
                        "network_passphrase": network.network_passphrase,
                    });
                    println!("{}", serde_json::to_string_pretty(&report).unwrap());
                } else if !ctx.quiet {
                    println!("added network `{name}`");
                    println!("  rpc url:    {}", network.rpc_url);
                    println!("  passphrase: {}", network.network_passphrase);
                }
                Ok(())
            }

            Some(("list", _sub)) => {
                let store = load_store(&path)?;
                if ctx.json {
                    println!("{}", serde_json::to_string_pretty(&store).unwrap());
                } else if !ctx.quiet {
                    print!("{}", format_list(&store));
                }
                Ok(())
            }

            Some(("use", sub)) => {
                let name = sub.get_one::<String>("name").unwrap();
                let mut store = load_store(&path)?;

                if !store.networks.contains_key(name.as_str()) {
                    return Err(ForgeError::InvalidArgument(format!(
                        "network `{name}` not found (use `soroban-forge network list` to see configured networks)"
                    )));
                }

                store.default = Some(name.clone());
                save_store(&path, &store)?;

                if ctx.json {
                    let report = serde_json::json!({ "default": name });
                    println!("{}", serde_json::to_string_pretty(&report).unwrap());
                } else if !ctx.quiet {
                    println!("using `{name}` as the default network");
                }
                Ok(())
            }

            _ => Err(ForgeError::InvalidArgument(
                "unknown network subcommand".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("networks.json");

        let mut store = NetworkStore::default();
        store.networks.insert(
            "testnet".into(),
            Network {
                rpc_url: "https://soroban-testnet.stellar.org".into(),
                network_passphrase: "Test SDF Network ; September 2015".into(),
            },
        );
        store.default = Some("testnet".into());
        save_store(&path, &store).unwrap();

        let loaded = load_store(&path).unwrap();
        assert_eq!(loaded.networks.len(), 1);
        assert_eq!(loaded.default.as_deref(), Some("testnet"));
        assert_eq!(
            loaded.networks["testnet"].rpc_url,
            "https://soroban-testnet.stellar.org"
        );
    }

    #[test]
    fn load_missing_file_returns_empty_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.json");
        let store = load_store(&path).unwrap();
        assert!(store.networks.is_empty());
        assert!(store.default.is_none());
    }

    #[test]
    fn well_known_covers_testnet_futurenet_localnet() {
        assert!(well_known("testnet").is_some());
        assert!(well_known("futurenet").is_some());
        assert!(well_known("localnet").is_some());
        assert!(well_known("mainnet").is_none());
    }

    #[test]
    fn format_list_empty() {
        let store = NetworkStore::default();
        assert!(format_list(&store).contains("no networks configured"));
    }

    #[test]
    fn format_list_marks_the_default_network() {
        let mut store = NetworkStore::default();
        store.networks.insert(
            "testnet".into(),
            Network {
                rpc_url: "https://soroban-testnet.stellar.org".into(),
                network_passphrase: "Test SDF Network ; September 2015".into(),
            },
        );
        store.networks.insert(
            "localnet".into(),
            Network {
                rpc_url: "http://localhost:8000/soroban/rpc".into(),
                network_passphrase: "Standalone Network ; February 2017".into(),
            },
        );
        store.default = Some("testnet".into());

        let out = format_list(&store);
        assert!(out.contains("* testnet"), "{out}");
        assert!(out.contains("  localnet"), "{out}");
    }

    #[test]
    fn network_command_has_subcommands() {
        let plugin = NetworkPlugin;
        let cmd = plugin.command();
        let sub_names: Vec<&str> = cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(sub_names.contains(&"add"));
        assert!(sub_names.contains(&"list"));
        assert!(sub_names.contains(&"use"));
    }

    #[test]
    fn plugin_name_matches_its_command() {
        let plugin = NetworkPlugin;
        assert_eq!(plugin.name(), plugin.command().get_name());
    }
}
