use std::process::Command;

#[test]
fn quiet_new_is_silent_and_still_creates_project() {
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args([
            "--quiet",
            "new",
            "silent-demo",
            "--template",
            "hello-world",
            "--author",
            "Test Author",
            "--output-dir",
            temp.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(temp.path().join("silent-demo/Cargo.toml").is_file());
    assert!(temp.path().join("silent-demo/forge.toml").is_file());
}

#[test]
fn quiet_mode_keeps_errors_visible() {
    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args(["--quiet", "new", "INVALID-NAME"])
        .output()
        .unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error:"), "{stderr}");
    assert!(stderr.contains("not a valid project name"), "{stderr}");
}

/// Generate a project with `--quiet new` into `parent`, returning its path.
fn generate_quiet_project(parent: &std::path::Path, name: &str) -> std::path::PathBuf {
    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args([
            "--quiet",
            "new",
            name,
            "--template",
            "hello-world",
            "--author",
            "Test Author",
            "--output-dir",
            parent.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    parent.join(name)
}

#[test]
fn quiet_templates_is_silent() {
    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args(["--quiet", "templates"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
}

#[test]
fn quiet_config_is_silent_without_config_file() {
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args(["--quiet", "config"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
}

#[test]
fn quiet_config_keeps_unknown_key_warnings_on_stderr() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("forge.toml"),
        "[project]\nnmae = \"typo\"\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args(["--quiet", "config"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    // Informational report suppressed...
    assert!(output.stdout.is_empty(), "{output:?}");
    // ...but diagnostics still reach stderr, like errors do.
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unknown key"), "{stderr}");
}

#[test]
fn quiet_test_init_is_silent() {
    let temp = tempfile::tempdir().unwrap();
    let project = generate_quiet_project(temp.path(), "quiet-ti");
    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args(["--quiet", "test-init"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(project.join("tests/forge_smoke.rs").is_file());
}

#[test]
fn quiet_ci_init_is_silent() {
    let temp = tempfile::tempdir().unwrap();
    let project = generate_quiet_project(temp.path(), "quiet-ci");
    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args(["--quiet", "ci-init"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
}
