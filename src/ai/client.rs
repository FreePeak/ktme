use crate::ai::providers::{AIProvider, AIProviderFactory, ClaudeConfig, OpenAIConfig};
use crate::error::Result;
use std::env;

pub struct AIClient {
    provider: Box<dyn AIProvider>,
}

impl AIClient {
    pub fn new() -> Result<Self> {
        // Try to detect available AI provider from environment variables
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            let config = OpenAIConfig {
                api_key,
                model: env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4".to_string()),
                max_tokens: env::var("OPENAI_MAX_TOKENS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(4096),
                temperature: env::var("OPENAI_TEMPERATURE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.7),
                base_url: env::var("OPENAI_BASE_URL").ok(),
            };

            let provider = AIProviderFactory::create_openai(config)?;
            Ok(Self { provider })
        } else if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            let config = ClaudeConfig {
                api_key,
                model: env::var("CLAUDE_MODEL")
                    .unwrap_or_else(|_| "claude-3-sonnet-20240229".to_string()),
                max_tokens: env::var("CLAUDE_MAX_TOKENS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(4096),
                temperature: env::var("CLAUDE_TEMPERATURE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.7),
            };

            let provider = AIProviderFactory::create_claude(config)?;
            Ok(Self { provider })
        } else {
            tracing::warn!("No AI provider configured. Using mock provider for testing.");
            let provider = AIProviderFactory::create_mock()?;
            Ok(Self { provider })
        }
    }

    pub fn new_with_fallback() -> Result<Self> {
        match Self::new() {
            Ok(client) => Ok(client),
            Err(_) => {
                tracing::warn!("AI client creation failed, using mock provider");
                let provider = AIProviderFactory::create_mock()?;
                Ok(Self { provider })
            }
        }
    }

    pub async fn generate_documentation(&self, prompt: &str) -> Result<String> {
        tracing::info!(
            "Generating documentation using {}",
            self.provider.provider_name()
        );

        let response = self.provider.generate(prompt).await?;

        tracing::info!("Documentation generated successfully");
        Ok(response)
    }

    pub fn provider_name(&self) -> &'static str {
        self.provider.provider_name()
    }

    pub async fn test_connection(&self) -> Result<bool> {
        match self.provider.generate("Hello").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
