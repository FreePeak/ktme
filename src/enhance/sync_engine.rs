use sha2::{Digest, Sha256};
use std::path::Path;

use crate::storage::models::{
    CloudSyncStatus, SyncDirection, SyncHistory, SyncHistoryStatus, SyncState,
};

pub struct SyncResult {
    pub files_processed: usize,
    pub files_modified: usize,
    pub conflicts: usize,
    pub errors: Vec<String>,
}

pub struct SyncChange {
    pub mapping_id: i64,
    pub remote_id: String,
    pub provider: String,
    pub change_type: ChangeType,
    pub local_content: Option<String>,
    pub remote_content: Option<String>,
    pub local_hash: String,
    pub remote_hash: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Unchanged,
    Conflict,
}

pub enum ConflictStrategy {
    LocalWins,
    RemoteWins,
    Timestamp,
    Manual,
}

impl Default for ConflictStrategy {
    fn default() -> Self {
        Self::RemoteWins
    }
}

pub struct SyncEngine;

impl SyncEngine {
    pub fn sync_docs(source: &Path, target: &Path, dry_run: bool) -> std::io::Result<SyncResult> {
        let mut result = SyncResult {
            files_processed: 0,
            files_modified: 0,
            conflicts: 0,
            errors: Vec::new(),
        };

        if !target.exists() {
            std::fs::create_dir_all(target)?;
        }

        if let Ok(entries) = std::fs::read_dir(source) {
            for entry in entries.flatten() {
                let source_path = entry.path();
                let target_path = target.join(source_path.file_name().unwrap());

                if source_path.is_file()
                    && source_path.extension().map(|e| e == "md").unwrap_or(false)
                {
                    result.files_processed += 1;

                    if !dry_run {
                        if let Ok(content) = std::fs::read_to_string(&source_path) {
                            if let Err(e) = std::fs::write(&target_path, content) {
                                result.errors.push(format!(
                                    "Failed to write {}: {}",
                                    target_path.display(),
                                    e
                                ));
                            } else {
                                result.files_modified += 1;
                            }
                        } else {
                            result
                                .errors
                                .push(format!("Failed to read {}", source_path.display()));
                        }
                    } else {
                        result.files_modified += 1;
                    }
                }
            }
        }

        Ok(result)
    }

    pub fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn detect_changes(
        local_content: Option<&str>,
        remote_content: Option<&str>,
    ) -> (ChangeType, String, String) {
        let local_hash = local_content
            .map(|c| Self::compute_hash(c))
            .unwrap_or_default();
        let remote_hash = remote_content
            .map(|c| Self::compute_hash(c))
            .unwrap_or_default();

        let change_type = match (local_content.is_some(), remote_content.is_some()) {
            (true, false) => ChangeType::Added,
            (false, true) => ChangeType::Deleted,
            (true, true) => {
                if local_hash == remote_hash {
                    ChangeType::Unchanged
                } else {
                    ChangeType::Modified
                }
            }
            (false, false) => ChangeType::Unchanged,
        };

        (change_type, local_hash, remote_hash)
    }

    pub fn resolve_conflict(
        local_content: &str,
        remote_content: &str,
        _local_hash: &str,
        _remote_hash: &str,
        strategy: &ConflictStrategy,
    ) -> ConflictResolution {
        match strategy {
            ConflictStrategy::LocalWins => ConflictResolution {
                content: local_content.to_string(),
                resolution: "local_wins".to_string(),
            },
            ConflictStrategy::RemoteWins => ConflictResolution {
                content: remote_content.to_string(),
                resolution: "remote_wins".to_string(),
            },
            ConflictStrategy::Timestamp => ConflictResolution {
                content: remote_content.to_string(),
                resolution: "timestamp_remote_wins".to_string(),
            },
            ConflictStrategy::Manual => ConflictResolution {
                content: String::new(),
                resolution: "manual".to_string(),
            },
        }
    }

    pub fn build_sync_change(
        mapping_id: i64,
        remote_id: String,
        provider: String,
        local_content: Option<&str>,
        remote_content: Option<&str>,
    ) -> SyncChange {
        let (change_type, local_hash, remote_hash) =
            Self::detect_changes(local_content, remote_content);

        SyncChange {
            mapping_id,
            remote_id,
            provider,
            change_type,
            local_content: local_content.map(String::from),
            remote_content: remote_content.map(String::from),
            local_hash,
            remote_hash,
        }
    }

    pub fn apply_change(
        change: &SyncChange,
        target_dir: &Path,
        strategy: &ConflictStrategy,
    ) -> std::io::Result<SyncApplyResult> {
        let file_path = target_dir.join(format!("{}.md", change.mapping_id));

        match change.change_type {
            ChangeType::Unchanged => Ok(SyncApplyResult {
                action: "skipped".to_string(),
                file_path,
                hash: change.local_hash.clone(),
            }),
            ChangeType::Added | ChangeType::Modified => {
                let content = change.remote_content.as_deref().unwrap_or("");
                std::fs::write(&file_path, content)?;
                Ok(SyncApplyResult {
                    action: if matches!(change.change_type, ChangeType::Added) {
                        "created"
                    } else {
                        "updated"
                    }
                    .to_string(),
                    file_path,
                    hash: change.remote_hash.clone(),
                })
            }
            ChangeType::Deleted => {
                if file_path.exists() {
                    std::fs::remove_file(&file_path)?;
                }
                Ok(SyncApplyResult {
                    action: "deleted".to_string(),
                    file_path,
                    hash: String::new(),
                })
            }
            ChangeType::Conflict => {
                let resolution = Self::resolve_conflict(
                    change.local_content.as_deref().unwrap_or(""),
                    change.remote_content.as_deref().unwrap_or(""),
                    &change.local_hash,
                    &change.remote_hash,
                    strategy,
                );
                if resolution.resolution == "manual" {
                    Ok(SyncApplyResult {
                        action: "conflict_manual".to_string(),
                        file_path,
                        hash: change.local_hash.clone(),
                    })
                } else {
                    std::fs::write(&file_path, &resolution.content)?;
                    Ok(SyncApplyResult {
                        action: format!("conflict_resolved_{}", resolution.resolution),
                        file_path,
                        hash: Self::compute_hash(&resolution.content),
                    })
                }
            }
        }
    }
}

pub struct ConflictResolution {
    pub content: String,
    pub resolution: String,
}

pub struct SyncApplyResult {
    pub action: String,
    pub file_path: std::path::PathBuf,
    pub hash: String,
}

pub fn create_sync_history_entry(
    provider: &str,
    direction: SyncDirection,
    mapping_id: Option<i64>,
    remote_id: Option<&str>,
    status: SyncHistoryStatus,
    changes_detected: bool,
    local_hash_before: Option<&str>,
    local_hash_after: Option<&str>,
    remote_hash_before: Option<&str>,
    remote_hash_after: Option<&str>,
    error_message: Option<&str>,
) -> SyncHistory {
    use chrono::Utc;
    use uuid::Uuid;

    SyncHistory {
        id: Uuid::new_v4().to_string(),
        provider: provider.to_string(),
        direction,
        mapping_id,
        remote_id: remote_id.map(String::from),
        status,
        changes_detected,
        local_hash_before: local_hash_before.map(String::from),
        local_hash_after: local_hash_after.map(String::from),
        remote_hash_before: remote_hash_before.map(String::from),
        remote_hash_after: remote_hash_after.map(String::from),
        error_message: error_message.map(String::from),
        synced_at: Utc::now(),
    }
}

pub fn create_cloud_sync_status(
    mapping_id: i64,
    provider: &str,
    remote_id: &str,
    content_hash: &str,
) -> CloudSyncStatus {
    use chrono::Utc;
    use uuid::Uuid;

    CloudSyncStatus {
        id: Uuid::new_v4().to_string(),
        mapping_id,
        provider: provider.to_string(),
        remote_id: remote_id.to_string(),
        content_hash_local: content_hash.to_string(),
        content_hash_remote: Some(content_hash.to_string()),
        sync_state: SyncState::Synced,
        conflict_data: None,
        last_synced: Some(Utc::now()),
        error_message: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_computation() {
        let content = "Hello, World!";
        let hash = SyncEngine::compute_hash(content);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_detect_changes_added() {
        let (change_type, local_hash, remote_hash) =
            SyncEngine::detect_changes(Some("content"), None);
        assert_eq!(change_type, ChangeType::Added);
        assert!(!local_hash.is_empty());
        assert!(remote_hash.is_empty());
    }

    #[test]
    fn test_detect_changes_deleted() {
        let (change_type, local_hash, remote_hash) =
            SyncEngine::detect_changes(None, Some("content"));
        assert_eq!(change_type, ChangeType::Deleted);
        assert!(local_hash.is_empty());
        assert!(!remote_hash.is_empty());
    }

    #[test]
    fn test_detect_changes_unchanged() {
        let content = "same content";
        let (change_type, local_hash, remote_hash) =
            SyncEngine::detect_changes(Some(content), Some(content));
        assert_eq!(change_type, ChangeType::Unchanged);
        assert_eq!(local_hash, remote_hash);
    }

    #[test]
    fn test_detect_changes_modified() {
        let (change_type, local_hash, remote_hash) =
            SyncEngine::detect_changes(Some("content1"), Some("content2"));
        assert_eq!(change_type, ChangeType::Modified);
        assert_ne!(local_hash, remote_hash);
    }

    #[test]
    fn test_conflict_resolution_local_wins() {
        let resolution = SyncEngine::resolve_conflict(
            "local content",
            "remote content",
            "local_hash",
            "remote_hash",
            &ConflictStrategy::LocalWins,
        );
        assert_eq!(resolution.content, "local content");
        assert_eq!(resolution.resolution, "local_wins");
    }

    #[test]
    fn test_conflict_resolution_remote_wins() {
        let resolution = SyncEngine::resolve_conflict(
            "local content",
            "remote content",
            "local_hash",
            "remote_hash",
            &ConflictStrategy::RemoteWins,
        );
        assert_eq!(resolution.content, "remote content");
        assert_eq!(resolution.resolution, "remote_wins");
    }

    #[test]
    fn test_build_sync_change() {
        let change = SyncEngine::build_sync_change(
            1,
            "remote_123".to_string(),
            "notion".to_string(),
            Some("local content"),
            Some("remote content"),
        );
        assert_eq!(change.mapping_id, 1);
        assert_eq!(change.remote_id, "remote_123");
        assert_eq!(change.provider, "notion");
    }
}
