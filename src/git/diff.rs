use crate::error::Result;
use crate::git::reader::GitReader;
use serde::{Deserialize, Serialize};

// Unit tests are in tests/ directory to avoid access issues

#[derive(Debug)]
pub struct DiffExtractor {
    pub source: String,
    pub identifier: String,
    git_reader: GitReader,
}

impl DiffExtractor {
    pub fn new(source: String, identifier: String, path: Option<&str>) -> Result<Self> {
        let git_reader = GitReader::new(path)?;
        Ok(Self { source, identifier, git_reader })
    }

    pub fn extract(&self) -> Result<ExtractedDiff> {
        tracing::info!("Extracting diff from {} {}", self.source, self.identifier);

        match self.source.as_str() {
            "commit" => {
                self.git_reader.read_commit(&self.identifier)
            },
            "staged" => {
                self.git_reader.read_staged()
            },
            _ => {
                Err(crate::error::KtmeError::InvalidInput(
                    format!("Unsupported source type: {}", self.source)
                ))
            }
        }
    }

    pub fn extract_range(&self, range: &str) -> Result<Vec<ExtractedDiff>> {
        tracing::info!("Extracting diff range from {} {}", self.source, range);

        match self.source.as_str() {
            "commit" | "range" => {
                self.git_reader.read_commit_range(range)
            },
            _ => {
                Err(crate::error::KtmeError::InvalidInput(
                    format!("Range extraction not supported for source type: {}", self.source)
                ))
            }
        }
    }

    pub fn get_repository_info(&self) -> Result<RepositoryInfo> {
        let path = self.git_reader.get_repository_path()?;
        let branch = self.git_reader.get_current_branch()?;
        let status = self.git_reader.get_status()?;

        Ok(RepositoryInfo {
            path,
            branch,
            status: status.into_iter().map(|(path, status)| FileStatus {
                path,
                status: format!("{:?}", status),
            }).collect(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub path: String,
    pub branch: String,
    pub status: Vec<FileStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub status: String,
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
