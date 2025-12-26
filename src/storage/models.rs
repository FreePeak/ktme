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

/// Feature entity representing a software feature within a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub service_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub feature_type: FeatureType,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub relevance_score: f64,
    pub embedding: Option<Vec<f32>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Feature type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureType {
    Api,
    Ui,
    BusinessLogic,
    Config,
    Database,
    Security,
    Performance,
    Testing,
    Deployment,
    Other,
}

impl std::fmt::Display for FeatureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api => write!(f, "api"),
            Self::Ui => write!(f, "ui"),
            Self::BusinessLogic => write!(f, "business_logic"),
            Self::Config => write!(f, "config"),
            Self::Database => write!(f, "database"),
            Self::Security => write!(f, "security"),
            Self::Performance => write!(f, "performance"),
            Self::Testing => write!(f, "testing"),
            Self::Deployment => write!(f, "deployment"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Relationship between features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRelation {
    pub id: String,
    pub parent_feature_id: String,
    pub child_feature_id: String,
    pub relation_type: RelationType,
    pub strength: f64,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Type of relationship between features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    DependsOn,
    Implements,
    Extends,
    Uses,
    Configures,
    Tests,
    Deploys,
    Other,
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DependsOn => write!(f, "depends_on"),
            Self::Implements => write!(f, "implements"),
            Self::Extends => write!(f, "extends"),
            Self::Uses => write!(f, "uses"),
            Self::Configures => write!(f, "configures"),
            Self::Tests => write!(f, "tests"),
            Self::Deploys => write!(f, "deploys"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Search index entry for semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub id: String,
    pub feature_id: String,
    pub content_type: SearchContentType,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub indexed_at: DateTime<Utc>,
}

/// Type of content in search index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchContentType {
    FeatureName,
    FeatureDescription,
    Documentation,
    CodeExample,
    ApiReference,
    UserGuide,
    Other,
}

impl std::fmt::Display for SearchContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FeatureName => write!(f, "feature_name"),
            Self::FeatureDescription => write!(f, "feature_description"),
            Self::Documentation => write!(f, "documentation"),
            Self::CodeExample => write!(f, "code_example"),
            Self::ApiReference => write!(f, "api_reference"),
            Self::UserGuide => write!(f, "user_guide"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Search result with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub feature_id: String,
    pub service_name: String,
    pub feature_name: String,
    pub feature_type: FeatureType,
    pub description: Option<String>,
    pub content: String,
    pub relevance_score: f64,
    pub content_type: SearchContentType,
    pub path: Option<String>,
    pub tags: Vec<String>,
}

/// Knowledge graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub node_type: KnowledgeNodeType,
    pub name: String,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub relevance_score: f64,
}

/// Type of knowledge graph node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeNodeType {
    Service,
    Feature,
    Document,
    Api,
    Example,
    Concept,
}

/// Knowledge graph edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: RelationType,
    pub strength: f64,
    pub metadata: serde_json::Value,
}

/// Search query with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub service_ids: Option<Vec<i64>>,
    pub feature_types: Option<Vec<FeatureType>>,
    pub content_types: Option<Vec<SearchContentType>>,
    pub limit: Option<u32>,
    pub similarity_threshold: Option<f64>,
    pub include_related: bool,
    pub depth: Option<u32>,
}

/// Context for AI agent queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    pub query: String,
    pub intent: QueryIntent,
    pub relevant_services: Vec<Service>,
    pub relevant_features: Vec<Feature>,
    pub related_documents: Vec<DocumentMapping>,
    pub knowledge_graph: Option<KnowledgeGraph>,
    pub confidence: f64,
}

/// Detected user intent from query
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryIntent {
    FindFeature,
    UnderstandFeature,
    FindDocumentation,
    GetExamples,
    Troubleshoot,
    CompareFeatures,
    ImplementFeature,
    Other,
}

/// Knowledge graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
    pub root_nodes: Vec<String>,
    pub metadata: serde_json::Value,
}
