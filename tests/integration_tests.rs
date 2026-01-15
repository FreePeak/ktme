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
fn test_init_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&[
        "init",
        "--path",
        temp_dir.path().to_str().unwrap(),
        "--service",
        "test-init-service",
    ]);

    cmd.assert().success();

    // Check that docs directory was created
    let docs_dir = temp_dir.path().join("docs");
    assert!(docs_dir.exists());
    
    // Check that subdirectories were created
    assert!(docs_dir.join("api").exists());
    assert!(docs_dir.join("guides").exists());
    assert!(docs_dir.join("examples").exists());
    
    // Check that documentation files were created
    assert!(docs_dir.join("README.md").exists());
    assert!(docs_dir.join("architecture.md").exists());
    assert!(docs_dir.join("api.md").exists());
    assert!(docs_dir.join("changelog.md").exists());
    
    // Verify content of README
    let readme_content = fs::read_to_string(docs_dir.join("README.md"))?;
    assert!(readme_content.contains("test-init-service"));
    assert!(readme_content.contains("Documentation"));
    
    Ok(())
}

#[test]
fn test_init_command_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    
    // Run init once
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&[
        "init",
        "--path",
        temp_dir.path().to_str().unwrap(),
        "--service",
        "test-service",
    ]);
    cmd.assert().success();

    // Run init again - should warn about existing docs
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&[
        "init",
        "--path",
        temp_dir.path().to_str().unwrap(),
        "--service",
        "test-service",
    ]);
    
    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("already exists") || output.status.success());
    
    Ok(())
}

#[test]
fn test_init_with_force() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    
    // Run init once
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&[
        "init",
        "--path",
        temp_dir.path().to_str().unwrap(),
        "--service",
        "test-service",
    ]);
    cmd.assert().success();

    // Run init again with force
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&[
        "init",
        "--path",
        temp_dir.path().to_str().unwrap(),
        "--service",
        "test-service",
        "--force",
    ]);
    cmd.assert().success();
    
    Ok(())
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
        "--commit",
        "HEAD",
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
    cmd.args(&["generate", "--commit", "HEAD", "--service", "test-service"]);

    // Should now succeed with mock AI provider (auto-initialized)
    cmd.assert().success();
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
        "--input",
        diff_path.to_str().unwrap(),
        "--service",
        "test-service",
    ]);

    // Should now succeed with mock AI provider (auto-initialized)
    cmd.assert().success();

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
        "--commit",
        "HEAD",
        "--output",
        diff_path.to_str().unwrap(),
    ]);

    cmd.assert().success();
    assert!(diff_path.exists());

    // Then generate documentation (now succeeds with mock AI provider)
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("ktme").unwrap();
    cmd.args(&[
        "generate",
        "--input",
        diff_path.to_str().unwrap(),
        "--service",
        "test-service",
        "--type",
        "changelog",
    ]);

    cmd.assert().success(); // Now succeeds with auto-initialization and mock provider

    Ok(())
}
