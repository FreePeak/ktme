use crate::ai::AIClient;
use crate::error::Result;
use crate::mcp::protocol::McpProtocolHandler;
use serde_json::Value;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::Arc;
use tokio::sync::RwLock;

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

#[derive(Clone)]
pub struct ServerState {
    pub running: Arc<RwLock<bool>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            running: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    pub async fn shutdown(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }
}

pub struct McpServer {
    config: ServerConfig,
    protocol_handler: McpProtocolHandler,
    ai_client: Option<AIClient>,
    state: ServerState,
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
            state: ServerState::new(),
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

    pub fn state(&self) -> ServerState {
        self.state.clone()
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
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        tracing::info!("HTTP/SSE server listening on port {}", port);

        let state = self.state.clone();
        let protocol_handler = self.protocol_handler.clone();

        loop {
            // Check if server should shutdown
            if !state.is_running().await {
                tracing::info!("Shutting down HTTP server");
                break;
            }

            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((socket, addr)) => {
                            tracing::debug!("New connection from: {}", addr);
                            let state_clone = state.clone();
                            let handler_clone = protocol_handler.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_http_connection(socket, state_clone, handler_clone).await {
                                    tracing::error!("Error handling HTTP connection: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::error!("Error accepting connection: {}", e);
                        }
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    // Periodic check for shutdown
                    continue;
                }
            }
        }

        Ok(())
    }

    async fn handle_http_connection(
        socket: tokio::net::TcpStream,
        state: ServerState,
        protocol_handler: McpProtocolHandler,
    ) -> Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

        let (reader, mut writer) = socket.into_split();
        let mut reader = BufReader::new(reader);

        // Read HTTP request line
        let mut request_line = String::new();
        reader.read_line(&mut request_line).await?;

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(());
        }

        let method = parts[0];
        let path = parts[1];

        tracing::debug!("HTTP Request: {} {}", method, path);

        // Read headers and extract Content-Length
        let mut content_length: usize = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            if line == "\r\n" || line == "\n" || line.is_empty() {
                break;
            }

            // Parse Content-Length header
            if line.to_lowercase().starts_with("content-length:") {
                if let Some(len_str) = line.split(':').nth(1) {
                    content_length = len_str.trim().parse().unwrap_or(0);
                }
            }
        }

        // Handle different endpoints
        match (method, path) {
            ("GET", "/status") => {
                let tools_count = McpProtocolHandler::get_tools_list().len();
                let status_json = serde_json::json!({
                    "status": "running",
                    "version": env!("CARGO_PKG_VERSION"),
                    "server_name": "ktme-mcp-server",
                    "tools_count": tools_count,
                    "transport": "http"
                });

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                    status_json.to_string()
                );
                writer.write_all(response.as_bytes()).await?;
                writer.flush().await?;
            }
            ("POST", "/shutdown") => {
                state.shutdown().await;

                let response_json = serde_json::json!({
                    "status": "shutdown",
                    "message": "Server shutting down gracefully"
                });

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                    response_json.to_string()
                );
                writer.write_all(response.as_bytes()).await?;
                writer.flush().await?;
            }
            ("POST", "/mcp") => {
                // Handle MCP JSON-RPC request via POST
                let body = if content_length > 0 {
                    let mut buffer = vec![0u8; content_length];
                    reader.read_exact(&mut buffer).await?;
                    String::from_utf8_lossy(&buffer).to_string()
                } else {
                    String::new()
                };

                tracing::debug!("MCP Request body: {}", body);

                match protocol_handler.handle_message(&body).await {
                    Ok(Some(response)) => {
                        let http_response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                            response.to_string()
                        );
                        writer.write_all(http_response.as_bytes()).await?;
                        writer.flush().await?;
                    }
                    Ok(None) => {
                        // Notification - 204 No Content
                        let response = "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n";
                        writer.write_all(response.as_bytes()).await?;
                        writer.flush().await?;
                    }
                    Err(e) => {
                        let error_json = serde_json::json!({
                            "error": e.to_string()
                        });
                        let response = format!(
                            "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                            error_json.to_string()
                        );
                        writer.write_all(response.as_bytes()).await?;
                        writer.flush().await?;
                    }
                }
            }
            _ => {
                // 404 Not Found
                let response = "HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n";
                writer.write_all(response.as_bytes()).await?;
                writer.flush().await?;
            }
        }

        Ok(())
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
