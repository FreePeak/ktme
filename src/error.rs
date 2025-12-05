use thiserror::Error;

pub type Result<T> = std::result::Result<T, KtmeError>;

#[derive(Error, Debug)]
pub enum KtmeError {
    #[error("Git operation failed: {0}")]
    Git(#[from] git2::Error),

    #[error("MCP communication failed: {0}")]
    Mcp(String),

    #[error("Documentation generation failed: {0}")]
    Documentation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Storage operation failed: {0}")]
    Storage(String),

    #[error("Confluence API error: {0}")]
    Confluence(String),

    #[error("Service mapping not found: {0}")]
    MappingNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Document already exists: {0}")]
    DocumentExists(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
