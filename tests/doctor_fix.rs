//! Integration coverage for `doctor --fix`.
//!
//! These tests only exercise environment-independent behaviour — argument
//! wiring and the JSON-mode safety rule — so they never depend on a live
//! toolchain or mutate the host. The remedy-selection logic itself is
//! unit-tested in the `soroban-forge-doctor` crate.

use std::process::Command;

#[test]
fn doctor_help_advertises_fix_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args(["doctor", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--fix"), "{stdout}");
    assert!(stdout.contains("--yes"), "{stdout}");
}

#[test]
fn doctor_json_fix_without_yes_still_emits_json_array() {
    // In JSON mode without `--yes`, doctor must not prompt (which would
    // corrupt the stream) and must not run any remedy. stdout stays a valid
    // JSON array regardless of which checks pass on the runner.
    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args(["doctor", "--json", "--fix"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("doctor --json --fix must emit a JSON array on stdout");
    assert!(parsed.is_array(), "{stdout}");
}