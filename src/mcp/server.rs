use crate::error::Result;
use crate::mcp::tools::McpTools;
use crate::ai::AIClient;
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub server_name: String,
    pub transport: String,
    pub port: Option<u16>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server_name: "ktme-mcp-server".to_string(),
            transport: "stdio".to_string(),
            port: None,
        }
    }
}

pub struct McpServer {
    config: ServerConfig,
    #[allow(dead_code)] // Tools field will be used when MCP server is fully implemented
    tools: McpTools,
    ai_client: Option<AIClient>,
}

impl McpServer {
    pub fn new(config: ServerConfig) -> Result<Self> {
        let tools = McpTools;
        let mut server = Self {
            config,
            tools,
            ai_client: None,
        };

        // Try to initialize AI client
        match AIClient::new() {
            Ok(client) => {
                // Only log if not in STDIO mode
                if server.config.transport != "stdio" {
                    tracing::info!("AI client initialized: {}", client.provider_name());
                }
                server.ai_client = Some(client);
            }
            Err(e) => {
                if server.config.transport != "stdio" {
                    tracing::warn!("AI client initialization failed: {}", e);
                }
            }
        }

        Ok(server)
    }

    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting MCP server: {} (transport: {})",
                   self.config.server_name, self.config.transport);

        match self.config.transport.as_str() {
            "stdio" => self.run_stdio_server().await,
            "sse" | "http" => {
                if let Some(port) = self.config.port {
                    self.run_sse_server(port).await
                } else {
                    return Err(crate::error::KtmeError::Config(
                        "HTTP/SSE transport requires port configuration".to_string()
                    ));
                }
            }
            _ => {
                return Err(crate::error::KtmeError::Config(
                    format!("Unsupported transport: {}", self.config.transport)
                ));
            }
        }
    }

    async fn run_stdio_server(&self) -> Result<()> {
        tracing::info!("Starting STDIO MCP server");

        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut stdout = io::stdout();

        // Don't send init response immediately - wait for initialize request

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    tracing::debug!("Received: {}", trimmed);

                    match self.handle_message(trimmed, &mut stdout).await {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error handling message: {}", e);
                            // Try to parse the request to get the ID for error response
                            if let Ok(parsed_request) = serde_json::from_str::<Value>(trimmed) {
                                // Only send error response if ID is not null (not a notification)
                                if let Some(request_id) = parsed_request.get("id") {
                                    if !request_id.is_null() {
                                        let error_response = json!({
                                            "jsonrpc": "2.0",
                                            "id": request_id,
                                            "error": {
                                                "code": -32603,
                                                "message": "Internal error",
                                                "data": e.to_string()
                                            }
                                        });
                                        self.send_response(&error_response, &mut stdout)?;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn run_sse_server(&self, port: u16) -> Result<()> {
        use tokio::net::TcpListener;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        tracing::info!("SSE server listening on port {}", port);

        loop {
            let (socket, _) = listener.accept().await?;
            let (mut reader, mut writer) = socket.into_split();

            // Handle SSE connection
            tokio::spawn(async move {
                // Send SSE headers
                let _ = writer.write_all(b"HTTP/1.1 200 OK\r\n").await;
                let _ = writer.write_all(b"Content-Type: text/event-stream\r\n").await;
                let _ = writer.write_all(b"Cache-Control: no-cache\r\n").await;
                let _ = writer.write_all(b"Connection: close\r\n").await;
                let _ = writer.write_all(b"\r\n").await;

                let mut buffer = [0; 1024];
                loop {
                    match reader.read(&mut buffer).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let _request = String::from_utf8_lossy(&buffer[..n]);
                            // Handle MCP request
                            // TODO: Implement proper MCP protocol handling for SSE
                        }
                        Err(_) => break,
                    }
                }
            });
        }
    }

    async fn handle_message(&self, message: &str, writer: &mut impl Write) -> Result<bool> {
        let request: Value = match serde_json::from_str(message) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Invalid JSON received: {}", e);
                // For invalid JSON, we can't determine if it was a request or notification
                // According to JSON-RPC spec, we should not send any response for parse errors
                // as we can't determine if it was a notification
                return Ok(true);
            }
        };

        let method = request.get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");

        let id = request.get("id");

        // Check if this is a notification (no ID field or ID is null)
        // For MCP, we should never respond to notifications
        let is_notification = id.is_none() || (id.is_some() && id.unwrap().is_null());

        // Check if this is a valid JSON-RPC request
        if method.is_empty() {
            // Missing method field - never respond to notifications
            if is_notification {
                return Ok(true);
            }

            let mut error_response = json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32600,
                    "message": "Invalid Request",
                    "data": "Missing 'method' field"
                }
            });
            // Only add ID if it exists (not a notification)
            if let Some(request_id) = id {
                error_response["id"] = request_id.clone();
            }
            self.send_response(&error_response, writer)?;
            return Ok(true);
        }

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
                                "name": self.config.server_name,
                                "version": env!("CARGO_PKG_VERSION")
                            }
                        }
                    });
                    // Only add ID field if this is not a notification
                    if let Some(request_id) = id {
                        response["id"] = request_id.clone();
                    }
                    self.send_response(&response, writer)?;
                }
                tracing::info!("Server initialized");
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

                    match tool_name {
                    "read_changes" => {
                        if let Some(source) = arguments.get("source").and_then(|s| s.as_str()) {
                            match McpTools::read_changes(source) {
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
                                    self.send_response(&response, writer)?;
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
                                    self.send_response(&response, writer)?;
                                }
                            }
                        }
                    }
                    "get_service_mapping" => {
                        if let Some(service) = arguments.get("service").and_then(|s| s.as_str()) {
                            match McpTools::get_service_mapping(service) {
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
                                    self.send_response(&response, writer)?;
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
                                    self.send_response(&response, writer)?;
                                }
                            }
                        }
                    }
                    "list_services" => {
                        match McpTools::list_services() {
                            Ok(services) => {
                                let response = json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "content": [{
                                            "type": "text",
                                            "text": format!("Services: {}", services.join(", "))
                                        }]
                                    }
                                });
                                self.send_response(&response, writer)?;
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
                                self.send_response(&response, writer)?;
                            }
                        }
                    }
                    "generate_documentation" => {
                        let service = arguments.get("service").and_then(|s| s.as_str()).unwrap_or("");
                        let changes = arguments.get("changes").and_then(|c| c.as_str()).unwrap_or("");
                        let format = arguments.get("format").and_then(|f| f.as_str());

                        match McpTools::generate_documentation(service, changes, format) {
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
                                self.send_response(&response, writer)?;
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
                                self.send_response(&response, writer)?;
                            }
                        }
                    }
                    "update_documentation" => {
                        let service = arguments.get("service").and_then(|s| s.as_str()).unwrap_or("");
                        let doc_path = arguments.get("doc_path").and_then(|d| d.as_str()).unwrap_or("");
                        let content = arguments.get("content").and_then(|c| c.as_str()).unwrap_or("");

                        match McpTools::update_documentation(service, doc_path, content) {
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
                                self.send_response(&response, writer)?;
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
                                self.send_response(&response, writer)?;
                            }
                        }
                    }
                    "search_services" => {
                        if let Some(query) = arguments.get("query").and_then(|q| q.as_str()) {
                            match McpTools::search_services(query) {
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
                                    self.send_response(&response, writer)?;
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
                                    self.send_response(&response, writer)?;
                                }
                            }
                        }
                    }
                    "search_by_feature" => {
                        if let Some(feature) = arguments.get("feature").and_then(|f| f.as_str()) {
                            match McpTools::search_by_feature(feature) {
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
                                    self.send_response(&response, writer)?;
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
                                    self.send_response(&response, writer)?;
                                }
                            }
                        }
                    }
                    "search_by_keyword" => {
                        if let Some(keyword) = arguments.get("keyword").and_then(|k| k.as_str()) {
                            match McpTools::search_by_keyword(keyword) {
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
                                    self.send_response(&response, writer)?;
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
                                    self.send_response(&response, writer)?;
                                }
                            }
                        }
                    }
                    "automated_documentation_workflow" => {
                        let service = arguments.get("service").and_then(|s| s.as_str()).unwrap_or("");
                        let source = arguments.get("source").and_then(|s| s.as_str()).unwrap_or("");

                        match McpTools::automated_documentation_workflow(service, source) {
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
                                self.send_response(&response, writer)?;
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
                                self.send_response(&response, writer)?;
                            }
                        }
                    }
                    "detect_service_name" => {
                        match McpTools::detect_service_name() {
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
                                self.send_response(&response, writer)?;
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
                                self.send_response(&response, writer)?;
                            }
                        }
                    }
                    "get_repository_info" => {
                        match McpTools::get_repository_info() {
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
                                self.send_response(&response, writer)?;
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
                                self.send_response(&response, writer)?;
                            }
                        }
                    }
                        _ => {
                            let response = json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {
                                    "code": -32601,
                                    "message": "Method not found",
                                    "data": format!("Unknown tool: {}", tool_name)
                                }
                            });
                            self.send_response(&response, writer)?;
                        }
                    }
                }
            }
            "ping" => {
                // Only send response if this is a request (has ID), not a notification
                if !is_notification {
                    let response = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {}
                    });
                    self.send_response(&response, writer)?;
                }
            }
            _ => {
                // Only send response if this is a request (has ID), not a notification
                if !is_notification {
                    let response = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": "Method not found",
                            "data": format!("Unknown method: {}", method)
                        }
                    });
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

    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping MCP server");
        Ok(())
    }
}