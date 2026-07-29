//! # soroban-forge-identity
//!
//! `soroban-forge identity generate|list|fund` — manage test keypairs and
//! fund them via friendbot on testnet.
//!
//! Identities are stored as a JSON file at
//! `~/.config/soroban-forge/identities.json`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command};
use serde::{Deserialize, Serialize};
use soroban_forge_core::{ForgeContext, ForgeError, ForgePlugin, Result};

/// A stored test identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub public_key: String,
    pub secret_key: String,
}

/// The on-disk identity store.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IdentityStore {
    #[serde(default)]
    pub identities: BTreeMap<String, Identity>,
}

/// Return the path to the identity store file.
/// `~/.config/soroban-forge/identities.json`
pub fn store_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        ForgeError::Other("could not determine user config directory".into())
    })?;
    Ok(config_dir.join("soroban-forge").join("identities.json"))
}

/// Load the identity store from disk, or return a default empty one.
pub fn load_store(path: &PathBuf) -> Result<IdentityStore> {
    if !path.is_file() {
        return Ok(IdentityStore::default());
    }
    let raw = std::fs::read_to_string(path)
        .map_err(ForgeError::io(format!("reading {}", path.display())))?;
    serde_json::from_str(&raw).map_err(|e| ForgeError::Config {
        path: path.clone(),
        message: e.to_string(),
    })
}

/// Save the identity store to disk, creating parent directories as needed.
pub fn save_store(path: &PathBuf, store: &IdentityStore) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(ForgeError::io(format!("creating {}", parent.display())))?;
    }
    let json = serde_json::to_string_pretty(store)
        .map_err(|e| ForgeError::Other(format!("serializing identity store: {e}")))?;
    std::fs::write(path, json)
        .map_err(ForgeError::io(format!("writing {}", path.display())))
}

/// Generate a new Stellar keypair and return `(public_key, secret_key)` as
/// Stellar-encoded strings (`G...` / `S...`).
pub fn generate_keypair() -> (String, String) {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use stellar_strkey::ed25519::{PrivateKey, PublicKey};

    let signing_key = SigningKey::generate(&mut OsRng);
    let seed = signing_key.to_bytes();
    let pubkey = signing_key.verifying_key().to_bytes();

    let public = PublicKey(pubkey).to_string();
    let secret = PrivateKey(seed).to_string();
    (public, secret)
}

/// Fund a Stellar testnet account via friendbot.
/// Returns the friendbot response body on success.
pub fn fund_friendbot(public_key: &str) -> Result<String> {
    let url = format!("https://friendbot.stellar.org/?addr={public_key}");
    log::debug!("requesting friendbot: {url}");
    let response = ureq::get(&url)
        .call()
        .map_err(|e| ForgeError::Other(format!("friendbot request failed: {e}")))?;
    let body = response
        .into_string()
        .map_err(|e| ForgeError::Other(format!("reading friendbot response: {e}")))?;
    Ok(body)
}

/// Format the identity list for display.
pub fn format_list(store: &IdentityStore) -> String {
    if store.identities.is_empty() {
        return "no identities stored. Use `soroban-forge identity generate <name>` to create one.\n".to_string();
    }
    let mut out = String::from("stored identities:\n\n");
    let name_width = store.identities.keys().map(|k| k.len()).max().unwrap_or(0);
    for (name, id) in &store.identities {
        out.push_str(&format!("  {:<width$}  {}\n", name, id.public_key, width = name_width));
    }
    out
}

/// The `identity` subcommand.
pub struct IdentityPlugin;

impl ForgePlugin for IdentityPlugin {
    fn name(&self) -> &'static str {
        "identity"
    }

    fn command(&self) -> Command {
        Command::new("identity")
            .about("Manage test keypairs and fund them via friendbot on testnet")
            .subcommand_required(true)
            .subcommand(
                Command::new("generate")
                    .about("Generate a new Stellar test keypair")
                    .arg(
                        Arg::new("name")
                            .help("Name for the identity (e.g. alice, deployer)")
                            .required(true),
                    )
                    .arg(
                        Arg::new("force")
                            .long("force")
                            .action(clap::ArgAction::SetTrue)
                            .help("Overwrite an existing identity with the same name"),
                    ),
            )
            .subcommand(
                Command::new("list")
                    .about("List all stored identities"),
            )
            .subcommand(
                Command::new("fund")
                    .about("Fund a stored identity via Stellar testnet friendbot")
                    .arg(
                        Arg::new("name")
                            .help("Name of the identity to fund")
                            .required(true),
                    ),
            )
    }

    fn run(&self, matches: &ArgMatches, ctx: &ForgeContext) -> Result<()> {
        let path = store_path()?;

        match matches.subcommand() {
            Some(("generate", sub)) => {
                let name = sub.get_one::<String>("name").unwrap();
                let force = sub.get_flag("force");
                let mut store = load_store(&path)?;

                if store.identities.contains_key(name.as_str()) && !force {
                    return Err(ForgeError::AlreadyExists(PathBuf::from(name.as_str())));
                }

                let (public_key, secret_key) = generate_keypair();
                store.identities.insert(
                    name.clone(),
                    Identity {
                        public_key: public_key.clone(),
                        secret_key: secret_key.clone(),
                    },
                );
                save_store(&path, &store)?;

                if ctx.json {
                    let report = serde_json::json!({
                        "name": name,
                        "public_key": public_key,
                        "secret_key": secret_key,
                    });
                    println!("{}", serde_json::to_string_pretty(&report).unwrap());
                } else if !ctx.quiet {
                    println!("generated identity `{name}`");
                    println!("  public key: {public_key}");
                    println!("  secret key: {secret_key}");
                    println!();
                    println!("fund on testnet: soroban-forge identity fund {name}");
                }
                Ok(())
            }

            Some(("list", _sub)) => {
                let store = load_store(&path)?;
                if ctx.json {
                    println!("{}", serde_json::to_string_pretty(&store.identities).unwrap());
                } else if !ctx.quiet {
                    print!("{}", format_list(&store));
                }
                Ok(())
            }

            Some(("fund", sub)) => {
                let name = sub.get_one::<String>("name").unwrap();
                let store = load_store(&path)?;

                let id = store.identities.get(name.as_str()).ok_or_else(|| {
                    ForgeError::InvalidArgument(format!(
                        "identity `{name}` not found (use `soroban-forge identity list` to see available identities)"
                    ))
                })?;

                if !ctx.quiet {
                    println!("funding `{name}` ({}) via testnet friendbot...", id.public_key);
                }

                fund_friendbot(&id.public_key)?;

                if ctx.json {
                    let report = serde_json::json!({
                        "name": name,
                        "public_key": id.public_key,
                        "funded": true,
                        "network": "testnet",
                    });
                    println!("{}", serde_json::to_string_pretty(&report).unwrap());
                } else if !ctx.quiet {
                    println!("funded `{name}` on testnet.");
                }
                Ok(())
            }

            _ => Err(ForgeError::InvalidArgument(
                "unknown identity subcommand".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_keypair_produces_valid_stellar_keys() {
        let (public, secret) = generate_keypair();
        assert!(public.starts_with('G'), "public key should start with G: {public}");
        assert!(secret.starts_with('S'), "secret key should start with S: {secret}");
        assert_eq!(public.len(), 56, "Stellar public keys are 56 chars");
        assert_eq!(secret.len(), 56, "Stellar secret keys are 56 chars");
    }

    #[test]
    fn generate_keypair_is_unique() {
        let (pub1, _) = generate_keypair();
        let (pub2, _) = generate_keypair();
        assert_ne!(pub1, pub2, "two generated keypairs should differ");
    }

    #[test]
    fn store_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("identities.json");

        let mut store = IdentityStore::default();
        store.identities.insert(
            "alice".into(),
            Identity {
                public_key: "GABC".into(),
                secret_key: "SABC".into(),
            },
        );
        save_store(&path, &store).unwrap();

        let loaded = load_store(&path).unwrap();
        assert_eq!(loaded.identities.len(), 1);
        assert_eq!(loaded.identities["alice"].public_key, "GABC");
    }

    #[test]
    fn load_missing_file_returns_empty_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.json");
        let store = load_store(&path).unwrap();
        assert!(store.identities.is_empty());
    }

    #[test]
    fn format_list_empty() {
        let store = IdentityStore::default();
        assert!(format_list(&store).contains("no identities stored"));
    }

    #[test]
    fn format_list_shows_names_and_public_keys() {
        let mut store = IdentityStore::default();
        store.identities.insert(
            "alice".into(),
            Identity {
                public_key: "GAAA".into(),
                secret_key: "SAAA".into(),
            },
        );
        store.identities.insert(
            "bob".into(),
            Identity {
                public_key: "GBBB".into(),
                secret_key: "SBBB".into(),
            },
        );
        let output = format_list(&store);
        assert!(output.contains("alice"));
        assert!(output.contains("GAAA"));
        assert!(output.contains("bob"));
        assert!(output.contains("GBBB"));
        // Secret keys must NOT appear in the list
        assert!(!output.contains("SAAA"));
        assert!(!output.contains("SBBB"));
    }

    #[test]
    fn identity_command_has_subcommands() {
        let plugin = IdentityPlugin;
        let cmd = plugin.command();
        let sub_names: Vec<&str> = cmd
            .get_subcommands()
            .map(|s| s.get_name())
            .collect();
        assert!(sub_names.contains(&"generate"));
        assert!(sub_names.contains(&"list"));
        assert!(sub_names.contains(&"fund"));
    }
}
