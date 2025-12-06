pub mod client;
pub mod prompts;
pub mod providers;

pub use client::AIClient;
pub use providers::{AIProvider, AIProviderFactory, OpenAIConfig, ClaudeConfig};