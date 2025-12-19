use crate::error::Result;
use crate::git::diff::ExtractedDiff;
use chrono::{DateTime, Utc};
use git2::{Repository, Commit, Diff, DiffOptions, Oid, Status, StatusOptions};

pub struct GitReader {
    repo: Repository,
}

impl std::fmt::Debug for GitReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitReader")
            .field("repo_path", &self.repo.workdir().map(|p| p.to_string_lossy()))
            .finish()
    }
}

impl Clone for GitReader {
    fn clone(&self) -> Self {
        // Clone by opening the same repository again
        let path = self.repo.workdir()
            .map(|p| p.to_string_lossy().to_string())
            .expect("Repository must have a workdir");
        Self {
            repo: Repository::open(path).expect("Should be able to reopen repository"),
        }
    }
}

impl GitReader {
    pub fn new(path: Option<&str>) -> Result<Self> {
        let repo = if let Some(p) = path {
            Repository::open(p)?
        } else {
            Repository::open_from_env()?
        };

        Ok(Self { repo })
    }

    pub fn get_repository_path(&self) -> Result<String> {
        Ok(self.repo.workdir()
            .ok_or_else(|| crate::error::KtmeError::Git(git2::Error::from_str("Not a working directory")))?
            .to_string_lossy()
            .to_string())
    }

    pub fn read_commit(&self, commit_ref: &str) -> Result<ExtractedDiff> {
        tracing::info!("Reading commit: {}", commit_ref);

        let oid = self.resolve_reference(commit_ref)?;
        let commit = self.repo.find_commit(oid)?;
        self.extract_commit_diff(&commit)
    }

    pub fn read_staged(&self) -> Result<ExtractedDiff> {
        tracing::info!("Reading staged changes");

        let head_oid = self.repo.refname_to_id("HEAD")
            .map_err(|e| crate::error::KtmeError::Git(e))?;
        let head_commit = self.repo.find_commit(head_oid)?;

        let tree_oid = self.repo.index()
            .map_err(|e| crate::error::KtmeError::Git(e))?
            .write_tree()
            .map_err(|e| crate::error::KtmeError::Git(e))?;
        let tree = self.repo.find_tree(tree_oid)
            .map_err(|e| crate::error::KtmeError::Git(e))?;

        self.extract_tree_diff("staged", "staged", &head_commit.tree()?, &tree)
    }

    pub fn read_commit_range(&self, range: &str) -> Result<Vec<ExtractedDiff>> {
        tracing::info!("Reading commit range: {}", range);

        let parts: Vec<&str> = range.split("..").collect();
        if parts.len() != 2 {
            return Err(crate::error::KtmeError::InvalidInput(
                "Invalid range format. Use: start..end".to_string()
            ));
        }

        let start_oid = self.resolve_reference(parts[0])?;
        let end_oid = self.resolve_reference(parts[1])?;

        let mut revwalk = self.repo.revwalk()
            .map_err(|e| crate::error::KtmeError::Git(e))?;
        revwalk.push_range(&format!("{}..{}", start_oid, end_oid))
            .map_err(|e| crate::error::KtmeError::Git(e))?;

        let mut diffs = Vec::new();
        for oid in revwalk {
            let oid = oid.map_err(|e| crate::error::KtmeError::Git(e))?;
            let commit = self.repo.find_commit(oid)?;
            diffs.push(self.extract_commit_diff(&commit)?);
        }

        Ok(diffs)
    }

    pub fn get_current_branch(&self) -> Result<String> {
        let head = self.repo.head()
            .map_err(|e| crate::error::KtmeError::Git(e))?;

        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }

    pub fn get_status(&self) -> Result<Vec<(String, Status)>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .include_ignored(false)
            .recurse_untracked_dirs(true);

        let statuses = self.repo.statuses(Some(&mut opts))
            .map_err(|e| crate::error::KtmeError::Git(e))?;

        let mut results = Vec::new();
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                results.push((path.to_string(), entry.status()));
            }
        }

        Ok(results)
    }

    fn resolve_reference(&self, reference: &str) -> Result<Oid> {
        // First try direct commit hash
        if let Ok(oid) = Oid::from_str(reference) {
            return Ok(oid);
        }

        // Try HEAD directly
        if reference == "HEAD" {
            if let Ok(head_ref) = self.repo.head() {
                if let Some(head_target) = head_ref.target() {
                    return Ok(head_target);
                }
            }
        }

        // Try symbolic references (HEAD, main, master, etc.)
        if let Ok(reference) = self.repo.revparse_single(reference) {
            return Ok(reference.id());
        }

        // Try common references
        let refs = [
            &format!("refs/heads/{}", reference),
            &format!("refs/tags/{}", reference),
            &format!("refs/remotes/origin/{}", reference),
        ];

        for ref_name in &refs {
            if let Ok(oid) = self.repo.refname_to_id(ref_name) {
                return Ok(oid);
            }
        }

        Err(crate::error::KtmeError::Git(
            git2::Error::from_str(&format!("Reference '{}' not found", reference))
        ))
    }

    fn extract_commit_diff(&self, commit: &Commit) -> Result<ExtractedDiff> {
        if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            self.extract_tree_diff(
                &commit.id().to_string(),
                &commit.id().to_string(),
                &parent.tree()?,
                &commit.tree()?,
            )
        } else {
            // Initial commit - compare with empty tree
            let empty_tree = self.repo.find_tree(self.repo.treebuilder(None)
                .map_err(|e| crate::error::KtmeError::Git(e))?
                .write()
                .map_err(|e| crate::error::KtmeError::Git(e))?)?;

            self.extract_tree_diff(
                &commit.id().to_string(),
                &commit.id().to_string(),
                &empty_tree,
                &commit.tree()?,
            )
        }
    }

    fn extract_tree_diff(
        &self,
        identifier: &str,
        source: &str,
        old_tree: &git2::Tree,
        new_tree: &git2::Tree,
    ) -> Result<ExtractedDiff> {
        let mut diff_opts = DiffOptions::new();
        diff_opts.context_lines(3)
            .include_unmodified(false);

        let diff = self.repo.diff_tree_to_tree(
            Some(old_tree),
            Some(new_tree),
            Some(&mut diff_opts),
        ).map_err(|e| crate::error::KtmeError::Git(e))?;

        let mut files = Vec::new();
        let mut total_additions = 0;
        let mut total_deletions = 0;

        for delta in diff.deltas() {
            let path = delta.new_file().path()
                .or_else(|| delta.old_file().path())
                .unwrap_or_else(|| std::path::Path::new(""))
                .to_string_lossy()
                .to_string();

            let status = match delta.status() {
                git2::Delta::Added => "added",
                git2::Delta::Deleted => "deleted",
                git2::Delta::Modified => "modified",
                git2::Delta::Renamed => "renamed",
                git2::Delta::Copied => "copied",
                git2::Delta::Untracked => "untracked",
                _ => "unknown",
            };

            let (additions, deletions, diff_text) = self.get_file_stats(&diff, delta.old_file().id(), delta.new_file().id())?;

            total_additions += additions;
            total_deletions += deletions;

            files.push(crate::git::diff::FileChange {
                path,
                status: status.to_string(),
                additions,
                deletions,
                diff: diff_text,
            });
        }

        // Get commit info if we have a commit ID
        let (author, message, timestamp) = if let Ok(oid) = Oid::from_str(identifier) {
            if let Ok(commit) = self.repo.find_commit(oid) {
                let author_name = commit.author().name().unwrap_or("Unknown").to_string();
                let commit_message = commit.message().unwrap_or("No message").to_string();
                let commit_time = DateTime::from_timestamp(commit.time().seconds(), 0)
                    .unwrap_or_else(|| DateTime::UNIX_EPOCH)
                    .to_rfc3339();
                (author_name, commit_message, commit_time)
            } else {
                ("Unknown".to_string(), "No message".to_string(), Utc::now().to_rfc3339())
            }
        } else {
            ("Unknown".to_string(), "No message".to_string(), Utc::now().to_rfc3339())
        };

        let total_files = files.len() as u32;
        Ok(ExtractedDiff {
            source: source.to_string(),
            identifier: identifier.to_string(),
            timestamp,
            author,
            message,
            files,
            summary: crate::git::diff::DiffSummary {
                total_files,
                total_additions,
                total_deletions,
            },
        })
    }

    fn get_file_stats(&self, diff: &Diff, _old_id: git2::Oid, _new_id: git2::Oid) -> Result<(u32, u32, String)> {
        let mut additions = 0u32;
        let mut deletions = 0u32;
        let mut diff_text = String::new();

        diff.foreach(
            &mut |delta, _hunk| {
                let path = delta.new_file().path()
                    .or_else(|| delta.old_file().path())
                    .unwrap_or_else(|| std::path::Path::new(""))
                    .to_string_lossy();

                tracing::debug!("Processing file: {}", path);
                true
            },
            None,
            Some(&mut |_delta, hunk| {
                tracing::debug!("Hunk: {} lines", hunk.old_lines());
                true
            }),
            Some(&mut |_delta, _hunk, line| {
                match line.origin() {
                    '+' | ' ' => additions += 1,
                    '-' => deletions += 1,
                    _ => {}
                }

                let prefix = match line.origin() {
                    '+' => "+",
                    '-' => "-",
                    ' ' => " ",
                    '>' => " ",
                    '<' => " ",
                    _ => "",
                };

                diff_text.push_str(&format!("{}{}\n", prefix,
                    String::from_utf8_lossy(line.content()).trim_end()));
                true
            }),
        ).map_err(|e| crate::error::KtmeError::Git(e))?;

        Ok((additions, deletions, diff_text))
    }
}
