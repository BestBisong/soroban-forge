//! `soroban-forge new`: overwriting an existing target directory (`--force`)
//! and supplying template variables (`--var`).
//!
//! These run the real binary with stdin closed, i.e. the non-interactive path.
//! The confirmation prompt `--force` shows a human, and the prompting of a
//! missing template variable, cannot be exercised here — a test harness is
//! never a TTY — so those are covered by unit tests in `crates/scaffold`
//! against a scripted prompter instead. What matters at this level is that the
//! non-interactive contract holds: no hang, no prompt, clear exit codes.

use std::path::Path;
use std::process::Command;

fn forge(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args(args)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("failed to run soroban-forge")
}

fn new_project(dir: &Path, name: &str, extra: &[&str]) -> std::process::Output {
    let mut args = vec![
        "new",
        name,
        "--template",
        "hello-world",
        "--no-git",
        "--output-dir",
        dir.to_str().unwrap(),
    ];
    args.extend_from_slice(extra);
    forge(&args)
}

#[test]
fn an_existing_directory_aborts_without_force() {
    let temp = tempfile::tempdir().unwrap();
    let first = new_project(temp.path(), "demo", &[]);
    assert_eq!(first.status.code(), Some(0), "{first:?}");

    let second = new_project(temp.path(), "demo", &[]);
    assert_eq!(second.status.code(), Some(1), "{second:?}");

    let stderr = String::from_utf8(second.stderr).unwrap();
    assert!(stderr.contains("already exists"), "{stderr}");
    assert!(
        stderr.contains("--force"),
        "the error must name the way out: {stderr}"
    );
}

#[test]
fn force_overwrites_an_existing_directory() {
    let temp = tempfile::tempdir().unwrap();
    assert_eq!(new_project(temp.path(), "demo", &[]).status.code(), Some(0));

    // Clobber a generated file so we can prove it gets rewritten.
    let lib = temp.path().join("demo/src/lib.rs");
    std::fs::write(&lib, "// scribbled over\n").unwrap();

    let forced = new_project(temp.path(), "demo", &["--force"]);
    assert_eq!(forced.status.code(), Some(0), "{forced:?}");

    let contents = std::fs::read_to_string(&lib).unwrap();
    assert!(
        contents.contains("#[contract]"),
        "--force should have restored the template file, got: {contents}"
    );
}

/// `--force` must not stop to ask when nobody can answer, and must not hang.
#[test]
fn force_does_not_prompt_when_not_interactive() {
    let temp = tempfile::tempdir().unwrap();
    new_project(temp.path(), "demo", &[]);

    let forced = new_project(temp.path(), "demo", &["--force"]);
    let stdout = String::from_utf8(forced.stdout).unwrap();
    assert_eq!(forced.status.code(), Some(0), "{stdout}");
    assert!(
        !stdout.contains("[y/N]"),
        "a non-interactive run must not prompt: {stdout}"
    );
}

#[test]
fn var_rejects_a_value_without_an_equals_sign() {
    let temp = tempfile::tempdir().unwrap();
    let output = new_project(temp.path(), "demo", &["--var", "symbol"]);

    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("NAME=VALUE"), "{stderr}");
    assert!(
        !temp.path().join("demo").exists(),
        "nothing should be written when the arguments are rejected"
    );
}

#[test]
fn var_refuses_to_override_a_derived_variable() {
    let temp = tempfile::tempdir().unwrap();
    let output = new_project(temp.path(), "demo", &["--var", "crate_name=sneaky"]);

    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("crate_name"), "{stderr}");
    assert!(stderr.contains("cannot be set with --var"), "{stderr}");
}

/// A value for a placeholder the template does not declare is harmless: it is
/// simply available to the renderer.
#[test]
fn extra_vars_do_not_break_generation() {
    let temp = tempfile::tempdir().unwrap();
    let output = new_project(temp.path(), "demo", &["--var", "unused=whatever"]);

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(temp.path().join("demo/src/lib.rs").is_file());
}
