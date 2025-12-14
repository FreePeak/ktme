use crate::error::Result;
use crate::git::reader::GitReader;
use crate::storage::mapping::StorageManager;
use serde_json;

pub struct McpTools;

impl McpTools {
    pub fn read_changes(file_path: &str) -> Result<String> {
        tracing::info!("MCP Tool: read_changes({})", file_path);
        
        // Check if file_path is a Git reference or a file
        if file_path.starts_with("commit:") {
            let commit_ref = &file_path[7..]; // Remove "commit:" prefix
            let reader = GitReader::new(None)?;
            let diff = reader.read_commit(commit_ref)?;
            Ok(serde_json::to_string_pretty(&diff)?)
        } else if file_path == "staged" {
            let reader = GitReader::new(None)?;
            let diff = reader.read_staged()?;
            Ok(serde_json::to_string_pretty(&diff)?)
        } else if file_path.contains("..") {
            let reader = GitReader::new(None)?;
            let diffs = reader.read_commit_range(file_path)?;
            Ok(serde_json::to_string_pretty(&diffs)?)
        } else {
            // Try to read as a file containing diff content
            std::fs::read_to_string(file_path)
                .map_err(|e| crate::error::KtmeError::Io(e))
        }
    }

    pub fn get_service_mapping(service: &str) -> Result<String> {
        tracing::info!("MCP Tool: get_service_mapping({})", service);
        
        let storage = StorageManager::new()?;
        let mapping = storage.get_mapping(service)?;
        Ok(serde_json::to_string_pretty(&mapping)?)
    }

    pub fn list_services() -> Result<Vec<String>> {
        tracing::info!("MCP Tool: list_services()");
        
        let storage = StorageManager::new()?;
        storage.list_services()
    }

    pub fn generate_documentation(service: &str, changes: &str, format: Option<&str>) -> Result<String> {
        tracing::info!("MCP Tool: generate_documentation(service={}, format={:?})", service, format);
        
        // Parse the changes
        let diff: crate::git::diff::ExtractedDiff = serde_json::from_str(changes)
            .map_err(|_| crate::error::KtmeError::InvalidInput("Invalid changes format".to_string()))?;
        
        // Generate basic documentation
        let doc_content = format!("# Documentation for {}\n\n## Changes\n\n**Source:** {}\n**Author:** {}\n**Timestamp:** {}\n\n### Summary\n- {} files changed\n- {} additions\n- {} deletions\n\n### Files Modified\n{}\n\n### Commit Message\n{}\n",
            service,
            diff.source,
            diff.author,
            diff.timestamp,
            diff.summary.total_files,
            diff.summary.total_additions,
            diff.summary.total_deletions,
            diff.files.iter()
                .map(|f| format!("- **{}**: {} ({}, +{}/-{})", f.path, f.status, f.path, f.additions, f.deletions))
                .collect::<Vec<_>>()
                .join("\n"),
            diff.message
        );
        
        Ok(doc_content)
    }

    pub fn update_documentation(service: &str, doc_path: &str, content: &str) -> Result<String> {
        tracing::info!("MCP Tool: update_documentation(service={}, doc_path={})", service, doc_path);
        
        // For now, just write to the file
        std::fs::write(doc_path, content)
            .map_err(|e| crate::error::KtmeError::Io(e))?;
        
        Ok(format!("Documentation updated at {}", doc_path))
    }
}
