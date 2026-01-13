pub mod types;

use crate::error::{KtmeError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub use types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub documentation: DocumentationConfig,
    #[serde(default)]
    pub confluence: ConfluenceConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            git: GitConfig::default(),
            mcp: McpConfig::default(),
            documentation: DocumentationConfig::default(),
            confluence: ConfluenceConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if !config_path.exists() {
            tracing::warn!("Configuration file not found, using defaults");
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;

        tracing::info!("Configuration saved to {}", config_path.display());

        Ok(())
    }

    pub fn config_file_path() -> Result<PathBuf> {
        if let Ok(custom_path) = std::env::var("KTME_CONFIG") {
            return Ok(PathBuf::from(custom_path));
        }

        // Use ~/.config/ktme explicitly
        let home_dir = dirs::home_dir()
            .ok_or_else(|| KtmeError::Config("Could not determine home directory".to_string()))?;
        let config_dir = home_dir.join(".config").join("ktme");

        Ok(config_dir.join("config.toml"))
    }

    pub fn config_dir() -> Result<PathBuf> {
        let config_path = Self::config_file_path()?;
        Ok(config_path
            .parent()
            .ok_or_else(|| KtmeError::Config("Invalid config path".to_string()))?
            .to_path_buf())
    }
}
