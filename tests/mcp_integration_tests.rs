use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// Test MCP server in STDIO mode with initialize
#[test]
fn test_mcp_stdio_initialize() {
    let mut child = Command::new("cargo")
        .args(&[
            "run",
            "--release",
            "--bin",
            "ktme",
            "--",
            "mcp",
            "start",
            "--stdio",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start MCP server");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // Send initialize request
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    writeln!(stdin, "{}", init_request.to_string()).expect("Failed to write to stdin");
    stdin.flush().expect("Failed to flush stdin");

    // Read response
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");

    let response: Value = serde_json::from_str(&response_line).expect("Invalid JSON response");

    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(response["result"]["serverInfo"]["name"], "ktme-mcp-server");

    // Clean up
    child.kill().expect("Failed to kill child process");
}

/// Test MCP server in STDIO mode with tools/list
#[test]
fn test_mcp_stdio_tools_list() {
    let mut child = Command::new("cargo")
        .args(&[
            "run",
            "--release",
            "--bin",
            "ktme",
            "--",
            "mcp",
            "start",
            "--stdio",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start MCP server");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // Send tools/list request
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    });

    writeln!(stdin, "{}", tools_request.to_string()).expect("Failed to write to stdin");
    stdin.flush().expect("Failed to flush stdin");

    // Read response
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");

    let response: Value = serde_json::from_str(&response_line).expect("Invalid JSON response");

    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["tools"].is_array());

    let tools = response["result"]["tools"].as_array().unwrap();
    // Should have at least some tools (STDIO server might have fewer)
    assert!(
        tools.len() >= 5,
        "Should have at least 5 tools, got {}",
        tools.len()
    );

    // Verify some expected tools exist
    let tool_names: Vec<String> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();

    assert!(tool_names.contains(&"read_changes".to_string()));
    assert!(tool_names.contains(&"list_services".to_string()));

    // Clean up
    child.kill().expect("Failed to kill child process");
}

/// Test MCP server HTTP mode with status endpoint
#[test]
#[ignore] // Ignored by default to avoid port conflicts in CI
fn test_mcp_http_status() {
    // Start server in background
    let mut child = Command::new("cargo")
        .args(&[
            "run",
            "--release",
            "--bin",
            "ktme",
            "--",
            "mcp",
            "start",
            "--daemon",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start MCP server");

    // Wait for server to start
    thread::sleep(Duration::from_secs(3));

    // Test status endpoint
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("http://localhost:3000/status")
        .send()
        .expect("Failed to connect to server");

    assert!(response.status().is_success());

    let body: Value = response.json().expect("Failed to parse JSON");
    assert_eq!(body["status"], "running");
    assert_eq!(body["server_name"], "ktme-mcp-server");
    assert_eq!(body["tools_count"], 11);

    // Shutdown server
    let _ = client.post("http://localhost:3000/shutdown").send();
    thread::sleep(Duration::from_secs(1));

    // Clean up
    let _ = child.kill();
}

/// Test MCP server HTTP mode with JSON-RPC endpoint
#[test]
#[ignore] // Ignored by default to avoid port conflicts in CI
fn test_mcp_http_jsonrpc() {
    // Start server in background
    let mut child = Command::new("cargo")
        .args(&[
            "run",
            "--release",
            "--bin",
            "ktme",
            "--",
            "mcp",
            "start",
            "--daemon",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start MCP server");

    // Wait for server to start
    thread::sleep(Duration::from_secs(3));

    let client = reqwest::blocking::Client::new();

    // Test initialize
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let response = client
        .post("http://localhost:3000/mcp")
        .json(&init_request)
        .send()
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let body: Value = response.json().expect("Failed to parse JSON");
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], 1);
    assert_eq!(body["result"]["protocolVersion"], "2024-11-05");

    // Test tools/list
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });

    let response = client
        .post("http://localhost:3000/mcp")
        .json(&tools_request)
        .send()
        .expect("Failed to send request");

    let body: Value = response.json().expect("Failed to parse JSON");
    assert_eq!(body["result"]["tools"].as_array().unwrap().len(), 11);

    // Shutdown server
    let _ = client.post("http://localhost:3000/shutdown").send();
    thread::sleep(Duration::from_secs(1));

    // Clean up
    let _ = child.kill();
}
