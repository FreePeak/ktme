use crate::ai::{prompts::PromptTemplates, AIClient};
use crate::config::Config;
use crate::doc::writers::confluence::ConfluenceWriter;
use crate::error::Result;
use crate::git::diff::DiffExtractor;
use crate::storage::mapping::StorageManager;
use std::fs;

pub async fn execute(
    commit: Option<String>,
    pr: Option<u32>,
    staged: bool,
    service: String,
    section: Option<String>,
    dry_run: bool,
) -> Result<()> {
    tracing::info!("Updating documentation for service: {}", service);

    // Get service mapping
    let storage = StorageManager::new()?;
    let mapping = storage.get_mapping(&service)?;

    if mapping.docs.is_empty() {
        return Err(crate::error::KtmeError::DocumentNotFound(format!(
            "No documentation locations mapped for service: {}",
            service
        )));
    }

    // Extract changes
    let diff = if let Some(commit_ref) = commit {
        tracing::info!("Using commit: {}", commit_ref);
        let extractor = DiffExtractor::new("commit".to_string(), commit_ref, None)?;
        extractor.extract()?
    } else if let Some(pr_number) = pr {
        tracing::info!("Using PR: #{}", pr_number);
        return Err(crate::error::KtmeError::UnsupportedProvider(
            "PR-based updates are not yet implemented".to_string(),
        ));
    } else if staged {
        tracing::info!("Using staged changes");
        let extractor = DiffExtractor::new("staged".to_string(), "staged".to_string(), None)?;
        extractor.extract()?
    } else {
        return Err(crate::error::KtmeError::InvalidInput(
            "No source specified. Use --commit, --pr, or --staged".to_string(),
        ));
    };

    if dry_run {
        println!("Dry run mode - would update the following locations:");
        for doc in &mapping.docs {
            println!("  - {} ({})", doc.location, doc.r#type);
        }
        println!("Changes to apply:");
        println!("  Source: {}", diff.identifier);
        println!("  Files: {}", diff.summary.total_files);
        return Ok(());
    }

    // Generate update content
    let ai_client = AIClient::new()?;
    let prompt = PromptTemplates::update_documentation_prompt(&diff, section.as_deref())?;

    tracing::info!("Generating update content...");
    let update_content = ai_client.generate_documentation(&prompt).await?;

    // Apply updates to each documentation location
    for doc_location in &mapping.docs {
        match doc_location.r#type.as_str() {
            "markdown" => {
                update_markdown_file(&doc_location.location, &update_content, section.as_deref())?;
                println!("✓ Updated markdown file: {}", doc_location.location);
            }
            "confluence" => {
                update_confluence_page(&doc_location.location, &update_content).await?;
                println!("✓ Updated Confluence page: {}", doc_location.location);
            }
            _ => {
                println!("⚠ Unknown documentation type: {}", doc_location.r#type);
            }
        }
    }

    println!("Documentation updated successfully!");
    Ok(())
}

fn update_markdown_file(file_path: &str, content: &str, section: Option<&str>) -> Result<()> {
    let existing_content =
        fs::read_to_string(file_path).map_err(|e| crate::error::KtmeError::Io(e))?;

    let updated_content = if let Some(section_name) = section {
        // Find and update specific section
        update_markdown_section(&existing_content, section_name, content)
    } else {
        // Append to end of file
        format!(
            "{}\n\n---\n\n## Update {}\n\n{}",
            existing_content,
            chrono::Utc::now().format("%Y-%m-%d"),
            content
        )
    };

    fs::write(file_path, updated_content).map_err(|e| crate::error::KtmeError::Io(e))?;

    Ok(())
}

fn update_markdown_section(content: &str, section_name: &str, new_content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut in_section = false;
    let mut section_found = false;

    for line in lines {
        if line.starts_with("#") && line.to_lowercase().contains(&section_name.to_lowercase()) {
            in_section = true;
            section_found = true;
            result.push(line.to_string());
            result.push("".to_string());
            result.extend(new_content.lines().map(|l| l.to_string()));
        } else if in_section && line.starts_with("#") {
            in_section = false;
            result.push(line.to_string());
        } else if !in_section {
            result.push(line.to_string());
        }
    }

    if !section_found {
        // Section not found, append to end
        result.push("".to_string());
        result.push(format!("## {}", section_name));
        result.push("".to_string());
        result.extend(new_content.lines().map(|l| l.to_string()));
    }

    result.join("\n")
}

async fn update_confluence_page(location: &str, content: &str) -> Result<()> {
    tracing::info!("Updating Confluence page at: {}", location);

    // Load Confluence configuration from config file
    let config = Config::load()?;
    let confluence_config = config.confluence;

    // Validate required configuration fields
    let base_url = confluence_config.base_url.ok_or_else(|| {
        crate::error::KtmeError::Config(
            "Confluence base_url not configured. Please set [confluence] base_url in config.toml"
                .to_string(),
        )
    })?;

    let api_token = confluence_config.api_token.ok_or_else(|| {
        crate::error::KtmeError::Config(
            "Confluence api_token not configured. Please set [confluence] api_token in config.toml"
                .to_string(),
        )
    })?;

    let space_key = confluence_config.space_key.ok_or_else(|| {
        crate::error::KtmeError::Config(
            "Confluence space_key not configured. Please set [confluence] space_key in config.toml"
                .to_string(),
        )
    })?;

    // Parse page ID from location (expecting URL like https://confluence.example.com/pages/viewpage.action?pageId=123456)
    let page_id = extract_confluence_page_id(location)?;

    // Create Confluence writer
    let writer = ConfluenceWriter::new(base_url, api_token, space_key);

    // Update the page
    writer.update_page(&page_id, content).await?;

    Ok(())
}

fn extract_confluence_page_id(url: &str) -> Result<String> {
    // Try to extract page ID from URL patterns:
    // 1. https://confluence.example.com/pages/viewpage.action?pageId=123456
    // 2. https://confluence.example.com/display/SPACE/Page+Title (would need API call)
    // 3. Just the page ID itself: "123456"

    // Check if it's already just a page ID (all digits)
    if url.chars().all(|c| c.is_ascii_digit()) {
        return Ok(url.to_string());
    }

    // Try to extract from URL
    if let Some(page_id_pos) = url.find("pageId=") {
        let start = page_id_pos + 7; // Length of "pageId="
        let page_id = url[start..].split('&').next().unwrap_or("").to_string();
        if !page_id.is_empty() {
            return Ok(page_id);
        }
    }

    // If we can't extract it, return an error with helpful message
    Err(crate::error::KtmeError::Config(format!(
        "Could not extract Confluence page ID from location: {}. \
        Please use either a full Confluence URL with pageId parameter, \
        or just the page ID number.",
        url
    )))
}
