use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Configuration for OpenAI API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    pub base_url: Option<String>,
}

/// Configuration for Anthropic Claude API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    pub api_key: String,
    #[serde(default = "default_claude_model")]
    pub model: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

/// Generic AI provider trait
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String>;
    fn provider_name(&self) -> &'static str;
}

/// Factory for creating AI providers
pub struct AIProviderFactory;

impl AIProviderFactory {
    pub fn create_openai(config: OpenAIConfig) -> Result<Box<dyn AIProvider>> {
        Ok(Box::new(OpenAIProvider::new(config)))
    }

    pub fn create_claude(config: ClaudeConfig) -> Result<Box<dyn AIProvider>> {
        Ok(Box::new(ClaudeProvider::new(config)))
    }

    pub fn create_mock() -> Result<Box<dyn AIProvider>> {
        Ok(Box::new(MockProvider::new()))
    }
}

// OpenAI Provider Implementation
pub struct OpenAIProvider {
    client: reqwest::Client,
    config: OpenAIConfig,
}

impl OpenAIProvider {
    pub fn new(config: OpenAIConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("ktme/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let base_url = self.config.base_url.as_deref()
            .unwrap_or("https://api.openai.com/v1");

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature
        });

        let response = self.client
            .post(&format!("{}/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::error::KtmeError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::error::KtmeError::ApiError(
                format!("OpenAI API error: {} - {}", status, error_text)
            ));
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<OpenAIChoice>,
        }

        #[derive(Deserialize)]
        struct OpenAIChoice {
            message: OpenAIMessage,
        }

        #[derive(Deserialize)]
        struct OpenAIMessage {
            content: String,
        }

        let openai_response: OpenAIResponse = response.json().await
            .map_err(|e| crate::error::KtmeError::DeserializationError(e.to_string()))?;

        openai_response.choices
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::KtmeError::ApiError("No response from OpenAI".to_string()))
            .map(|choice| choice.message.content)
    }

    fn provider_name(&self) -> &'static str {
        "OpenAI"
    }
}

// Claude Provider Implementation
pub struct ClaudeProvider {
    client: reqwest::Client,
    config: ClaudeConfig,
}

impl ClaudeProvider {
    pub fn new(config: ClaudeConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("ktme/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let request_body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::error::KtmeError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::error::KtmeError::ApiError(
                format!("Claude API error: {} - {}", status, error_text)
            ));
        }

        #[derive(Deserialize)]
        struct ClaudeResponse {
            content: Vec<ClaudeContent>,
        }

        #[derive(Deserialize)]
        struct ClaudeContent {
            #[serde(rename = "type")]
            content_type: String,
            text: String,
        }

        let claude_response: ClaudeResponse = response.json().await
            .map_err(|e| crate::error::KtmeError::DeserializationError(e.to_string()))?;

        claude_response.content
            .into_iter()
            .find(|c| c.content_type == "text")
            .map(|c| c.text)
            .ok_or_else(|| crate::error::KtmeError::ApiError("No text response from Claude".to_string()))
    }

    fn provider_name(&self) -> &'static str {
        "Claude"
    }
}

// Mock Provider for testing without API keys
pub struct MockProvider;

impl MockProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AIProvider for MockProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        tracing::info!("Using mock provider to generate documentation");

        // Simple mock documentation generation based on prompt analysis
        let documentation = if prompt.contains("API") || prompt.contains("endpoint") || prompt.contains("api-doc") {
            format!(
                "# API Documentation

## Overview
This section documents the API endpoints and their usage.

## Endpoints
- GET /services - List all services
  - Query Parameters: `q` (optional) - Search query
  - Response: JSON array of services with mappings

- POST /mapping - Create new service mapping
  - Body: JSON object with service name, documentation type, and location
  - Response: Created mapping details

- GET /search - Search services by feature or keyword
  - Query Parameters: `feature` or `keyword`
  - Response: Ranked search results with relevance scores

## Authentication
API uses bearer token authentication for write operations.

## Error Handling
All endpoints return appropriate HTTP status codes and error messages in JSON format.

*Generated by ktme mock provider*"
            )
        } else if prompt.contains("type: \"API\"") || prompt.contains("doc_type: \"API\"") {
            format!(
                "# API Reference Documentation

## ktme API

### Service Management
The ktme API provides endpoints for managing service documentation and mappings.

#### Core Endpoints

**GET /services**
Retrieve a list of all registered services with their documentation mappings.

**POST /mapping**
Create a new documentation mapping for a service.

**GET /search**
Search services by features or keywords with relevance scoring.

### MCP Tools
- `read_changes`: Extract code changes from git references
- `generate_documentation`: Generate AI-powered documentation
- `update_documentation`: Update existing documentation files

### Usage Examples
See the implementation in `src/mcp/tools.rs` for detailed usage patterns.

*Generated by ktme mock provider*"
            )
        } else if prompt.contains("service") || prompt.contains("functionality") {
            format!(
                "# Service Documentation

## Overview
The ktme service provides automated documentation generation and knowledge transfer capabilities.

## Core Features
- **Service Detection**: Automatically detects service names from Git repositories and project files
- **Documentation Generation**: AI-powered documentation from code changes
- **Mapping Management**: SQLite-based storage of service-to-documentation mappings
- **Search Functionality**: Feature and keyword-based search with relevance scoring

## Architecture
- **CLI Interface**: Command-line tool for direct interaction
- **MCP Server**: Model Context Protocol server for AI agent integration
- **Storage Backend**: SQLite with TOML fallback support
- **AI Integration**: Support for OpenAI and Anthropic providers with mock fallback

## Recent Changes
Based on the staged changes:
- Added intelligent service name detection with Git repository scanning
- Implemented comprehensive search functionality with relevance scoring
- Enhanced MCP tools with AI integration and automated workflows
- Added mock AI provider for testing without API keys

## Configuration
- AI providers configured via environment variables
- SQLite database for persistent storage
- Configurable documentation templates and output formats

*Generated by ktme mock provider*"
            )
        } else {
            format!(
                "# Documentation

## Summary
This document describes the ktme (Knowledge Transfer Me) system implementation.

## Key Components
- Service detection and mapping from Git repositories
- AI-powered documentation generation with multiple providers
- SQLite-based storage with search capabilities
- MCP server integration for AI agent workflows

## Implementation Details
ktme is implemented in Rust with a focus on:
- Reliability and performance
- Extensible AI provider architecture
- Automated end-to-end documentation workflows
- Multi-format output support (Markdown, JSON)

## Notes
Generated from git diff analysis using ktme's automated documentation system.

*Generated by ktme mock provider*"
            )
        };

        Ok(documentation)
    }

    fn provider_name(&self) -> &'static str {
        "Mock"
    }
}

// Default values
fn default_model() -> String {
    "gpt-3.5-turbo".to_string()
}

fn default_claude_model() -> String {
    "claude-3-haiku-20240307".to_string()
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f32 {
    0.7
}