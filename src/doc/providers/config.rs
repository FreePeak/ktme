use serde::{Deserialize, Serialize};
use crate::storage::models::{ProviderConfig as DbProviderConfig};

/// Provider configuration from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: i64,
    pub provider_type: String,
    pub config: serde_json::Value,
    pub is_default: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<DbProviderConfig> for ProviderConfig {
    fn from(db_config: DbProviderConfig) -> Self {
        Self {
            id: db_config.id,
            provider_type: db_config.provider_type,
            config: db_config.config,
            is_default: db_config.is_default,
            created_at: db_config.created_at,
            updated_at: db_config.updated_at,
        }
    }
}

/// Confluence-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfluenceConfig {
    pub base_url: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,
    pub space_key: String,
    #[serde(default)]
    pub default_parent_id: Option<String>,
    #[serde(default)]
    pub default_labels: Vec<String>,
    #[serde(default = "default_true")]
    pub is_cloud: bool,
}


fn default_true() -> bool {
    true
}

/// Markdown-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownConfig {
    pub base_path: String,
    #[serde(default = "default_extension")]
    pub extension: String,
    #[serde(default = "default_true")]
    pub auto_create_dirs: bool,
}


fn default_extension() -> String {
    "md".to_string()
}

/// Provider registry for managing multiple providers
#[derive(Debug, Clone)]
pub struct ProviderRegistry {
    pub providers: std::collections::HashMap<String, ProviderConfig>,
    pub default_provider: Option<String>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
            default_provider: None,
        }
    }

    pub fn add_provider(&mut self, config: ProviderConfig) {
        let provider_type = config.provider_type.clone();
        if config.is_default {
            self.default_provider = Some(provider_type.clone());
        }
        self.providers.insert(provider_type, config);
    }

    pub fn get_provider(&self, provider_type: &str) -> Option<&ProviderConfig> {
        self.providers.get(provider_type)
    }

    pub fn get_default_provider(&self) -> Option<&ProviderConfig> {
        self.default_provider.as_ref()
            .and_then(|ptype| self.providers.get(ptype))
    }

    pub fn list_providers(&self) -> Vec<&ProviderConfig> {
        self.providers.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_registry() {
        let mut registry = ProviderRegistry::new();

        let config = ProviderConfig {
            id: 1,
            provider_type: "confluence".to_string(),
            config: serde_json::json!({}),
            is_default: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        registry.add_provider(config);

        assert!(registry.get_provider("confluence").is_some());
        assert!(registry.get_default_provider().is_some());
        assert_eq!(registry.list_providers().len(), 1);
    }
}