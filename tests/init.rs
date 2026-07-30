use std::process::Command;

#[test]
fn init_adds_config_to_an_existing_contract_non_destructively() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = "[package]\nname = \"existing-contract\"\nversion = \"0.1.0\"\n";
    std::fs::write(temp.path().join("Cargo.toml"), manifest).unwrap();
    std::fs::create_dir(temp.path().join("src")).unwrap();
    std::fs::write(temp.path().join("src/lib.rs"), "pub fn existing() {}\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args(["init", "--path", temp.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert!(temp.path().join("forge.toml").is_file());
    assert_eq!(
        std::fs::read_to_string(temp.path().join("Cargo.toml")).unwrap(),
        manifest
    );
    assert_eq!(
        std::fs::read_to_string(temp.path().join("src/lib.rs")).unwrap(),
        "pub fn existing() {}\n"
    );
}

#[test]
fn init_does_not_overwrite_existing_config() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"existing-contract\"\n",
    )
    .unwrap();
    std::fs::write(temp.path().join("forge.toml"), "keep = true\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args(["init", "--path", temp.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success(), "{output:?}");
    assert_eq!(
        std::fs::read_to_string(temp.path().join("forge.toml")).unwrap(),
        "keep = true\n"
    );
}
