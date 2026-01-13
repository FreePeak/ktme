use crate::ai::AIClient;
use crate::error::Result;
use crate::mcp::protocol::McpProtocolHandler;
use serde_json::Value;
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
    protocol_handler: McpProtocolHandler,
    ai_client: Option<AIClient>,
}

impl McpServer {
    pub fn new(config: ServerConfig) -> Result<Self> {
        let protocol_handler = McpProtocolHandler::new(
            config.server_name.clone(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        let mut server = Self {
            config,
            protocol_handler,
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
        tracing::info!(
            "Starting MCP server: {} (transport: {})",
            self.config.server_name,
            self.config.transport
        );

        match self.config.transport.as_str() {
            "stdio" => self.run_stdio_server().await,
            "sse" | "http" => {
                if let Some(port) = self.config.port {
                    self.run_sse_server(port).await
                } else {
                    return Err(crate::error::KtmeError::Config(
                        "HTTP/SSE transport requires port configuration".to_string(),
                    ));
                }
            }
            _ => {
                return Err(crate::error::KtmeError::Config(format!(
                    "Unsupported transport: {}",
                    self.config.transport
                )));
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

                    match self.handle_message(trimmed, &mut stdout).await {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error handling message: {}", e);
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
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        tracing::info!("SSE server listening on port {}", port);

        loop {
            let (socket, _) = listener.accept().await?;
            let (mut reader, mut writer) = socket.into_split();

            // Handle SSE connection
            tokio::spawn(async move {
                // Send SSE headers
                let _ = writer.write_all(b"HTTP/1.1 200 OK\r\n").await;
                let _ = writer
                    .write_all(b"Content-Type: text/event-stream\r\n")
                    .await;
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
        tracing::debug!("Received: {}", message);

        match self.protocol_handler.handle_message(message).await {
            Ok(Some(response)) => {
                self.send_response(&response, writer)?;
            }
            Ok(None) => {
                // Notification - no response needed
            }
            Err(e) => {
                tracing::error!("Error handling message: {}", e);
                // Try to parse the request to get the ID for error response
                if let Ok(parsed_request) = serde_json::from_str::<Value>(message) {
                    if let Some(request_id) = parsed_request.get("id") {
                        if !request_id.is_null() {
                            let error_response = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": request_id,
                                "error": {
                                    "code": -32603,
                                    "message": "Internal error",
                                    "data": e.to_string()
                                }
                            });
                            self.send_response(&error_response, writer)?;
                        }
                    }
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
