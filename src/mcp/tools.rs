use crate::ai::AIClient;
use crate::error::Result;
use crate::git::reader::GitReader;
use crate::service_detector::ServiceDetector;
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
        } else if file_path == "HEAD"
            || file_path == "HEAD~1"
            || file_path.len() == 7
            || file_path.len() == 40
        {
            // Handle raw commit hashes and Git references
            let reader = GitReader::new(None)?;
            let diff = reader.read_commit(file_path)?;
            Ok(serde_json::to_string_pretty(&diff)?)
        } else {
            // Try to read as a file containing diff content
            std::fs::read_to_string(file_path).map_err(|e| crate::error::KtmeError::Io(e))
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

    pub fn generate_documentation(
        service: &str,
        changes: &str,
        format: Option<&str>,
    ) -> Result<String> {
        tracing::info!(
            "MCP Tool: generate_documentation(service={}, format={:?})",
            service,
            format
        );

        // Auto-initialize service if not present
        Self::ensure_service_initialized(service)?;

        // Parse the changes
        let diff: crate::git::diff::ExtractedDiff =
            serde_json::from_str(changes).map_err(|_| {
                crate::error::KtmeError::InvalidInput("Invalid changes format".to_string())
            })?;

        // Try to use AI for intelligent documentation generation
        match AIClient::new() {
            Ok(ai_client) => {
                tracing::info!("Using AI client for documentation generation");
                Self::generate_ai_documentation(&ai_client, service, &diff, format)
            }
            Err(_) => {
                tracing::warn!("AI client not available, falling back to basic documentation");
                Ok(Self::generate_basic_documentation(service, &diff, format))
            }
        }
    }

    fn ensure_service_initialized(service: &str) -> Result<()> {
        use crate::storage::database::Database;
        use crate::storage::repository::ServiceRepository;

        let db = Database::new(None)?;
        let service_repo = ServiceRepository::new(db);

        // Check if service exists, create if not
        if service_repo.get_by_name(service)?.is_none() {
            tracing::info!("Auto-initializing service: {}", service);
            service_repo.create(
                service,
                None,
                Some(&format!("Auto-initialized via MCP")),
            )?;
        }

        Ok(())
    }

    fn generate_ai_documentation(
        _ai_client: &AIClient,
        service: &str,
        diff: &crate::git::diff::ExtractedDiff,
        format: Option<&str>,
    ) -> Result<String> {
        let _prompt = format!(
            "Generate comprehensive documentation for the service '{}' based on the following code changes:\n\n\
            Commit Message: {}\n\
            Author: {}\n\
            Timestamp: {}\n\n\
            Files Changed:\n{}\n\n\
            Changes Summary:\n\
            - Total files: {}\n\
            - Additions: {}\n\
            - Deletions: {}\n\n\
            Please generate:\n\
            1. A clear overview of what changed\n\
            2. Technical details of the implementation\n\
            3. Impact on existing functionality\n\
            4. Usage examples if applicable\n\
            5. Migration notes if breaking changes\n\n\
            Format the output as {} documentation.",
            service,
            diff.message,
            diff.author,
            diff.timestamp,
            diff.files.iter()
                .map(|f| format!("  - {}: {} (+{}/-{})", f.path, f.status, f.additions, f.deletions))
                .collect::<Vec<_>>()
                .join("\n"),
            diff.summary.total_files,
            diff.summary.total_additions,
            diff.summary.total_deletions,
            format.unwrap_or("markdown")
        );

        // For now, fall back to basic documentation since AI is async
        Ok(Self::generate_basic_documentation(service, diff, format))
    }

    fn generate_basic_documentation(
        service: &str,
        diff: &crate::git::diff::ExtractedDiff,
        format: Option<&str>,
    ) -> String {
        let output_format = format.unwrap_or("markdown");

        match output_format {
            "json" => serde_json::to_string_pretty(&serde_json::json!({
                "service": service,
                "changes": {
                    "source": diff.source,
                    "author": diff.author,
                    "timestamp": diff.timestamp,
                    "message": diff.message,
                    "summary": diff.summary,
                    "files": diff.files
                }
            }))
            .unwrap_or_else(|_| "Error generating JSON".to_string()),
            _ => {
                format!("# Documentation for {}\n\n## Changes\n\n**Source:** {}\n**Author:** {}\n**Timestamp:** {}\n\n### Summary\n- {} files changed\n- {} additions\n- {} deletions\n\n### Files Modified\n{}\n\n### Commit Message\n{}\n\n### Technical Details\n\n> **Note**: This is basic documentation. Configure an AI provider (OPENAI_API_KEY or ANTHROPIC_API_KEY) for intelligent documentation generation.\n",
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
                )
            }
        }
    }

    pub fn update_documentation(service: &str, doc_path: &str, content: &str) -> Result<String> {
        tracing::info!(
            "MCP Tool: update_documentation(service={}, doc_path={})",
            service,
            doc_path
        );

        // For now, just write to the file
        std::fs::write(doc_path, content).map_err(|e| crate::error::KtmeError::Io(e))?;

        Ok(format!("Documentation updated at {}", doc_path))
    }

    /// Search services by query string
    pub fn search_services(query: &str) -> Result<String> {
        tracing::info!("MCP Tool: search_services(query={})", query);

        let storage = StorageManager::new()?;
        let results = storage.search_services(query)?;

        if results.is_empty() {
            return Ok(format!("No services found matching: {}", query));
        }

        let mut output = format!("Search Results for '{}':\n\n", query);
        for (idx, result) in results.iter().enumerate() {
            output.push_str(&format!(
                "{}. **{}** (Relevance: {:.1})\n",
                idx + 1,
                result.name,
                result.relevance_score
            ));

            if let Some(ref desc) = result.description {
                output.push_str(&format!("   Description: {}\n", desc));
            }

            if let Some(ref path) = result.path {
                output.push_str(&format!("   Path: {}\n", path));
            }

            if !result.docs.is_empty() {
                output.push_str("   Documentation:\n");
                for doc in &result.docs {
                    output.push_str(&format!("     - {}\n", doc));
                }
            }

            output.push('\n');
        }

        Ok(output)
    }

    /// Search services by feature
    pub fn search_by_feature(feature: &str) -> Result<String> {
        tracing::info!("MCP Tool: search_by_feature(feature={})", feature);

        let storage = StorageManager::new()?;
        let results = storage.search_by_feature(feature)?;

        if results.is_empty() {
            return Ok(format!("No services found with feature: {}", feature));
        }

        let mut output = format!("Services with feature '{}':\n\n", feature);
        for result in results {
            output.push_str(&format!("**{}**\n", result.name));
            if let Some(ref desc) = result.description {
                output.push_str(&format!("  {}\n", desc));
            }
            output.push('\n');
        }

        Ok(output)
    }

    /// Search services by keyword
    pub fn search_by_keyword(keyword: &str) -> Result<String> {
        tracing::info!("MCP Tool: search_by_keyword(keyword={})", keyword);

        let storage = StorageManager::new()?;
        let results = storage.search_by_keyword(keyword)?;

        if results.is_empty() {
            return Ok(format!("No services found matching keyword: {}", keyword));
        }

        let mut output = format!("Keyword search results for '{}':\n\n", keyword);
        for result in results {
            output.push_str(&format!(
                "• **{}** (Score: {:.1})\n",
                result.name, result.relevance_score
            ));

            if let Some(ref path) = result.path {
                output.push_str(&format!("  Path: {}\n", path));
            }

            output.push_str(&format!("  Documents: {}\n\n", result.docs.len()));
        }

        Ok(output)
    }

    /// Automated workflow: extract → generate → save
    pub fn automated_documentation_workflow(service: &str, source: &str) -> Result<String> {
        tracing::info!(
            "MCP Tool: automated_documentation_workflow(service={}, source={})",
            service,
            source
        );

        // Step 1: Extract changes
        let changes = Self::read_changes(source)?;

        // Step 2: Generate documentation
        let doc_content = Self::generate_documentation(service, &changes, Some("markdown"))?;

        // Step 3: Get service mapping to determine where to save
        let storage = StorageManager::new()?;
        let mapping = storage.get_mapping(service)?;

        // Step 4: Save documentation
        if let Some(primary_doc) = mapping.docs.iter().find(|d| d.r#type == "markdown") {
            std::fs::write(&primary_doc.location, doc_content)
                .map_err(|e| crate::error::KtmeError::Io(e))?;

            Ok(format!("✓ Automated workflow completed!\n  ✓ Extracted changes from {}\n  ✓ Generated documentation for {}\n  ✓ Saved to: {}\n", source, service, primary_doc.location))
        } else {
            // Save to default location
            let default_path = format!("/tmp/{}-documentation.md", service);
            std::fs::write(&default_path, doc_content)
                .map_err(|e| crate::error::KtmeError::Io(e))?;

            Ok(format!("✓ Automated workflow completed!\n  ✓ Extracted changes from {}\n  ✓ Generated documentation for {}\n  ✓ Saved to: {} (no markdown mapping found)\n", source, service, default_path))
        }
    }

    /// Detect service name from current directory with AI fallback
    pub fn detect_service_name() -> Result<String> {
        tracing::info!("MCP Tool: detect_service_name()");

        let detector = ServiceDetector::new()?;

        // Use blocking execution for async function in sync context
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            crate::error::KtmeError::Storage(format!("Failed to create runtime: {}", e))
        })?;

        let service_name = rt.block_on(detector.detect_with_ai_fallback())?;

        let repo_info = detector.get_repository_info();

        let mut result = format!("**Detected Service Name:** {}\n\n", service_name);

        if repo_info.is_git_repository {
            if let Some(ref repo_root) = repo_info.repository_root {
                result.push_str(&format!(
                    "**Git Repository Root:** {}\n",
                    repo_root.display()
                ));
            }
        }

        result.push_str(&format!(
            "**Current Directory:** {}\n",
            repo_info.current_dir.display()
        ));

        Ok(result)
    }

    /// Get repository information
    pub fn get_repository_info() -> Result<String> {
        tracing::info!("MCP Tool: get_repository_info()");

        let detector = ServiceDetector::new()?;
        let repo_info = detector.get_repository_info();

        let mut result = format!("**Repository Information:**\n\n");
        result.push_str(&format!(
            "**Current Directory:** {}\n",
            repo_info.current_dir.display()
        ));

        if repo_info.is_git_repository {
            result.push_str("**Git Repository:** Yes\n");
            if let Some(ref repo_root) = repo_info.repository_root {
                result.push_str(&format!("**Repository Root:** {}\n", repo_root.display()));
            }
        } else {
            result.push_str("**Git Repository:** No\n");
        }

        Ok(result)
    }
}
