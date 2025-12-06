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