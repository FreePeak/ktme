use crate::error::{KtmeError, Result};
use crate::git::diff::{DiffSummary, ExtractedDiff, FileChange};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubPullRequest {
    number: u32,
    title: String,
    body: Option<String>,
    state: String,
    head: GitHubRef,
    base: GitHubRef,
    created_at: String,
    user: GitHubUser,
}

#[derive(Debug, Deserialize)]
struct GitHubFile {
    filename: String,
    status: String,
    additions: u32,
    deletions: u32,
    changes: u32,
    patch: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRef {
    #[serde(rename = "ref")]
    ref_name: String,
    sha: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GitHubPullRequestFile {
    filename: String,
    status: String,
    additions: u32,
    deletions: u32,
    patch: Option<String>,
}

pub struct GitHubProvider {
    api_token: Option<String>,
    client: reqwest::Client,
}

impl GitHubProvider {
    pub fn new(api_token: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("ktme-cli")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { api_token, client }
    }

    /// Create a new provider using config or environment variable
    pub fn from_config(config_token: Option<String>) -> Self {
        // Priority: config token > env var > None
        let token = config_token
            .or_else(|| env::var("GITHUB_TOKEN").ok())
            .or_else(|| env::var("GH_TOKEN").ok());

        if token.is_some() {
            tracing::debug!("Using GitHub token from config or environment");
        } else {
            tracing::warn!("No GitHub token configured, API rate limits will apply");
        }

        Self::new(token)
    }

    pub async fn fetch_pull_request(&self, repo: &str, pr_number: u32) -> Result<ExtractedDiff> {
        tracing::info!("Fetching GitHub PR #{} from {}", pr_number, repo);

        // Parse repo (owner/repo format)
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            return Err(KtmeError::InvalidInput(format!(
                "Invalid repository format '{}'. Expected 'owner/repo'",
                repo
            )));
        }
        let (owner, repo_name) = (parts[0], parts[1]);

        // Fetch PR metadata
        let pr_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo_name, pr_number
        );

        let pr: GitHubPullRequest = self.fetch_json(&pr_url).await?;

        // Fetch PR files/diff
        let files_url = format!("{}/files", pr_url);
        let files: Vec<GitHubPullRequestFile> = self.fetch_json(&files_url).await?;

        // Convert to ExtractedDiff format
        let file_changes: Vec<FileChange> = files
            .into_iter()
            .map(|f| FileChange {
                path: f.filename,
                status: Self::normalize_status(&f.status),
                additions: f.additions,
                deletions: f.deletions,
                diff: f.patch.unwrap_or_default(),
            })
            .collect();

        let summary = DiffSummary {
            total_files: file_changes.len() as u32,
            total_additions: file_changes.iter().map(|f| f.additions).sum(),
            total_deletions: file_changes.iter().map(|f| f.deletions).sum(),
        };

        Ok(ExtractedDiff {
            source: format!("github-pr-{}", repo),
            identifier: format!("#{}", pr_number),
            timestamp: pr.created_at,
            author: pr.user.login,
            message: format!("{}\n\n{}", pr.title, pr.body.unwrap_or_default()),
            files: file_changes,
            summary,
        })
    }

    /// Fetch JSON from GitHub API with authentication
    async fn fetch_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        let mut request = self.client.get(url);

        // Add authentication if token is available
        if let Some(token) = &self.api_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        // Add required GitHub API headers
        request = request
            .header("Accept", "application/vnd.github.v3+json")
            .header("X-GitHub-Api-Version", "2022-11-28");

        let response = request.send().await.map_err(|e| {
            KtmeError::NetworkError(format!("Failed to fetch from GitHub API: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(KtmeError::ApiError(format!(
                "GitHub API request failed with status {}: {}",
                status, error_body
            )));
        }

        response.json().await.map_err(|e| {
            KtmeError::DeserializationError(format!("Failed to parse GitHub API response: {}", e))
        })
    }

    /// Normalize GitHub file status to our standard format
    fn normalize_status(status: &str) -> String {
        match status {
            "added" => "added",
            "removed" => "deleted",
            "modified" => "modified",
            "renamed" => "renamed",
            _ => "modified",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = GitHubProvider::new(Some("test-token".to_string()));
        assert!(provider.api_token.is_some());
    }

    #[test]
    fn test_provider_creation_without_token() {
        let provider = GitHubProvider::new(None);
        assert!(provider.api_token.is_none());
    }

    #[test]
    fn test_normalize_status() {
        assert_eq!(GitHubProvider::normalize_status("added"), "added");
        assert_eq!(GitHubProvider::normalize_status("removed"), "deleted");
        assert_eq!(GitHubProvider::normalize_status("modified"), "modified");
        assert_eq!(GitHubProvider::normalize_status("renamed"), "renamed");
        assert_eq!(GitHubProvider::normalize_status("unknown"), "modified");
    }

    #[tokio::test]
    async fn test_invalid_repo_format() {
        let provider = GitHubProvider::new(None);
        let result = provider.fetch_pull_request("invalid-format", 1).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(KtmeError::InvalidInput(_))));
    }
}
