use crate::error::{KtmeError, Result};
use crate::git::diff::{DiffSummary, ExtractedDiff, FileChange};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitLabMergeRequest {
    iid: u32,
    title: String,
    description: Option<String>,
    state: String,
    source_branch: String,
    target_branch: String,
    sha: String,
    created_at: String,
    author: GitLabUser,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitLabMergeRequestChanges {
    changes: Vec<GitLabMergeRequestChange>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitLabMergeRequestChange {
    old_path: String,
    new_path: String,
    new_file: bool,
    renamed_file: bool,
    deleted_file: bool,
    diff: String,
}

#[derive(Debug, Deserialize)]
struct GitLabUser {
    username: String,
}

pub struct GitLabProvider {
    api_token: Option<String>,
    base_url: String,
    client: reqwest::Client,
}

impl GitLabProvider {
    pub fn new(api_token: Option<String>) -> Self {
        Self::new_with_url(api_token, "https://gitlab.com".to_string())
    }

    pub fn new_with_url(api_token: Option<String>, base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("ktme-cli")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            api_token,
            base_url,
            client,
        }
    }

    /// Create a new provider using config or environment variable
    pub fn from_config(config_token: Option<String>) -> Self {
        // Priority: config token > env var > None
        let token = config_token
            .or_else(|| env::var("GITLAB_TOKEN").ok())
            .or_else(|| env::var("GL_TOKEN").ok());

        // Support custom GitLab instance URL
        let base_url = env::var("GITLAB_URL").unwrap_or_else(|_| "https://gitlab.com".to_string());

        if token.is_some() {
            tracing::debug!("Using GitLab token from config or environment");
        } else {
            tracing::warn!("No GitLab token configured, API rate limits will apply");
        }

        Self::new_with_url(token, base_url)
    }

    pub async fn fetch_merge_request(
        &self,
        project: &str,
        mr_number: u32,
    ) -> Result<ExtractedDiff> {
        tracing::info!("Fetching GitLab MR #{} from {}", mr_number, project);

        // URL-encode the project path
        let encoded_project = urlencoding::encode(project);

        // Fetch MR metadata
        let mr_url = format!(
            "{}/api/v4/projects/{}/merge_requests/{}",
            self.base_url, encoded_project, mr_number
        );

        let mr: GitLabMergeRequest = self.fetch_json(&mr_url).await?;

        // Fetch MR changes
        let changes_url = format!("{}/changes", mr_url);
        let changes_response: GitLabMergeRequestChanges = self.fetch_json(&changes_url).await?;

        // Convert to FileChange format
        let file_changes: Vec<FileChange> = changes_response
            .changes
            .into_iter()
            .map(|c| {
                let (additions, deletions) = Self::count_diff_lines(&c.diff);
                let status = Self::determine_status(&c);

                FileChange {
                    path: c.new_path.clone(),
                    status,
                    additions,
                    deletions,
                    diff: c.diff,
                }
            })
            .collect();

        let summary = DiffSummary {
            total_files: file_changes.len() as u32,
            total_additions: file_changes.iter().map(|f| f.additions).sum(),
            total_deletions: file_changes.iter().map(|f| f.deletions).sum(),
        };

        Ok(ExtractedDiff {
            source: format!("gitlab-mr-{}", project),
            identifier: format!("!{}", mr_number),
            timestamp: mr.created_at,
            author: mr.author.username,
            message: format!("{}\n\n{}", mr.title, mr.description.unwrap_or_default()),
            files: file_changes,
            summary,
        })
    }

    /// Fetch JSON from GitLab API with authentication
    async fn fetch_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        let mut request = self.client.get(url);

        // Add authentication if token is available
        if let Some(token) = &self.api_token {
            request = request.header("PRIVATE-TOKEN", token);
        }

        let response = request.send().await.map_err(|e| {
            KtmeError::NetworkError(format!("Failed to fetch from GitLab API: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(KtmeError::ApiError(format!(
                "GitLab API request failed with status {}: {}",
                status, error_body
            )));
        }

        response.json().await.map_err(|e| {
            KtmeError::DeserializationError(format!("Failed to parse GitLab API response: {}", e))
        })
    }

    /// Determine file status from GitLab change
    fn determine_status(change: &GitLabMergeRequestChange) -> String {
        if change.new_file {
            "added".to_string()
        } else if change.deleted_file {
            "deleted".to_string()
        } else if change.renamed_file {
            "renamed".to_string()
        } else {
            "modified".to_string()
        }
    }

    /// Count additions and deletions from a diff string
    fn count_diff_lines(diff: &str) -> (u32, u32) {
        let mut additions = 0;
        let mut deletions = 0;

        for line in diff.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                additions += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                deletions += 1;
            }
        }

        (additions, deletions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = GitLabProvider::new(Some("test-token".to_string()));
        assert!(provider.api_token.is_some());
        assert_eq!(provider.base_url, "https://gitlab.com");
    }

    #[test]
    fn test_provider_creation_with_custom_url() {
        let provider = GitLabProvider::new_with_url(
            Some("token".to_string()),
            "https://gitlab.example.com".to_string(),
        );
        assert_eq!(provider.base_url, "https://gitlab.example.com");
    }

    #[test]
    fn test_provider_creation_without_token() {
        let provider = GitLabProvider::new(None);
        assert!(provider.api_token.is_none());
    }

    #[test]
    fn test_count_diff_lines() {
        let diff = r#"
--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,4 @@
 unchanged line
-removed line
+added line 1
+added line 2
 another unchanged line
"#;

        let (additions, deletions) = GitLabProvider::count_diff_lines(diff);
        assert_eq!(additions, 2);
        assert_eq!(deletions, 1);
    }

    #[test]
    fn test_determine_status_new_file() {
        let change = GitLabMergeRequestChange {
            old_path: "".to_string(),
            new_path: "new.txt".to_string(),
            new_file: true,
            renamed_file: false,
            deleted_file: false,
            diff: String::new(),
        };

        assert_eq!(GitLabProvider::determine_status(&change), "added");
    }

    #[test]
    fn test_determine_status_deleted_file() {
        let change = GitLabMergeRequestChange {
            old_path: "old.txt".to_string(),
            new_path: "".to_string(),
            new_file: false,
            renamed_file: false,
            deleted_file: true,
            diff: String::new(),
        };

        assert_eq!(GitLabProvider::determine_status(&change), "deleted");
    }

    #[test]
    fn test_determine_status_renamed_file() {
        let change = GitLabMergeRequestChange {
            old_path: "old.txt".to_string(),
            new_path: "new.txt".to_string(),
            new_file: false,
            renamed_file: true,
            deleted_file: false,
            diff: String::new(),
        };

        assert_eq!(GitLabProvider::determine_status(&change), "renamed");
    }

    #[test]
    fn test_determine_status_modified_file() {
        let change = GitLabMergeRequestChange {
            old_path: "file.txt".to_string(),
            new_path: "file.txt".to_string(),
            new_file: false,
            renamed_file: false,
            deleted_file: false,
            diff: String::new(),
        };

        assert_eq!(GitLabProvider::determine_status(&change), "modified");
    }
}
