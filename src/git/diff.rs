use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffExtractor {
    pub source: String,
    pub identifier: String,
}

impl DiffExtractor {
    pub fn new(source: String, identifier: String) -> Self {
        Self { source, identifier }
    }

    pub fn extract(&self) -> Result<ExtractedDiff> {
        tracing::info!("Extracting diff from {} {}", self.source, self.identifier);
        // TODO: Implement diff extraction
        Ok(ExtractedDiff::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractedDiff {
    pub source: String,
    pub identifier: String,
    pub timestamp: String,
    pub author: String,
    pub message: String,
    pub files: Vec<FileChange>,
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
    pub diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiffSummary {
    pub total_files: u32,
    pub total_additions: u32,
    pub total_deletions: u32,
}
