use std::process::Command;

#[test]
fn log_file_receives_structured_logs_without_replacing_normal_output() {
    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("forge.jsonl");
    let output = Command::new(env!("CARGO_BIN_EXE_soroban-forge"))
        .args([
            "--log-file",
            log_path.to_str().unwrap(),
            "templates",
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert!(!output.stdout.is_empty(), "normal output should remain on stdout");

    let log = std::fs::read_to_string(log_path).unwrap();
    let entries: Vec<serde_json::Value> = log
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(!entries.is_empty(), "expected at least one log record");
    assert!(entries.iter().all(|entry| entry["timestamp_ms"].is_number()));
    assert!(entries.iter().all(|entry| entry["level"].is_string()));
    assert!(entries.iter().all(|entry| entry["target"].is_string()));
    assert!(entries.iter().all(|entry| entry["message"].is_string()));
}
