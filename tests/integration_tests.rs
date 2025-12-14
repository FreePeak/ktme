use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.arg("--help");

    cmd.assert().success();
}

#[test]
fn test_extract_command() {
    #[allow(deprecated)]
let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&["extract", "--commit", "HEAD"]);

    // Should succeed in a git repository
    let assert = cmd.assert();
    assert.success();
}

#[test]
fn test_extract_output_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let output_path = temp_dir.path().join("test_diff.json");

    #[allow(deprecated)]
let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&[
        "extract",
        "--commit", "HEAD",
        "--output",
        output_path.to_str().unwrap(),
    ]);

    cmd.assert().success();

    // Check that file was created
    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path)?;
    assert!(content.contains("source"));
    assert!(content.contains("identifier"));

    Ok(())
}

#[test]
fn test_generate_command_without_ai_key() {
    #[allow(deprecated)]
let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&[
        "generate",
        "--commit", "HEAD",
        "--service", "test-service",
    ]);

    // Should fail without AI API key
    cmd.assert().failure();
}

#[test]
fn test_generate_command_with_input_file() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test diff file
    let temp_dir = TempDir::new()?;
    let diff_path = temp_dir.path().join("test_diff.json");

    let test_diff = r#"{
        "source": "test",
        "identifier": "test-commit",
        "timestamp": "2025-12-06T00:00:00Z",
        "author": "test@example.com",
        "message": "Test commit",
        "files": [],
        "summary": {
            "total_files": 0,
            "total_additions": 0,
            "total_deletions": 0
        }
    }"#;

    fs::write(&diff_path, test_diff)?;

    #[allow(deprecated)]
let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&[
        "generate",
        "--input", diff_path.to_str().unwrap(),
        "--service", "test-service",
    ]);

    // Should still fail due to no AI key, but after processing the input
    cmd.assert().failure();

    Ok(())
}

#[test]
fn test_extract_and_generate_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let diff_path = temp_dir.path().join("diff.json");

    // First extract changes
    #[allow(deprecated)]
let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&[
        "extract",
        "--commit", "HEAD",
        "--output",
        diff_path.to_str().unwrap(),
    ]);

    cmd.assert().success();
    assert!(diff_path.exists());

    // Then try to generate (will fail without AI key, but pipeline works)
    #[allow(deprecated)]
let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&[
        "generate",
        "--input", diff_path.to_str().unwrap(),
        "--service", "test-service",
        "--doc-type", "changelog",
    ]);

    cmd.assert().failure(); // Expected without AI key

    Ok(())
}