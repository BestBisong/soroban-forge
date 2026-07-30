//! # soroban-forge-spec
//!
//! `soroban-forge spec` — dump the interface of a built contract: every
//! entrypoint with its argument and return types, plus the custom types
//! (structs, enums, errors) the interface refers to.
//!
//! The interface lives in the `contractspecv0` custom section of the built
//! wasm as XDR. Per soroban-forge's "wrap, don't reimplement" rule this
//! module does **not** decode that XDR itself — it shells out to the official
//! `stellar contract info interface` and only handles:
//!
//! - locating the built `.wasm` for a scaffolded project
//!   (`target/wasm32v1-none/release/<crate_name>.wasm`, the same
//!   `wasm32v1-none` layout `bindings ts`, `verify` and `doctor` expect), or
//!   using `--wasm <path>` when given
//! - asking for the representation the caller wants — the Rust-style listing
//!   for humans, raw JSON under the global `--json` flag
//! - surfacing a friendly error (pointing at `soroban-forge doctor`) when
//!   `stellar-cli` isn't on `PATH`
//!
//! Nothing here touches the network, so `spec` works under `--offline`.

use std::path::{Path, PathBuf};

use clap::{Arg, ArgMatches, Command};
use serde::Deserialize;
use soroban_forge_core::{ForgeContext, ForgeError, ForgePlugin, Result};

#[derive(Deserialize)]
struct Manifest {
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

/// Read `[package].name` from `dir/Cargo.toml` and return it as a crate name
/// (snake_case), which is what the build output is named after.
///
/// Deliberately duplicated rather than shared with `bindings ts` / `verify`:
/// modules depend only on `soroban-forge-core`, never on each other.
pub fn read_crate_name(dir: &Path) -> Result<String> {
    let manifest_path = dir.join("Cargo.toml");
    if !manifest_path.is_file() {
        return Err(ForgeError::InvalidArgument(format!(
            "{} is not a cargo project (no Cargo.toml) — pass --path or --wasm",
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

/// Resolve which wasm to read the spec from: `wasm_override` when given,
/// otherwise the release build of the cargo project in `dir`. Errors when the
/// file is not there, pointing at `stellar contract build`.
pub fn resolve_wasm(dir: &Path, wasm_override: Option<&Path>) -> Result<PathBuf> {
    let wasm_path = match wasm_override {
        Some(path) => path.to_path_buf(),
        None => {
            let crate_name = read_crate_name(dir)?;
            locate_wasm(dir, &crate_name)
        }
    };

    if !wasm_path.is_file() {
        return Err(ForgeError::InvalidArgument(format!(
            "no built wasm found at {} — run `stellar contract build` first (or pass --wasm)",
            wasm_path.display()
        )));
    }
    Ok(wasm_path)
}

/// Which representation of the interface to print.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecFormat {
    /// Rust-style listing: one `fn` per entrypoint plus the custom types.
    Rust,
    /// The same spec as JSON, for editors and scripts.
    Json,
}

impl SpecFormat {
    /// Value to pass to the CLI's `--output` flag.
    ///
    /// `json-formatted` rather than `json`: the multiline form parses
    /// identically and matches the pretty-printed JSON every other
    /// soroban-forge subcommand emits.
    pub fn cli_output(self) -> &'static str {
        match self {
            SpecFormat::Rust => "rust",
            SpecFormat::Json => "json-formatted",
        }
    }

    /// `--json` picks [`SpecFormat::Json`]; everything else is the human listing.
    pub fn from_json_flag(json: bool) -> Self {
        if json {
            SpecFormat::Json
        } else {
            SpecFormat::Rust
        }
    }
}

/// The `stellar` arguments used to read a wasm's interface.
///
/// Kept as a pure function so the command line we build is unit-tested
/// without a `stellar` binary present.
///
/// Flags follow `stellar contract info interface` as of stellar-cli 27.0.0
/// (`--wasm <PATH>`, `--output <rust|xdr-base64|json|json-formatted>`); we
/// depend on the official CLI's interface rather than decoding the spec XDR
/// ourselves.
pub fn spec_cli_args(wasm: &str, format: SpecFormat) -> Vec<String> {
    vec![
        "contract".to_string(),
        "info".to_string(),
        "interface".to_string(),
        "--wasm".to_string(),
        wasm.to_string(),
        "--output".to_string(),
        format.cli_output().to_string(),
    ]
}

/// Ask the official CLI for the interface of `wasm` and return its stdout.
///
/// Thin system-touching wrapper; not unit-tested.
fn run_stellar_info(wasm: &Path, format: SpecFormat) -> Result<String> {
    let wasm_str = wasm.to_str().ok_or_else(|| {
        ForgeError::Other(format!("wasm path {} is not valid UTF-8", wasm.display()))
    })?;

    log::debug!("reading contract interface from {}", wasm.display());
    let result = std::process::Command::new("stellar")
        .args(spec_cli_args(wasm_str, format))
        .output();

    match result {
        Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout).into_owned()),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Err(ForgeError::Other(format!(
                "stellar contract info interface failed — is {} a contract built with \
                 `stellar contract build`?\n{stderr}",
                wasm.display()
            )))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(ForgeError::ToolMissing("stellar-cli".into()))
        }
        Err(e) => Err(ForgeError::io("running stellar contract info interface")(e)),
    }
}

/// Locate the contract's wasm and return `(wasm_path, interface)` in the
/// requested representation.
pub fn dump_interface(
    contract_dir: &Path,
    wasm_override: Option<&Path>,
    format: SpecFormat,
) -> Result<(PathBuf, String)> {
    let wasm = resolve_wasm(contract_dir, wasm_override)?;
    let interface = run_stellar_info(&wasm, format)?;
    Ok((wasm, interface))
}

/// Header printed above the human listing (suppressed by `--quiet`).
pub fn format_header(wasm: &Path) -> String {
    format!("contract interface — {}\n\n", wasm.display())
}

/// The `spec` subcommand.
pub struct SpecPlugin;

impl ForgePlugin for SpecPlugin {
    fn name(&self) -> &'static str {
        "spec"
    }

    fn command(&self) -> Command {
        Command::new("spec")
            .about("Print the contract interface (entrypoints and types) from the built wasm")
            .long_about(
                "Dump the interface of a built contract: every entrypoint with its \
                 argument and return types, plus the structs, enums and error enums \
                 the interface refers to.\n\n\
                 Reads the spec out of the built wasm, so run `stellar contract build` \
                 first. Pass the global --json flag for machine-readable output.",
            )
            .arg(
                Arg::new("path")
                    .long("path")
                    .help("Contract project directory [default: current directory]"),
            )
            .arg(Arg::new("wasm").long("wasm").help(
                "Path to the built .wasm [default: target/wasm32v1-none/release/<crate>.wasm]",
            ))
    }

    fn run(&self, matches: &ArgMatches, ctx: &ForgeContext) -> Result<()> {
        let dir = matches
            .get_one::<String>("path")
            .map(|p| ctx.cwd.join(p))
            .unwrap_or_else(|| ctx.cwd.clone());
        let wasm_override = matches.get_one::<String>("wasm").map(|p| ctx.cwd.join(p));

        let format = SpecFormat::from_json_flag(ctx.json);
        let (wasm, interface) = dump_interface(&dir, wasm_override.as_deref(), format)?;

        if ctx.json {
            // The CLI already emits JSON; pass it through unchanged so the
            // spec stays byte-identical to what stellar-cli reports.
            print!("{interface}");
            if !interface.ends_with('\n') {
                println!();
            }
            return Ok(());
        }

        if !ctx.quiet {
            print!("{}", format_header(&wasm));
        }
        print!("{interface}");
        if !interface.ends_with('\n') {
            println!();
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
    fn reads_crate_name_and_normalizes_dashes() {
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
    fn missing_build_points_at_stellar_contract_build() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();

        let err = resolve_wasm(tmp.path(), None).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("stellar contract build"), "{msg}");
        assert!(msg.contains("demo.wasm"), "{msg}");
    }

    #[test]
    fn explicit_wasm_override_is_used_verbatim() {
        let tmp = tempfile::tempdir().unwrap();
        let wasm = tmp.path().join("custom.wasm");
        std::fs::write(&wasm, b"\0asm").unwrap();
        // No Cargo.toml in `dir` — the override must short-circuit the lookup.
        assert_eq!(resolve_wasm(tmp.path(), Some(&wasm)).unwrap(), wasm);
    }

    #[test]
    fn missing_wasm_override_is_reported() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("nope.wasm");
        let err = resolve_wasm(tmp.path(), Some(&missing)).unwrap_err();
        assert!(err.to_string().contains("nope.wasm"), "{err}");
    }

    #[test]
    fn human_format_asks_the_cli_for_the_rust_listing() {
        assert_eq!(
            spec_cli_args("/tmp/demo.wasm", SpecFormat::Rust),
            vec![
                "contract",
                "info",
                "interface",
                "--wasm",
                "/tmp/demo.wasm",
                "--output",
                "rust"
            ]
        );
    }

    #[test]
    fn json_format_asks_the_cli_for_json() {
        let args = spec_cli_args("/tmp/demo.wasm", SpecFormat::Json);
        assert_eq!(args.last().unwrap(), "json-formatted");
    }

    #[test]
    fn json_flag_selects_the_json_format() {
        assert_eq!(SpecFormat::from_json_flag(true), SpecFormat::Json);
        assert_eq!(SpecFormat::from_json_flag(false), SpecFormat::Rust);
    }

    #[test]
    fn header_names_the_wasm_that_was_read() {
        let header = format_header(Path::new("/proj/target/demo.wasm"));
        assert!(header.starts_with("contract interface — "));
        assert!(header.contains("/proj/target/demo.wasm"));
    }

    #[test]
    fn command_exposes_path_and_wasm_flags() {
        let matches = SpecPlugin
            .command()
            .try_get_matches_from(vec!["spec", "--path", "proj", "--wasm", "a.wasm"])
            .unwrap();
        assert_eq!(
            matches.get_one::<String>("path").map(String::as_str),
            Some("proj")
        );
        assert_eq!(
            matches.get_one::<String>("wasm").map(String::as_str),
            Some("a.wasm")
        );
    }

    #[test]
    fn command_name_matches_plugin_name() {
        assert_eq!(SpecPlugin.name(), SpecPlugin.command().get_name());
    }
}
