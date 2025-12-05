pub mod config;
pub mod confluence;
pub mod markdown;

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Represents a document in any provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub url: Option<String>,
    pub parent_id: Option<String>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub author: Option<String>,
    pub version: Option<u32>,
    pub labels: Vec<String>,
}

/// Result of a publish operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub document_id: String,
    pub url: String,
    pub version: u32,
    pub status: PublishStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PublishStatus {
    Created,
    Updated,
    NoChanges,
    Failed(String),
}

/// Core trait for all document providers
#[async_trait]
pub trait DocumentProvider: Send + Sync {
    /// Get provider name (e.g., "confluence", "google_docs")
    fn name(&self) -> &str;

    /// Check if provider is properly configured and accessible
    async fn health_check(&self) -> Result<bool>;

    /// Get a document by ID
    async fn get_document(&self, id: &str) -> Result<Option<Document>>;

    /// Find a document by title/path
    async fn find_document(&self, title: &str) -> Result<Option<Document>>;

    /// Create a new document
    async fn create_document(&self, doc: &Document) -> Result<PublishResult>;

    /// Update an existing document
    async fn update_document(&self, id: &str, content: &str) -> Result<PublishResult>;

    /// Update a specific section within a document
    async fn update_section(&self, id: &str, section: &str, content: &str) -> Result<PublishResult>;

    /// Delete a document
    async fn delete_document(&self, id: &str) -> Result<()>;

    /// List documents in a container (space, folder, etc.)
    async fn list_documents(&self, container: &str) -> Result<Vec<Document>>;

    /// Search for documents
    async fn search_documents(&self, query: &str) -> Result<Vec<Document>>;

    /// Get provider-specific configuration
    fn config(&self) -> &config::ProviderConfig;
}

/// Provider factory for creating provider instances
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(provider_type: &str, config: config::ProviderConfig) -> Result<Box<dyn DocumentProvider>> {
        match provider_type {
            "confluence" => {
                let confluence_config: config::ConfluenceConfig = serde_json::from_value(config.config.clone())
                    .map_err(|e| crate::error::KtmeError::Config(e.to_string()))?;
                Ok(Box::new(confluence::ConfluenceProvider::new(confluence_config)))
            },
            "markdown" => {
                let markdown_config: config::MarkdownConfig = serde_json::from_value(config.config.clone())
                    .map_err(|e| crate::error::KtmeError::Config(e.to_string()))?;
                Ok(Box::new(markdown::MarkdownProvider::new(markdown_config)))
            },
            _ => Err(crate::error::KtmeError::UnsupportedProvider(
                format!("Provider '{}' is not supported", provider_type)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_factory_confluence() {
        let config = config::ProviderConfig {
            id: 1,
            provider_type: "confluence".to_string(),
            config: serde_json::json!({
                "base_url": "https://example.atlassian.net",
                "username": "test@example.com",
                "api_token": "token",
                "space_key": "DEV"
            }),
            is_default: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let provider = ProviderFactory::create("confluence", config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_factory_unsupported() {
        let config = config::ProviderConfig {
            id: 1,
            provider_type: "unsupported".to_string(),
            config: serde_json::json!({}),
            is_default: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let provider = ProviderFactory::create("unsupported", config);
        assert!(provider.is_err());
    }
}