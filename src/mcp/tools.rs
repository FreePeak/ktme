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
                // Create async runtime for AI call
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    crate::error::KtmeError::Storage(format!("Failed to create runtime: {}", e))
                })?;

                rt.block_on(Self::generate_ai_documentation_async(
                    &ai_client, service, &diff, format,
                ))
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
            service_repo.create(service, None, Some(&format!("Auto-initialized via MCP")))?;
        }

        Ok(())
    }

    async fn generate_ai_documentation_async(
        ai_client: &AIClient,
        service: &str,
        diff: &crate::git::diff::ExtractedDiff,
        format: Option<&str>,
    ) -> Result<String> {
        let prompt = format!(
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

        match ai_client.generate_documentation(&prompt).await {
            Ok(documentation) => Ok(documentation),
            Err(e) => {
                tracing::warn!(
                    "AI generation failed: {}, falling back to basic documentation",
                    e
                );
                Ok(Self::generate_basic_documentation(service, diff, format))
            }
        }
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

    /// Scan documentation and return statistics
    pub fn scan_documentation(path: Option<&str>) -> Result<String> {
        tracing::info!("MCP Tool: scan_documentation(path={:?})", path);

        let project_path = path.unwrap_or(".");
        let project_dir = std::path::PathBuf::from(project_path);
        let docs_dir = project_dir.join("docs");

        if !docs_dir.exists() {
            return Ok(format!("No documentation directory found at: {}", docs_dir.display()));
        }

        let mut total_files = 0;
        let mut total_sections = 0;
        let mut total_code_blocks = 0;
        let mut file_details = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&docs_dir) {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                    total_files += 1;
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let sections = content.lines().filter(|l| l.starts_with("## ")).count();
                        let code_blocks = content.matches("```").count() / 2;
                        total_sections += sections;
                        total_code_blocks += code_blocks;
                        file_details.push(format!(
                            "- **{}**: {} sections, {} code blocks",
                            path.file_name().unwrap().to_string_lossy(),
                            sections,
                            code_blocks
                        ));
                    }
                }
            }
        }

        let mut result = format!("# Documentation Scan Report\n\n**Path:** {}\n\n## Summary\n", project_path);
        result.push_str(&format!("- Total markdown files: {}\n", total_files));
        result.push_str(&format!("- Total sections: {}\n", total_sections));
        result.push_str(&format!("- Total code blocks: {}\n\n", total_code_blocks));
        result.push_str("## Files\n");
        for detail in file_details {
            result.push_str(&format!("{}\n", detail));
        }

        Ok(result)
    }

    /// Validate documentation for common issues
    pub fn validate_documentation(path: Option<&str>) -> Result<String> {
        tracing::info!("MCP Tool: validate_documentation(path={:?})", path);

        let project_path = path.unwrap_or(".");
        let project_dir = std::path::PathBuf::from(project_path);
        let docs_dir = project_dir.join("docs");

        if !docs_dir.exists() {
            return Ok(format!("No documentation directory found at: {}", docs_dir.display()));
        }

        let mut validation_warnings = Vec::new();
        let mut validation_errors = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&docs_dir) {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let filename = path.file_name().unwrap().to_string_lossy();

                        if !content.starts_with("# ") {
                            validation_warnings.push(format!("{}: Missing title header", filename));
                        }

                        if !content.contains("## ") {
                            validation_warnings.push(format!("{}: No sections found", filename));
                        }

                        let open_brackets = content.matches("[").count();
                        let close_brackets = content.matches("]").count();
                        if open_brackets != close_brackets {
                            validation_errors.push(format!("{}: Potentially broken links detected", filename));
                        }
                    }
                }
            }
        }

        let mut result = format!("# Documentation Validation Report\n\n**Path:** {}\n\n", project_path);

        if validation_warnings.is_empty() && validation_errors.is_empty() {
            result.push_str("## Result\nAll checks passed!\n");
        } else {
            if !validation_warnings.is_empty() {
                result.push_str("## Warnings\n");
                for warning in &validation_warnings {
                    result.push_str(&format!("- {}\n", warning));
                }
            }
            if !validation_errors.is_empty() {
                result.push_str("\n## Errors\n");
                for error in &validation_errors {
                    result.push_str(&format!("- {}\n", error));
                }
            }
        }

        Ok(result)
    }

    /// Detect technology stack for a project
    pub fn detect_tech_stack(path: Option<&str>) -> Result<String> {
        tracing::info!("MCP Tool: detect_tech_stack(path={:?})", path);

        let project_path = path.unwrap_or(".");
        let project_dir = std::path::PathBuf::from(project_path);

        let mut result = format!("# Technology Stack Report\n\n**Path:** {}\n\n", project_path);
        result.push_str("## Detected Technologies\n\n");

        let cargo_toml = project_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            result.push_str("### Rust Project\n");
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("tokio") {
                    result.push_str("- Async runtime: **tokio**\n");
                }
                if content.contains("serde") {
                    result.push_str("- Serialization: **serde**\n");
                }
                if content.contains("reqwest") {
                    result.push_str("- HTTP client: **reqwest**\n");
                }
                if content.contains("tracing") {
                    result.push_str("- Logging: **tracing**\n");
                }
                if content.contains("clap") {
                    result.push_str("- CLI parsing: **clap**\n");
                }
                if content.contains("rusqlite") {
                    result.push_str("- Database: **rusqlite**\n");
                }
            }
            result.push_str("\n");
        }

        let package_json = project_dir.join("package.json");
        if package_json.exists() {
            result.push_str("### Node.js Project\n");
            result.push_str("- Package manager detected\n\n");
        }

        let go_mod = project_dir.join("go.mod");
        if go_mod.exists() {
            result.push_str("### Go Project\n");
            result.push_str("- Go modules detected\n\n");
        }

        let pom_xml = project_dir.join("pom.xml");
        if pom_xml.exists() {
            result.push_str("### Java Project\n");
            result.push_str("- Maven detected\n\n");
        }

        Ok(result)
    }

    /// Find TODO markers in documentation
    pub fn find_documentation_todos(path: Option<&str>) -> Result<String> {
        tracing::info!("MCP Tool: find_documentation_todos(path={:?})", path);

        let project_path = path.unwrap_or(".");
        let project_dir = std::path::PathBuf::from(project_path);
        let docs_dir = project_dir.join("docs");

        if !docs_dir.exists() {
            return Ok(format!("No documentation directory found at: {}", docs_dir.display()));
        }

        let mut todos = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&docs_dir) {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        for (line_num, line) in content.lines().enumerate() {
                            if line.contains("TODO:") {
                                todos.push(format!(
                                    "- **{}** (line {}): {}",
                                    path.file_name().unwrap().to_string_lossy(),
                                    line_num + 1,
                                    line.trim()
                                ));
                            }
                        }
                    }
                }
            }
        }

        let mut result = format!("# Documentation TODOs\n\n**Path:** {}\n\n", project_path);

        if todos.is_empty() {
            result.push_str("No TODO markers found in documentation.\n");
        } else {
            result.push_str(&format!("Found {} TODO marker(s):\n\n", todos.len()));
            for todo in todos {
                result.push_str(&format!("{}\n", todo));
            }
        }

        Ok(result)
    }
}
