use crate::error::Result;
use crate::git::diff::{DiffExtractor, ExtractedDiff};
use std::fs;
use std::path::Path;

pub async fn execute(
    commit: Option<String>,
    staged: bool,
    pr: Option<u32>,
    provider: Option<String>,
    output: Option<String>,
) -> Result<()> {
    tracing::info!("Extracting code changes...");

    let extracted_diff = if let Some(commit_ref) = commit {
        tracing::info!("Extracting from commit: {}", commit_ref);
        let extractor = DiffExtractor::new("commit".to_string(), commit_ref.clone(), None)?;
        extractor.extract()?
    } else if staged {
        tracing::info!("Extracting staged changes");
        let extractor = DiffExtractor::new("staged".to_string(), "staged".to_string(), None)?;
        extractor.extract()?
    } else if let Some(pr_number) = pr {
        let provider_name = provider.unwrap_or_else(|| "github".to_string());
        tracing::info!("Extracting from PR #{} ({})", pr_number, provider_name);

        // TODO: Implement PR extraction via GitHub/GitLab APIs
        // For now, return an error
        return Err(crate::error::KtmeError::UnsupportedProvider(
            format!("PR extraction for {} is not yet implemented", provider_name)
        ));
    } else {
        return Err(crate::error::KtmeError::InvalidInput(
            "No source specified. Use --commit, --staged, or --pr".to_string()
        ));
    };

    // Print summary to console
    print_diff_summary(&extracted_diff);

    // Save to file if requested
    if let Some(output_path) = output {
        save_to_file(&extracted_diff, &output_path)?;
        tracing::info!("Output saved to: {}", output_path);
    }

    Ok(())
}

fn print_diff_summary(diff: &ExtractedDiff) {
    println!("\n📊 Diff Summary");
    println!("Source: {}", diff.source);
    println!("Identifier: {}", diff.identifier);
    println!("Author: {}", diff.author);
    println!("Timestamp: {}", diff.timestamp);
    println!("Message: {}", diff.message);
    println!("Files changed: {}", diff.summary.total_files);
    println!("Additions: +{}", diff.summary.total_additions);
    println!("Deletions: -{}", diff.summary.total_deletions);

    if !diff.files.is_empty() {
        println!("\n📁 Files:");
        for file in &diff.files {
            let icon = match file.status.as_str() {
                "added" => "🆕",
                "modified" => "✏️",
                "deleted" => "🗑️",
                "renamed" => "🔄",
                _ => "📄",
            };
            println!("  {} {} (+{}/-{})", icon, file.path, file.additions, file.deletions);
        }
    }
}

fn save_to_file(diff: &ExtractedDiff, path: &str) -> Result<()> {
    let json_output = serde_json::to_string_pretty(diff)
        .map_err(|e| crate::error::KtmeError::Serialization(e))?;

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| crate::error::KtmeError::Io(e))?;
    }

    fs::write(path, json_output)
        .map_err(|e| crate::error::KtmeError::Io(e))?;

    Ok(())
}
