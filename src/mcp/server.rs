use crate::error::Result;
use crate::mcp::tools::McpTools;
use crate::ai::AIClient;
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};
use uuid::Uuid;

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
                tracing::info!("AI client initialized: {}", client.provider_name());
                server.ai_client = Some(client);
            }
            Err(e) => {
                tracing::warn!("AI client not available: {}. MCP server will run without AI capabilities.", e);
            }
        }

        Ok(server)
    }

    
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting MCP server: {} (transport: {})",
                   self.config.server_name, self.config.transport);

        match self.config.transport.as_str() {
            "stdio" => self.run_stdio_server().await?,
            "http" => {
                if let Some(port) = self.config.port {
                    self.run_http_server(port).await?;
                } else {
                    return Err(crate::error::KtmeError::Config(
                        "HTTP transport requires port configuration".to_string()
                    ));
                }
            }
            _ => {
                return Err(crate::error::KtmeError::Config(
                    format!("Unsupported transport: {}", self.config.transport)
                ));
            }
        }

        Ok(())
    }

    async fn run_stdio_server(&self) -> Result<()> {
        tracing::info!("Starting STDIO MCP server");

        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut stdout = io::stdout();

        // Send initialize message
        let init_response = json!({
            "jsonrpc": "2.0",
            "id": Uuid::new_v4().to_string(),
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": self.config.server_name,
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        writeln!(stdout, "{}", init_response)?;

        // Process incoming messages
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    match self.handle_message(&line.trim(), &mut stdout).await {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("Error handling message: {}", e);
                            let error_response = json!({
                                "jsonrpc": "2.0",
                                "id": null,
                                "error": {
                                    "code": -32603,
                                    "message": "Internal error",
                                    "data": e.to_string()
                                }
                            });
                            writeln!(stdout, "{}", error_response)?;
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

    async fn run_http_server(&self, port: u16) -> Result<()> {
        tracing::info!("Starting HTTP MCP server on port {}", port);

        // Simple HTTP server implementation
        let listener = std::net::TcpListener::bind(format!("127.0.0.1:{}", port))?;
        tracing::info!("HTTP server listening on {}", port);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    tracing::debug!("New HTTP connection");
                    if let Err(e) = self.handle_http_connection(stream).await {
                        tracing::error!("Error handling HTTP connection: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Error accepting connection: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn handle_http_connection(&self, mut stream: std::net::TcpStream) -> Result<()> {
        use std::io::{Read, BufReader};

        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        if request_line.starts_with("POST") {
            // Read headers
            let mut content_length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line)?;
                if line.trim().is_empty() {
                    break;
                }
                if line.to_lowercase().starts_with("content-length:") {
                    content_length = line.split(':')
                        .nth(1)
                        .unwrap_or("0")
                        .trim()
                        .parse()
                        .unwrap_or(0);
                }
            }

            // Read body
            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body)?;

            let request_str = String::from_utf8_lossy(&body);
            let _response = self.handle_message(&request_str, &mut stream).await;

            // Send simple HTTP response
            let http_response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\"}";
            stream.write_all(http_response.as_bytes())?;
        }

        Ok(())
    }

    async fn handle_message(&self, message: &str, writer: &mut impl Write) -> Result<()> {
        tracing::debug!("Received MCP message: {}", message);

        let parsed: Value = serde_json::from_str(message)
            .map_err(|e| crate::error::KtmeError::Serialization(e))?;

        let id = parsed.get("id").cloned();
        let method = parsed.get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let response = match method {
            "initialize" => self.handle_initialize(id).await?,
            "tools/list" => self.handle_tools_list(id).await?,
            "tools/call" => self.handle_tools_call(id, parsed.get("params")).await?,
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            }),
        };

        writeln!(writer, "{}", response)?;
        writer.flush()?;
        Ok(())
    }

    async fn handle_initialize(&self, id: Option<Value>) -> Result<Value> {
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": self.config.server_name,
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        }))
    }

    async fn handle_tools_list(&self, id: Option<Value>) -> Result<Value> {
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": [
                    {
                        "name": "extract_changes",
                        "description": "Extract code changes from Git commits, PRs, or staged changes",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "source": {
                                    "type": "string",
                                    "enum": ["commit", "pr", "staged"],
                                    "description": "Source of changes to extract"
                                },
                                "identifier": {
                                    "type": "string",
                                    "description": "Commit hash, PR number, or 'staged'"
                                },
                                "provider": {
                                    "type": "string",
                                    "enum": ["github", "gitlab", "bitbucket"],
                                    "description": "Git provider for PR extraction"
                                }
                            },
                            "required": ["source"]
                        }
                    },
                    {
                        "name": "generate_documentation",
                        "description": "Generate documentation from code changes using AI",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "service": {
                                    "type": "string",
                                    "description": "Service name for documentation"
                                },
                                "doc_type": {
                                    "type": "string",
                                    "enum": ["changelog", "api-doc", "readme", "general"],
                                    "description": "Type of documentation to generate"
                                },
                                "provider": {
                                    "type": "string",
                                    "enum": ["markdown", "confluence"],
                                    "description": "Where to save the documentation"
                                },
                                "format": {
                                    "type": "string",
                                    "enum": ["markdown", "json"],
                                    "description": "Output format"
                                }
                            },
                            "required": ["service"]
                        }
                    },
                    {
                        "name": "read_changes",
                        "description": "Read extracted changes from a file",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                    "type": "string",
                                    "description": "Path to the diff JSON file"
                                }
                            },
                            "required": ["file_path"]
                        }
                    }
                ]
            }
        }))
    }

    async fn handle_tools_call(&self, id: Option<Value>, params: Option<&Value>) -> Result<Value> {
        let default_params = json!({});
        let params = params.unwrap_or(&default_params);

        let tool_name = params.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let default_args = json!({});
        let arguments = params.get("arguments").unwrap_or(&default_args);

        let result = match tool_name {
            "extract_changes" => self.extract_changes(arguments).await,
            "generate_documentation" => self.generate_documentation(arguments).await,
            "read_changes" => self.read_changes(arguments).await,
            _ => Ok(format!("Unknown tool: {}", tool_name)),
        };

        match result {
            Ok(content) => Ok(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [
                        {
                            "type": "text",
                            "text": content
                        }
                    ]
                }
            })),
            Err(e) => Ok(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32000,
                    "message": "Tool execution failed",
                    "data": e.to_string()
                }
            })),
        }
    }

    async fn extract_changes(&self, arguments: &Value) -> Result<String> {
        let source = arguments.get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("commit");

        let identifier = arguments.get("identifier")
            .and_then(|v| v.as_str())
            .unwrap_or("HEAD");

        // Use existing extract functionality
        use crate::git::diff::DiffExtractor;

        let extractor = DiffExtractor::new(
            source.to_string(),
            identifier.to_string(),
            None,
        )?;

        let diff = extractor.extract()?;

        // Save to file for later use
        let filename = format!("/tmp/ktme_extract_{}.json", Uuid::new_v4());
        std::fs::write(&filename, serde_json::to_string_pretty(&diff)?)?;

        Ok(format!("Changes extracted successfully. File saved to: {}", filename))
    }

    async fn generate_documentation(&self, arguments: &Value) -> Result<String> {
        let service = arguments.get("service")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::KtmeError::InvalidInput(
                "Service name is required".to_string()
            ))?;

        let doc_type = arguments.get("doc_type")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let provider = arguments.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("markdown");

        // Extract current changes if no input specified
        let diff = if let Some(input_file) = arguments.get("input_file").and_then(|v| v.as_str()) {
            // Load from file
            let content = std::fs::read_to_string(input_file)?;
            serde_json::from_str::<crate::git::diff::ExtractedDiff>(&content)?
        } else {
            // Extract from HEAD
            use crate::git::diff::DiffExtractor;
            let extractor = DiffExtractor::new(
                "commit".to_string(),
                "HEAD".to_string(),
                None,
            )?;
            extractor.extract()?
        };

        // Generate documentation if AI client is available
        if let Some(ai_client) = &self.ai_client {
            use crate::ai::prompts::PromptTemplates;

            let prompt = PromptTemplates::generate_documentation_prompt(&diff, doc_type, None)?;
            let documentation = ai_client.generate_documentation(&prompt).await?;

            // Save using appropriate provider
            if provider == "confluence" {
                self.save_to_confluence(service, &documentation, doc_type).await?;
            } else {
                self.save_to_markdown(service, &documentation, doc_type).await?;
            }

            Ok(format!("Documentation generated successfully for service: {}", service))
        } else {
            Err(crate::error::KtmeError::Config(
                "AI client not available. Cannot generate documentation.".to_string()
            ))
        }
    }

    async fn read_changes(&self, arguments: &Value) -> Result<String> {
        let file_path = arguments.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::KtmeError::InvalidInput(
                "File path is required".to_string()
            ))?;

        let content = std::fs::read_to_string(file_path)?;
        Ok(content)
    }

    async fn save_to_markdown(&self, service: &str, documentation: &str, doc_type: &str) -> Result<()> {
        let filename = format!("docs/{}_{}.md", service, doc_type);

        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&filename).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");
        let content = format!(
            "# Documentation for {}\n\n**Type**: {}\n**Generated**: {}\n\n---\n\n{}",
            service, doc_type, timestamp, documentation
        );

        std::fs::write(&filename, content)?;
        tracing::info!("Documentation saved to: {}", filename);
        Ok(())
    }

    async fn save_to_confluence(&self, _service: &str, _documentation: &str, _doc_type: &str) -> Result<()> {
        // TODO: Implement Confluence provider
        Err(crate::error::KtmeError::UnsupportedProvider(
            "Confluence provider not yet implemented".to_string()
        ))
    }

    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping MCP server");
        Ok(())
    }
}
