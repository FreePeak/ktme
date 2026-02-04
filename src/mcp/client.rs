use crate::error::Result;
use serde_json::{json, Value};

/// MCP Client for sending requests to MCP servers
pub struct McpClient {
    server_url: String,
    client: reqwest::Client,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            client: reqwest::Client::new(),
        }
    }

    /// Send a JSON-RPC request to the MCP server
    pub async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        tracing::info!("Sending MCP request: method={}", method);

        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = self
            .client
            .post(&format!("{}/mcp", self.server_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| crate::error::KtmeError::NetworkError(e.to_string()))?;

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| crate::error::KtmeError::NetworkError(e.to_string()))?;

        if let Some(error) = response_json.get("error") {
            return Err(crate::error::KtmeError::Mcp(format!(
                "MCP error: {}",
                error
            )));
        }

        response_json
            .get("result")
            .cloned()
            .ok_or_else(|| crate::error::KtmeError::Mcp("No result in response".to_string()))
    }

    /// Initialize connection to the MCP server
    pub async fn initialize(&self) -> Result<Value> {
        self.send_request("initialize", json!({})).await
    }

    /// List available tools
    pub async fn list_tools(&self) -> Result<Value> {
        self.send_request("tools/list", json!({})).await
    }

    /// Call a tool with arguments
    pub async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<Value> {
        let params = json!({
            "name": tool_name,
            "arguments": arguments
        });
        self.send_request("tools/call", params).await
    }

    /// Check server status
    pub async fn status(&self) -> Result<Value> {
        let response = self
            .client
            .get(&format!("{}/status", self.server_url))
            .send()
            .await
            .map_err(|e| crate::error::KtmeError::NetworkError(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| crate::error::KtmeError::NetworkError(e.to_string()))
    }

    /// Shutdown the server
    pub async fn shutdown(&self) -> Result<Value> {
        let response = self
            .client
            .post(&format!("{}/shutdown", self.server_url))
            .send()
            .await
            .map_err(|e| crate::error::KtmeError::NetworkError(e.to_string()))?;

        response
            .json()
            .await
            .map_err(|e| crate::error::KtmeError::NetworkError(e.to_string()))
    }
}
