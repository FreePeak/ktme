use crate::error::Result;
use crate::mcp::tools::McpTools;
use serde_json::{json, Value};

/// Shared MCP protocol handler for JSON-RPC 2.0 message processing
#[derive(Clone)]
pub struct McpProtocolHandler {
    server_name: String,
    server_version: String,
}

impl McpProtocolHandler {
    pub fn new(server_name: String, server_version: String) -> Self {
        Self {
            server_name,
            server_version,
        }
    }

    /// Handle incoming JSON-RPC message
    /// Returns Some(response) if a response should be sent, None for notifications
    pub async fn handle_message(&self, message: &str) -> Result<Option<Value>> {
        let request: Value = match serde_json::from_str(message) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Invalid JSON received: {}", e);
                // For invalid JSON, we can't determine if it was a request or notification
                // According to JSON-RPC spec, we should not send any response for parse errors
                return Ok(None);
            }
        };

        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = request.get("id");

        // Check if this is a notification (no ID field or ID is null)
        let is_notification = id.is_none() || (id.is_some() && id.unwrap().is_null());

        // Check if this is a valid JSON-RPC request
        if method.is_empty() {
            // Missing method field - never respond to notifications
            if is_notification {
                return Ok(None);
            }

            let error_response = json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request",
                    "data": "Missing 'method' field"
                }
            });
            return Ok(Some(error_response));
        }

        // Route to appropriate handler
        match method {
            "initialize" => self.handle_initialize(id, is_notification),
            "tools/list" => self.handle_tools_list(id, is_notification),
            "tools/call" => self.handle_tools_call(&request, id, is_notification).await,
            "ping" => self.handle_ping(id, is_notification),
            _ => self.handle_unknown_method(method, id, is_notification),
        }
    }

    fn handle_initialize(
        &self,
        id: Option<&Value>,
        is_notification: bool,
    ) -> Result<Option<Value>> {
        if is_notification {
            tracing::info!("Server initialized (notification)");
            return Ok(None);
        }

        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    },
                    "logging": {}
                },
                "serverInfo": {
                    "name": self.server_name,
                    "version": self.server_version
                }
            }
        });
        tracing::info!("Server initialized");
        Ok(Some(response))
    }

    fn handle_tools_list(
        &self,
        id: Option<&Value>,
        is_notification: bool,
    ) -> Result<Option<Value>> {
        if is_notification {
            return Ok(None);
        }

        let tools = Self::get_tools_list();
        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tools
            }
        });
        Ok(Some(response))
    }

    async fn handle_tools_call(
        &self,
        request: &Value,
        id: Option<&Value>,
        is_notification: bool,
    ) -> Result<Option<Value>> {
        if is_notification {
            return Ok(None);
        }

        let empty_params = json!({});
        let params = request.get("params").unwrap_or(&empty_params);
        let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");

        let empty_args = json!({});
        let arguments = params.get("arguments").unwrap_or(&empty_args);

        match Self::execute_tool(tool_name, arguments).await {
            Ok(result) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{
                            "type": "text",
                            "text": result
                        }]
                    }
                });
                Ok(Some(response))
            }
            Err(e) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": "Tool execution failed",
                        "data": e.to_string()
                    }
                });
                Ok(Some(response))
            }
        }
    }

    fn handle_ping(&self, id: Option<&Value>, is_notification: bool) -> Result<Option<Value>> {
        if is_notification {
            return Ok(None);
        }

        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {}
        });
        Ok(Some(response))
    }

    fn handle_unknown_method(
        &self,
        method: &str,
        id: Option<&Value>,
        is_notification: bool,
    ) -> Result<Option<Value>> {
        if is_notification {
            return Ok(None);
        }

        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": format!("Unknown method: {}", method)
            }
        });
        Ok(Some(response))
    }

    /// Get the list of available MCP tools
    pub fn get_tools_list() -> Vec<Value> {
        vec![
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
            }),
            json!({
                "name": "search_services",
                "description": "Search services by query with relevance scoring",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query string"
                        }
                    },
                    "required": ["query"]
                }
            }),
            json!({
                "name": "search_by_feature",
                "description": "Search services by specific feature or functionality",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "feature": {
                            "type": "string",
                            "description": "Feature to search for"
                        }
                    },
                    "required": ["feature"]
                }
            }),
            json!({
                "name": "search_by_keyword",
                "description": "Search services by keyword with flexible matching",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "keyword": {
                            "type": "string",
                            "description": "Keyword to search for"
                        }
                    },
                    "required": ["keyword"]
                }
            }),
            json!({
                "name": "automated_documentation_workflow",
                "description": "Automated workflow: extract changes → generate documentation → save to mapped location",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "service": {
                            "type": "string",
                            "description": "Service name"
                        },
                        "source": {
                            "type": "string",
                            "description": "Source identifier (commit hash, 'staged', or file path)"
                        }
                    },
                    "required": ["service", "source"]
                }
            }),
            json!({
                "name": "detect_service_name",
                "description": "Detect service name from current directory with AI fallback",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            json!({
                "name": "get_repository_info",
                "description": "Get information about the current Git repository and directory",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
        ]
    }

    /// Execute a tool by name with given arguments
    pub async fn execute_tool(tool_name: &str, arguments: &Value) -> Result<String> {
        match tool_name {
            "read_changes" => {
                if let Some(source) = arguments.get("source").and_then(|s| s.as_str()) {
                    McpTools::read_changes(source)
                } else {
                    Err(crate::error::KtmeError::InvalidInput(
                        "Missing 'source' parameter".to_string(),
                    ))
                }
            }
            "get_service_mapping" => {
                if let Some(service) = arguments.get("service").and_then(|s| s.as_str()) {
                    McpTools::get_service_mapping(service)
                } else {
                    Err(crate::error::KtmeError::InvalidInput(
                        "Missing 'service' parameter".to_string(),
                    ))
                }
            }
            "list_services" => McpTools::list_services()
                .map(|services| format!("Services: {}", services.join(", "))),
            "generate_documentation" => {
                let service = arguments
                    .get("service")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                let changes = arguments
                    .get("changes")
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                let format = arguments.get("format").and_then(|f| f.as_str());
                McpTools::generate_documentation(service, changes, format)
            }
            "update_documentation" => {
                let service = arguments
                    .get("service")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                let doc_path = arguments
                    .get("doc_path")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                let content = arguments
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                McpTools::update_documentation(service, doc_path, content)
            }
            "search_services" => {
                if let Some(query) = arguments.get("query").and_then(|q| q.as_str()) {
                    McpTools::search_services(query)
                } else {
                    Err(crate::error::KtmeError::InvalidInput(
                        "Missing 'query' parameter".to_string(),
                    ))
                }
            }
            "search_by_feature" => {
                if let Some(feature) = arguments.get("feature").and_then(|f| f.as_str()) {
                    McpTools::search_by_feature(feature)
                } else {
                    Err(crate::error::KtmeError::InvalidInput(
                        "Missing 'feature' parameter".to_string(),
                    ))
                }
            }
            "search_by_keyword" => {
                if let Some(keyword) = arguments.get("keyword").and_then(|k| k.as_str()) {
                    McpTools::search_by_keyword(keyword)
                } else {
                    Err(crate::error::KtmeError::InvalidInput(
                        "Missing 'keyword' parameter".to_string(),
                    ))
                }
            }
            "automated_documentation_workflow" => {
                let service = arguments
                    .get("service")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                let source = arguments
                    .get("source")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                McpTools::automated_documentation_workflow(service, source)
            }
            "detect_service_name" => McpTools::detect_service_name(),
            "get_repository_info" => McpTools::get_repository_info(),
            _ => Err(crate::error::KtmeError::InvalidInput(format!(
                "Unknown tool: {}",
                tool_name
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_initialize() {
        let handler = McpProtocolHandler::new("test-server".to_string(), "0.1.0".to_string());
        let message = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;

        let response = handler.handle_message(message).await.unwrap();
        assert!(response.is_some());

        let resp = response.unwrap();
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["serverInfo"]["name"], "test-server");
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let handler = McpProtocolHandler::new("test-server".to_string(), "0.1.0".to_string());
        let message = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;

        let response = handler.handle_message(message).await.unwrap();
        assert!(response.is_some());

        let resp = response.unwrap();
        assert_eq!(resp["jsonrpc"], "2.0");
        assert!(resp["result"]["tools"].is_array());
        assert!(resp["result"]["tools"].as_array().unwrap().len() > 0);
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let handler = McpProtocolHandler::new("test-server".to_string(), "0.1.0".to_string());
        let message = r#"{"jsonrpc":"2.0","id":3,"method":"ping"}"#;

        let response = handler.handle_message(message).await.unwrap();
        assert!(response.is_some());

        let resp = response.unwrap();
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 3);
    }

    #[tokio::test]
    async fn test_handle_notification() {
        let handler = McpProtocolHandler::new("test-server".to_string(), "0.1.0".to_string());
        // No "id" field = notification
        let message = r#"{"jsonrpc":"2.0","method":"initialize"}"#;

        let response = handler.handle_message(message).await.unwrap();
        assert!(response.is_none()); // Should not respond to notifications
    }

    #[tokio::test]
    async fn test_handle_invalid_json() {
        let handler = McpProtocolHandler::new("test-server".to_string(), "0.1.0".to_string());
        let message = r#"invalid json"#;

        let response = handler.handle_message(message).await.unwrap();
        assert!(response.is_none()); // Should not respond to parse errors
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let handler = McpProtocolHandler::new("test-server".to_string(), "0.1.0".to_string());
        let message = r#"{"jsonrpc":"2.0","id":4,"method":"unknown_method"}"#;

        let response = handler.handle_message(message).await.unwrap();
        assert!(response.is_some());

        let resp = response.unwrap();
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["error"]["code"], -32601);
    }
}
