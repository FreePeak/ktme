use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_temp_directory")]
    pub temp_directory: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            temp_directory: default_temp_directory(),
            log_level: default_log_level(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    #[serde(default = "default_branch")]
    pub default_branch: String,
    #[serde(default)]
    pub include_merge_commits: bool,
    #[serde(default = "default_max_commit_range")]
    pub max_commit_range: u32,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            default_branch: default_branch(),
            include_merge_commits: false,
            max_commit_range: default_max_commit_range(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub server_binary: Option<String>,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            server_binary: None,
            model: default_model(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            timeout: default_timeout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    #[serde(default = "default_format")]
    pub default_format: String,
    pub template_directory: Option<PathBuf>,
    #[serde(default = "default_include_metadata")]
    pub include_metadata: bool,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            default_format: default_format(),
            template_directory: None,
            include_metadata: default_include_metadata(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfluenceConfig {
    pub base_url: Option<String>,
    #[serde(default = "default_auth_type")]
    pub auth_type: String,
    pub api_token: Option<String>,
    pub username: Option<String>,
    pub space_key: Option<String>,
    pub default_parent_page: Option<String>,
}

impl Default for ConfluenceConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            auth_type: default_auth_type(),
            api_token: None,
            username: None,
            space_key: None,
            default_parent_page: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub mappings_file: Option<PathBuf>,
    #[serde(default)]
    pub auto_discover: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            mappings_file: None,
            auto_discover: false,
        }
    }
}

fn default_temp_directory() -> String {
    "/tmp/ktme".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_max_commit_range() -> u32 {
    100
}

fn default_model() -> String {
    "claude-3-5-sonnet-20241022".to_string()
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f32 {
    0.7
}

fn default_timeout() -> u64 {
    120
}

fn default_format() -> String {
    "markdown".to_string()
}

fn default_include_metadata() -> bool {
    true
}

fn default_auth_type() -> String {
    "token".to_string()
}
