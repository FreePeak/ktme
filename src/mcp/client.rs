use crate::error::Result;

pub struct McpClient {
    model: String,
}

impl McpClient {
    pub fn new(model: String) -> Self {
        Self { model }
    }

    pub async fn send_request(&self, prompt: &str) -> Result<String> {
        tracing::info!("Sending request to MCP server using model: {}", self.model);
        // TODO: Implement MCP client communication
        Ok("Response placeholder".to_string())
    }
}
