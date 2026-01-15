use crate::ai::{prompts::PromptTemplates, AIClient};
use crate::error::{KtmeError, Result};
use crate::git::diff::{DiffExtractor, ExtractedDiff};
use crate::storage::database::Database;
use crate::storage::repository::{FeatureRepository, ServiceRepository};
use crate::storage::models::FeatureType;
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    commit: Option<String>,
    input: Option<String>,
    pr: Option<u32>,
    staged: bool,
    service: String,
    doc_type: Option<String>,
    format: Option<String>,
    output: Option<String>,
    template: Option<String>,
) -> Result<()> {
    tracing::info!("Generating documentation for service: {}", service);

    // Auto-initialize if not already done
    check_and_initialize(&service).await?;

    // Get the diff data
    let diff = if let Some(input_file) = input {
        tracing::info!("Using input file: {}", input_file);
        load_diff_from_file(&input_file)?
    } else if let Some(commit_ref) = commit {
        tracing::info!("Using commit: {}", commit_ref);
        let extractor = DiffExtractor::new("commit".to_string(), commit_ref, None)?;
        extractor.extract()?
    } else if staged {
        tracing::info!("Using staged changes");
        let extractor = DiffExtractor::new("staged".to_string(), "staged".to_string(), None)?;
        extractor.extract()?
    } else if let Some(pr_number) = pr {
        tracing::info!("Using PR: #{}", pr_number);
        return Err(crate::error::KtmeError::UnsupportedProvider(
            "PR-based documentation generation is not yet implemented".to_string(),
        ));
    } else {
        return Err(crate::error::KtmeError::InvalidInput(
            "No source specified. Use --commit, --input, --staged, or --pr".to_string(),
        ));
    };

    // Initialize AI client
    let ai_client = AIClient::new()?;
    tracing::info!("Using AI provider: {}", ai_client.provider_name());

    // Determine documentation type
    let doc_type = doc_type.as_deref().unwrap_or("general");

    // Generate prompt
    let prompt = if let Some(template_file) = template {
        load_custom_template(&template_file, &diff)?
    } else {
        PromptTemplates::generate_documentation_prompt(&diff, doc_type, None)?
    };

    tracing::info!("Generating documentation using {}...", doc_type);

    // Generate documentation
    let documentation = ai_client.generate_documentation(&prompt).await?;

    // Output the documentation
    match format.as_deref() {
        Some("markdown") | Some("md") => {
            let content = format_documentation(&documentation, doc_type, &service);
            write_output(&content, output.as_deref())?;
        }
        Some("json") => {
            let json_output = serde_json::json!({
                "service": service,
                "doc_type": doc_type,
                "source": diff.identifier,
                "documentation": documentation,
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "provider": ai_client.provider_name()
            });
            write_json_output(&json_output, output.as_deref())?;
        }
        _ => {
            // Default to plain text/markdown
            let content = format_documentation(&documentation, doc_type, &service);
            write_output(&content, output.as_deref())?;
        }
    }

    tracing::info!("Documentation generated successfully!");
    
    // Update knowledge graph with generated documentation
    update_knowledge_graph(&service, &diff, &documentation, doc_type).await?;
    
    Ok(())
}

fn load_diff_from_file(file_path: &str) -> Result<ExtractedDiff> {
    let content = fs::read_to_string(file_path).map_err(|e| crate::error::KtmeError::Io(e))?;

    serde_json::from_str(&content).map_err(|e| crate::error::KtmeError::Serialization(e))
}

fn load_custom_template(template_file: &str, diff: &ExtractedDiff) -> Result<String> {
    let template_content =
        fs::read_to_string(template_file).map_err(|e| crate::error::KtmeError::Io(e))?;

    // Simple template substitution
    let mut prompt = template_content;
    prompt = prompt.replace("{{SERVICE}}", &diff.source);
    prompt = prompt.replace("{{AUTHOR}}", &diff.author);
    prompt = prompt.replace("{{MESSAGE}}", &diff.message);
    prompt = prompt.replace("{{TIMESTAMP}}", &diff.timestamp);
    prompt = prompt.replace("{{FILES_CHANGED}}", &diff.summary.total_files.to_string());
    prompt = prompt.replace("{{ADDITIONS}}", &diff.summary.total_additions.to_string());
    prompt = prompt.replace("{{DELETIONS}}", &diff.summary.total_deletions.to_string());

    // Add diff content at the end
    prompt.push_str(&format!(
        "\n\nChanges:\n{}",
        PromptTemplates::format_diff_content(diff)
    ));

    Ok(prompt)
}

fn format_documentation(content: &str, doc_type: &str, service: &str) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");

    format!(
        "# Documentation for {}\n\n**Type**: {}\n**Generated**: {}\n\n---\n\n{}",
        service, doc_type, timestamp, content
    )
}

fn write_output(content: &str, output: Option<&str>) -> Result<()> {
    match output {
        Some(path) => {
            // Create parent directories if they don't exist
            if let Some(parent) = Path::new(path).parent() {
                fs::create_dir_all(parent).map_err(|e| crate::error::KtmeError::Io(e))?;
            }

            fs::write(path, content).map_err(|e| crate::error::KtmeError::Io(e))?;

            println!("Documentation saved to: {}", path);
        }
        None => {
            println!("{}", content);
        }
    }
    Ok(())
}

fn write_json_output(json: &serde_json::Value, output: Option<&str>) -> Result<()> {
    let json_content = serde_json::to_string_pretty(json)
        .map_err(|e| crate::error::KtmeError::Serialization(e))?;

    match output {
        Some(path) => {
            if let Some(parent) = Path::new(path).parent() {
                fs::create_dir_all(parent).map_err(|e| crate::error::KtmeError::Io(e))?;
            }

            fs::write(path, json_content).map_err(|e| crate::error::KtmeError::Io(e))?;

            println!("JSON documentation saved to: {}", path);
        }
        None => {
            println!("{}", json_content);
        }
    }
    Ok(())
}

/// Check if service is initialized and auto-initialize if needed
async fn check_and_initialize(service: &str) -> Result<()> {
    let db = Database::new(None)?;
    let service_repo = ServiceRepository::new(db);

    // Check if service exists
    if service_repo.get_by_name(service)?.is_none() {
        tracing::info!("Service '{}' not found in knowledge graph, auto-initializing...", service);
        
        // Create service entry
        service_repo.create(
            service,
            None, // No path specified
            Some(&format!("Auto-initialized for documentation generation")),
        )?;
        
        println!("ℹ️  Initialized knowledge graph for service '{}'", service);
    }

    Ok(())
}

/// Update knowledge graph with generated documentation
async fn update_knowledge_graph(
    service: &str,
    diff: &ExtractedDiff,
    documentation: &str,
    doc_type: &str,
) -> Result<()> {
    let db = Database::new(None)?;
    let service_repo = ServiceRepository::new(db.clone());
    let feature_repo = FeatureRepository::new(db);

    // Get service ID
    let service_entry = service_repo.get_by_name(service)?
        .ok_or_else(|| KtmeError::Storage(format!("Service '{}' not found", service)))?;

    // Extract features from the diff and create feature entries
    for file in &diff.files {
        // Determine feature type based on file path
        let feature_type = determine_feature_type(&file.path);
        
        // Create a feature entry for significant changes
        if file.additions > 5 || file.deletions > 5 {
            let feature_name = extract_feature_name(&file.path, &diff.message);
            let feature_id = Uuid::new_v4().to_string();
            
            // Create feature
            let tags = vec![
                doc_type.to_string(),
                file.status.clone(),
            ];
            
            let metadata = serde_json::json!({
                "file_path": file.path,
                "additions": file.additions,
                "deletions": file.deletions,
                "commit": diff.identifier,
                "author": diff.author,
                "timestamp": diff.timestamp,
            });
            
            // Try to create feature (ignore if already exists)
            match feature_repo.create(
                &feature_id,
                service_entry.id,
                &feature_name,
                Some(&format!("Documentation: {}", documentation.chars().take(200).collect::<String>())),
                feature_type,
                tags,
                metadata,
            ) {
                Ok(_) => {
                    tracing::info!("Created feature '{}' in knowledge graph", feature_name);
                }
                Err(e) => {
                    tracing::debug!("Feature creation skipped: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// Determine feature type from file path
fn determine_feature_type(path: &str) -> FeatureType {
    let path_lower = path.to_lowercase();
    
    if path_lower.contains("api") || path_lower.contains("endpoint") || path_lower.contains("route") {
        FeatureType::Api
    } else if path_lower.contains("ui") || path_lower.contains("component") || path_lower.contains("view") {
        FeatureType::Ui
    } else if path_lower.contains("test") {
        FeatureType::Testing
    } else if path_lower.contains("config") || path_lower.ends_with(".toml") || path_lower.ends_with(".yaml") || path_lower.ends_with(".yml") {
        FeatureType::Config
    } else if path_lower.contains("db") || path_lower.contains("database") || path_lower.contains("migration") {
        FeatureType::Database
    } else if path_lower.contains("security") || path_lower.contains("auth") {
        FeatureType::Security
    } else if path_lower.contains("deploy") || path_lower.contains("docker") || path_lower.contains("ci") {
        FeatureType::Deployment
    } else if path_lower.contains("performance") || path_lower.contains("optimize") {
        FeatureType::Performance
    } else {
        FeatureType::BusinessLogic
    }
}

/// Extract a meaningful feature name from file path and commit message
fn extract_feature_name(path: &str, commit_message: &str) -> String {
    // Try to extract from commit message first
    let message_words: Vec<&str> = commit_message.split_whitespace().collect();
    if message_words.len() > 2 {
        let feature_from_message = message_words.iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        if feature_from_message.len() > 10 {
            return feature_from_message;
        }
    }
    
    // Fallback to file path
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.replace('_', " ").replace('-', " "))
        .unwrap_or_else(|| "unknown feature".to_string())
}

