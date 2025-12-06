#[cfg(test)]
mod tests {
    use crate::error::KtmeError;
    use crate::ai::client::AIClient;
    use crate::ai::providers::{OpenAIProvider, ClaudeConfig, OpenAIConfig, AIProvider};

    #[test]
    fn test_ai_client_fails_without_key() {
        // Should fail when no environment variables are set
        let result = AIClient::new();
        assert!(result.is_err());

        match result.err().unwrap() {
            KtmeError::Config(msg) => {
                assert!(msg.contains("AI provider"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_openai_config_defaults() {
        let config = OpenAIConfig {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            max_tokens: 2048,
            temperature: 0.5,
            base_url: None,
        };

        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.temperature, 0.5);
        assert!(config.base_url.is_none());
    }

    #[test]
    fn test_claude_config_creation() {
        let config = ClaudeConfig {
            api_key: "test-key".to_string(),
            model: "claude-3-sonnet".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        };

        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.model, "claude-3-sonnet");
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_openai_provider_creation() {
        let config = OpenAIConfig {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            max_tokens: 2048,
            temperature: 0.5,
            base_url: None,
        };

        let provider = OpenAIProvider::new(config);
        assert_eq!(provider.provider_name(), "OpenAI");
    }

    #[tokio::test]
    async fn test_openai_provider_no_network() {
        let config = OpenAIConfig {
            api_key: "invalid-key".to_string(),
            model: "gpt-4".to_string(),
            max_tokens: 100,
            temperature: 0.5,
            base_url: Some("http://localhost:9999".to_string()), // Invalid URL
        };

        let provider = OpenAIProvider::new(config);
        let result = provider.generate("test").await;

        // Should fail with network error
        assert!(result.is_err());
    }
}