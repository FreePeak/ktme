use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Service entity representing a codebase/project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: i64,
    pub name: String,
    pub path: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Document mapping linking a service to a documentation location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMapping {
    pub id: i64,
    pub service_id: i64,
    pub provider: String,
    pub location: String,
    pub title: Option<String>,
    pub section: Option<String>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Provider configuration stored as JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: i64,
    pub provider_type: String,
    pub config: serde_json::Value,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Prompt template for AI generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub template: String,
    pub variables: Vec<PromptVariable>,
    pub output_format: String,
    pub is_builtin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Variable definition for prompt templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVariable {
    pub name: String,
    pub description: String,
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
}

/// Document template for output formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplate {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub template_type: Option<String>,
    pub is_builtin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Record of a documentation generation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRecord {
    pub id: i64,
    pub service_id: Option<i64>,
    pub provider: String,
    pub document_id: Option<String>,
    pub document_url: Option<String>,
    pub action: String,
    pub source_type: Option<String>,
    pub source_identifier: Option<String>,
    pub content_hash: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Cached diff data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffCache {
    pub id: i64,
    pub source_type: String,
    pub source_identifier: String,
    pub repository_path: Option<String>,
    pub diff_json: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Confluence provider configuration
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

/// Markdown provider configuration
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

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            base_path: ".".to_string(),
            extension: "md".to_string(),
            auto_create_dirs: true,
        }
    }
}

/// Generation action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationAction {
    Create,
    Update,
    UpdateSection,
}

impl std::fmt::Display for GenerationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create => write!(f, "create"),
            Self::Update => write!(f, "update"),
            Self::UpdateSection => write!(f, "update_section"),
        }
    }
}

/// Generation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStatus {
    Success,
    Failed,
    Pending,
}

impl std::fmt::Display for GenerationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failed => write!(f, "failed"),
            Self::Pending => write!(f, "pending"),
        }
    }
}

/// Source types for diff extraction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Commit,
    Staged,
    Pr,
    Range,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Commit => write!(f, "commit"),
            Self::Staged => write!(f, "staged"),
            Self::Pr => write!(f, "pr"),
            Self::Range => write!(f, "range"),
        }
    }
}
