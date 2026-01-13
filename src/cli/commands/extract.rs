use crate::config::Config;
use crate::error::Result;
use crate::git::diff::{DiffExtractor, ExtractedDiff};
use crate::git::providers::github::GitHubProvider;
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

        match provider_name.to_lowercase().as_str() {
            "github" => {
                // Load config to get token
                let config = Config::load().unwrap_or_default();
                let github = GitHubProvider::from_config(config.git.github_token);

                // Get repository from current directory or error
                let repo = detect_github_repo()?;

                github.fetch_pull_request(&repo, pr_number).await?
            }
            "gitlab" => {
                // Load config to get token
                let config = Config::load().unwrap_or_default();
                let gitlab = crate::git::providers::gitlab::GitLabProvider::from_config(
                    config.git.gitlab_token,
                );

                // Get project from current directory or error
                let project = detect_gitlab_project()?;

                gitlab.fetch_merge_request(&project, pr_number).await?
            }
            _ => {
                return Err(crate::error::KtmeError::UnsupportedProvider(format!(
                    "Unknown provider: {}",
                    provider_name
                )));
            }
        }
    } else {
        return Err(crate::error::KtmeError::InvalidInput(
            "No source specified. Use --commit, --staged, or --pr".to_string(),
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

/// Detect GitHub repository from Git remote
fn detect_github_repo() -> Result<String> {
    use git2::Repository;

    let repo = Repository::open(".").map_err(|e| crate::error::KtmeError::Git(e))?;

    let remote = repo
        .find_remote("origin")
        .map_err(|e| crate::error::KtmeError::Git(e))?;

    let url = remote
        .url()
        .ok_or_else(|| crate::error::KtmeError::InvalidInput("No remote URL found".to_string()))?;

    // Parse GitHub URL to owner/repo format
    // Handles both HTTPS and SSH formats:
    // - https://github.com/owner/repo.git
    // - git@github.com:owner/repo.git
    let repo_path = if url.starts_with("https://github.com/") {
        url.trim_start_matches("https://github.com/")
            .trim_end_matches(".git")
    } else if url.starts_with("git@github.com:") {
        url.trim_start_matches("git@github.com:")
            .trim_end_matches(".git")
    } else {
        return Err(crate::error::KtmeError::InvalidInput(format!(
            "Not a GitHub repository: {}",
            url
        )));
    };

    Ok(repo_path.to_string())
}

/// Detect GitLab project from Git remote
fn detect_gitlab_project() -> Result<String> {
    use git2::Repository;

    let repo = Repository::open(".").map_err(|e| crate::error::KtmeError::Git(e))?;

    let remote = repo
        .find_remote("origin")
        .map_err(|e| crate::error::KtmeError::Git(e))?;

    let url = remote
        .url()
        .ok_or_else(|| crate::error::KtmeError::InvalidInput("No remote URL found".to_string()))?;

    // Parse GitLab URL to namespace/project format
    // Handles both HTTPS and SSH formats:
    // - https://gitlab.com/namespace/project.git
    // - git@gitlab.com:namespace/project.git
    // Also supports self-hosted GitLab instances

    let project_path = if url.contains("gitlab.com") {
        if url.starts_with("https://gitlab.com/") {
            url.trim_start_matches("https://gitlab.com/")
                .trim_end_matches(".git")
        } else if url.starts_with("git@gitlab.com:") {
            url.trim_start_matches("git@gitlab.com:")
                .trim_end_matches(".git")
        } else {
            return Err(crate::error::KtmeError::InvalidInput(format!(
                "Unsupported GitLab URL format: {}",
                url
            )));
        }
    } else if url.contains("://") {
        // HTTPS format for self-hosted GitLab
        url.split("://")
            .nth(1)
            .and_then(|s| s.split_once('/'))
            .map(|(_, path)| path.trim_end_matches(".git"))
            .ok_or_else(|| {
                crate::error::KtmeError::InvalidInput(format!(
                    "Could not parse GitLab project path from: {}",
                    url
                ))
            })?
    } else if url.starts_with("git@") {
        // SSH format for self-hosted GitLab
        url.split(':')
            .nth(1)
            .map(|path| path.trim_end_matches(".git"))
            .ok_or_else(|| {
                crate::error::KtmeError::InvalidInput(format!(
                    "Could not parse GitLab project path from: {}",
                    url
                ))
            })?
    } else {
        return Err(crate::error::KtmeError::InvalidInput(format!(
            "Not a GitLab repository: {}",
            url
        )));
    };

    Ok(project_path.to_string())
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
            println!(
                "  {} {} (+{}/-{})",
                icon, file.path, file.additions, file.deletions
            );
        }
    }
}

fn save_to_file(diff: &ExtractedDiff, path: &str) -> Result<()> {
    let json_output = serde_json::to_string_pretty(diff)
        .map_err(|e| crate::error::KtmeError::Serialization(e))?;

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| crate::error::KtmeError::Io(e))?;
    }

    fs::write(path, json_output).map_err(|e| crate::error::KtmeError::Io(e))?;

    Ok(())
}
