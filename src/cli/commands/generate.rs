use crate::error::Result;
use crate::ai::{AIClient, prompts::PromptTemplates};
use crate::git::diff::{DiffExtractor, ExtractedDiff};
use std::fs;
use std::path::Path;

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
            "PR-based documentation generation is not yet implemented".to_string()
        ));
    } else {
        return Err(crate::error::KtmeError::InvalidInput(
            "No source specified. Use --commit, --input, --staged, or --pr".to_string()
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
    Ok(())
}

fn load_diff_from_file(file_path: &str) -> Result<ExtractedDiff> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| crate::error::KtmeError::Io(e))?;

    serde_json::from_str(&content)
        .map_err(|e| crate::error::KtmeError::Serialization(e))
}

fn load_custom_template(template_file: &str, diff: &ExtractedDiff) -> Result<String> {
    let template_content = fs::read_to_string(template_file)
        .map_err(|e| crate::error::KtmeError::Io(e))?;

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
    prompt.push_str(&format!("\n\nChanges:\n{}", PromptTemplates::format_diff_content(diff)));

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
                fs::create_dir_all(parent)
                    .map_err(|e| crate::error::KtmeError::Io(e))?;
            }

            fs::write(path, content)
                .map_err(|e| crate::error::KtmeError::Io(e))?;

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
                fs::create_dir_all(parent)
                    .map_err(|e| crate::error::KtmeError::Io(e))?;
            }

            fs::write(path, json_content)
                .map_err(|e| crate::error::KtmeError::Io(e))?;

            println!("JSON documentation saved to: {}", path);
        }
        None => {
            println!("{}", json_content);
        }
    }
    Ok(())
}
