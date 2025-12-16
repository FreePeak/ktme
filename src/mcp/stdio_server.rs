use crate::error::Result;
use crate::mcp::tools::McpTools;
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};

pub struct StdioServer {
    #[allow(dead_code)] // Tools field will be used when MCP server is fully implemented
    tools: McpTools,
}

impl StdioServer {
    pub fn new() -> Self {
        Self {
            tools: McpTools,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut stdout = io::stdout();

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Parse and handle the JSON-RPC message
                    let request: Value = match serde_json::from_str(trimmed) {
                        Ok(r) => r,
                        Err(_) => {
                            // Send parse error response
                            let error_response = json!({
                                "jsonrpc": "2.0",
                                "id": null,
                                "error": {
                                    "code": -32700,
                                    "message": "Parse error"
                                }
                            });
                            let _ = stdout.write_all(error_response.to_string().as_bytes());
                            let _ = stdout.write_all(b"\n");
                            let _ = stdout.flush();
                            continue;
                        }
                    };

                    match self.handle_message(&request, &mut stdout) {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                        }
                        Err(_) => {
                            // Extract the request ID for proper error response
                            let request_id = request.get("id").cloned().unwrap_or(json!(null));
                            
                            // Send error response
                            let error_response = json!({
                                "jsonrpc": "2.0",
                                "id": request_id,
                                "error": {
                                    "code": -32603,
                                    "message": "Internal error"
                                }
                            });
                            let _ = stdout.write_all(error_response.to_string().as_bytes());
                            let _ = stdout.write_all(b"\n");
                            let _ = stdout.flush();
                        }
                    }
                }
                Err(_) => break,
            }
        }

        Ok(())
    }

    fn handle_message(&self, request: &Value, writer: &mut impl Write) -> Result<bool> {
        let method = request.get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");

        let id = request.get("id");

        // Check if this is a notification (no ID field)
        // According to JSON-RPC spec, we should never respond to notifications
        let is_notification = id.is_none();

        match method {
            "initialize" => {
                // Only send response if this is a request (has ID), not a notification
                if !is_notification {
                    let mut response = json!({
                        "jsonrpc": "2.0",
                        "result": {
                            "protocolVersion": "2024-11-05",
                            "capabilities": {
                                "tools": {
                                    "listChanged": false
                                },
                                "logging": {}
                            },
                            "serverInfo": {
                                "name": "ktme-mcp-server",
                                "version": "0.1.0"
                            }
                        }
                    });
                    // Only add ID field if this is not a notification
                    if let Some(request_id) = id {
                        response["id"] = request_id.clone();
                    }
                    self.send_response(&response, writer)?;
                }
            }
            "tools/list" => {
                // Only send response if this is a request (has ID), not a notification
                if !is_notification {
                    let tools = vec![
                        json!({
                            "name": "read_changes",
                            "description": "Read extracted code changes from Git",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "source": {
                                        "type": "string",
                                        "description": "Source identifier (commit hash, 'staged', or file path)"
                                    }
                                },
                                "required": ["source"]
                            }
                        }),
                        json!({
                            "name": "get_service_mapping",
                            "description": "Get documentation location for a service",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "service": {
                                        "type": "string",
                                        "description": "Service name"
                                    }
                                },
                                "required": ["service"]
                            }
                        }),
                        json!({
                            "name": "list_services",
                            "description": "List all mapped services",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        }),
                        json!({
                            "name": "generate_documentation",
                            "description": "Generate documentation from code changes",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "service": {
                                        "type": "string",
                                        "description": "Service name"
                                    },
                                    "changes": {
                                        "type": "string",
                                        "description": "JSON string of extracted changes"
                                    },
                                    "format": {
                                        "type": "string",
                                        "description": "Output format (markdown, json)",
                                        "enum": ["markdown", "json"]
                                    }
                                },
                                "required": ["service", "changes"]
                            }
                        }),
                        json!({
                            "name": "update_documentation",
                            "description": "Update existing documentation",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "service": {
                                        "type": "string",
                                        "description": "Service name"
                                    },
                                    "doc_path": {
                                        "type": "string",
                                        "description": "Path to documentation file"
                                    },
                                    "content": {
                                        "type": "string",
                                        "description": "Content to append/update"
                                    }
                                },
                                "required": ["service", "doc_path", "content"]
                            }
                        })
                    ];

                    // Build response without ID field initially
                    let mut response = json!({
                        "jsonrpc": "2.0",
                        "result": {
                            "tools": tools
                        }
                    });
                    // Only add ID field if this is not a notification
                    if let Some(request_id) = id {
                        response["id"] = request_id.clone();
                    }
                    self.send_response(&response, writer)?;
                }
            }
            "tools/call" => {
                // Only send response if this is a request (has ID), not a notification
                if !is_notification {
                    let empty_params = json!({});
                    let params = request.get("params").unwrap_or(&empty_params);
                    let tool_name = params.get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("");

                    let empty_args = json!({});
                    let arguments = params.get("arguments").unwrap_or(&empty_args);

                    let result = match tool_name {
                        "read_changes" => {
                            if let Some(source) = arguments.get("source").and_then(|s| s.as_str()) {
                                McpTools::read_changes(source).unwrap_or_else(|e| format!("Error: {}", e))
                            } else {
                                "Error: No source provided".to_string()
                            }
                        }
                        "get_service_mapping" => {
                            if let Some(service) = arguments.get("service").and_then(|s| s.as_str()) {
                                McpTools::get_service_mapping(service).unwrap_or_else(|e| format!("Error: {}", e))
                            } else {
                                "Error: No service provided".to_string()
                            }
                        }
                        "list_services" => {
                            McpTools::list_services()
                                .map(|s| format!("Services: {}", s.join(", ")))
                                .unwrap_or_else(|e| format!("Error: {}", e))
                        }
                        "generate_documentation" => {
                            let service = arguments.get("service").and_then(|s| s.as_str()).unwrap_or("");
                            let changes = arguments.get("changes").and_then(|c| c.as_str()).unwrap_or("");
                            let format = arguments.get("format").and_then(|f| f.as_str());

                            McpTools::generate_documentation(service, changes, format)
                                .unwrap_or_else(|e| format!("Error: {}", e))
                        }
                        "update_documentation" => {
                            let service = arguments.get("service").and_then(|s| s.as_str()).unwrap_or("");
                            let doc_path = arguments.get("doc_path").and_then(|d| d.as_str()).unwrap_or("");
                            let content = arguments.get("content").and_then(|c| c.as_str()).unwrap_or("");

                            McpTools::update_documentation(service, doc_path, content)
                                .unwrap_or_else(|e| format!("Error: {}", e))
                        }
                        _ => {
                            format!("Unknown tool: {}", tool_name)
                        }
                    };

                    // Build response without ID field initially
                    let mut response = json!({
                        "jsonrpc": "2.0",
                        "result": {
                            "content": [{
                                "type": "text",
                                "text": result
                            }]
                        }
                    });
                    // Only add ID field if this is not a notification
                    if let Some(request_id) = id {
                        response["id"] = request_id.clone();
                    }
                    self.send_response(&response, writer)?;
                }
            }
            _ => {
                // Only send response if this is a request (has ID), not a notification
                if !is_notification {
                    let mut response = json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32601,
                            "message": "Method not found"
                        }
                    });
                    // Only add ID field if this is not a notification
                    if let Some(request_id) = id {
                        response["id"] = request_id.clone();
                    }
                    self.send_response(&response, writer)?;
                }
            }
        }

        Ok(true)
    }

    fn send_response(&self, response: &Value, writer: &mut impl Write) -> Result<()> {
        let response_str = response.to_string();
        writer.write_all(response_str.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }
}