use super::{
    config::MarkdownConfig, Document, DocumentMetadata, DocumentProvider, PublishResult,
    PublishStatus,
};
use crate::error::{KtmeError, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Markdown file provider
pub struct MarkdownProvider {
    config: MarkdownConfig,
    base_path: PathBuf,
}

impl MarkdownProvider {
    pub fn new(config: MarkdownConfig) -> Self {
        let base_path = PathBuf::from(&config.base_path);
        Self { config, base_path }
    }

    fn resolve_path(&self, location: &str) -> PathBuf {
        let mut path = if Path::new(location).is_absolute() {
            PathBuf::from(location)
        } else {
            self.base_path.join(location)
        };

        // Ensure the file has the correct extension
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ext != self.config.extension {
                path.set_extension(&self.config.extension);
            }
        } else {
            path.set_extension(&self.config.extension);
        }

        path
    }

    fn read_file(&self, path: &Path) -> Result<String> {
        std::fs::read_to_string(path).map_err(KtmeError::Io)
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        if self.config.auto_create_dirs {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(KtmeError::Io)?;
            }
        }

        std::fs::write(path, content).map_err(KtmeError::Io)
    }

    fn file_metadata(&self, path: &Path) -> Result<DocumentMetadata> {
        let metadata = std::fs::metadata(path).map_err(KtmeError::Io)?;

        let created_at = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| {
                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                    .unwrap_or_else(|| chrono::Utc::now())
            })
            .map(|dt| dt.to_rfc3339());

        let updated_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| {
                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                    .unwrap_or_else(|| chrono::Utc::now())
            })
            .map(|dt| dt.to_rfc3339());

        Ok(DocumentMetadata {
            created_at,
            updated_at,
            author: None,
            version: None,
            labels: vec![],
        })
    }
}

#[async_trait]
impl DocumentProvider for MarkdownProvider {
    fn name(&self) -> &str {
        "markdown"
    }

    async fn health_check(&self) -> Result<bool> {
        match std::fs::create_dir_all(&self.base_path) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_document(&self, id: &str) -> Result<Option<Document>> {
        let path = self.resolve_path(id);

        if !path.exists() {
            return Ok(None);
        }

        let content = self.read_file(&path)?;
        let metadata = self.file_metadata(&path)?;
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        Ok(Some(Document {
            id: id.to_string(),
            title,
            content,
            url: Some(path.to_string_lossy().to_string()),
            parent_id: path
                .parent()
                .and_then(|p| p.to_str())
                .map(|s| s.to_string()),
            metadata,
        }))
    }

    async fn find_document(&self, title: &str) -> Result<Option<Document>> {
        let search_path = self.resolve_path(title);
        self.get_document(&search_path.to_string_lossy()).await
    }

    async fn create_document(&self, doc: &Document) -> Result<PublishResult> {
        let path = self.resolve_path(&doc.id);

        if path.exists() {
            return Err(KtmeError::DocumentExists(
                path.to_string_lossy().to_string(),
            ));
        }

        self.write_file(&path, &doc.content)?;

        Ok(PublishResult {
            document_id: doc.id.clone(),
            url: path.to_string_lossy().to_string(),
            version: 1,
            status: PublishStatus::Created,
        })
    }

    async fn update_document(&self, id: &str, content: &str) -> Result<PublishResult> {
        let path = self.resolve_path(id);

        if !path.exists() {
            return Err(KtmeError::DocumentNotFound(id.to_string()));
        }

        let old_content = self.read_file(&path)?;

        if old_content == content {
            return Ok(PublishResult {
                document_id: id.to_string(),
                url: path.to_string_lossy().to_string(),
                version: 1,
                status: PublishStatus::NoChanges,
            });
        }

        self.write_file(&path, content)?;

        Ok(PublishResult {
            document_id: id.to_string(),
            url: path.to_string_lossy().to_string(),
            version: 2,
            status: PublishStatus::Updated,
        })
    }

    async fn update_section(
        &self,
        id: &str,
        section: &str,
        content: &str,
    ) -> Result<PublishResult> {
        let path = self.resolve_path(id);

        if !path.exists() {
            return Err(KtmeError::DocumentNotFound(id.to_string()));
        }

        let old_content = self.read_file(&path)?;

        // Simple section replacement - look for a header and replace content under it
        let section_header = format!("## {}", section);
        let next_header = "\n## ";

        let new_content = if let Some(start) = old_content.find(&section_header) {
            if let Some(end) = old_content[start..].find(next_header) {
                let _end_pos = start + end;
                format!(
                    "{}\n{}\n{}",
                    &old_content[..start],
                    &section_header,
                    content
                )
            } else {
                // No next header, replace till end
                format!(
                    "{}\n{}\n{}",
                    &old_content[..start],
                    &section_header,
                    content
                )
            }
        } else {
            // Section not found, append to end
            format!("{}\n\n## {}\n{}", old_content, section, content)
        };

        self.write_file(&path, &new_content)?;

        Ok(PublishResult {
            document_id: id.to_string(),
            url: path.to_string_lossy().to_string(),
            version: 2,
            status: PublishStatus::Updated,
        })
    }

    async fn delete_document(&self, id: &str) -> Result<()> {
        let path = self.resolve_path(id);

        if path.exists() {
            std::fs::remove_file(&path).map_err(KtmeError::Io)?;
        }

        Ok(())
    }

    async fn list_documents(&self, container: &str) -> Result<Vec<Document>> {
        let container_path = self.base_path.join(container);

        if !container_path.exists() {
            return Ok(vec![]);
        }

        let mut documents = Vec::new();

        let entries = std::fs::read_dir(&container_path).map_err(KtmeError::Io)?;

        for entry in entries {
            let entry = entry.map_err(KtmeError::Io)?;
            let path = entry.path();

            if path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == self.config.extension)
                .unwrap_or(false)
            {
                if let Some(id) = path.file_name().and_then(|s| s.to_str()) {
                    if let Some(doc) = self.get_document(id).await? {
                        documents.push(doc);
                    }
                }
            }
        }

        Ok(documents)
    }

    async fn search_documents(&self, query: &str) -> Result<Vec<Document>> {
        let mut matches = Vec::new();

        fn search_dir(
            path: &Path,
            query: &str,
            matches: &mut Vec<std::path::PathBuf>,
        ) -> Result<()> {
            if path.is_dir() {
                for entry in std::fs::read_dir(path).map_err(KtmeError::Io)? {
                    let entry = entry.map_err(KtmeError::Io)?;
                    search_dir(&entry.path(), query, matches)?;
                }
            } else {
                matches.push(path.to_path_buf());
            }
            Ok(())
        }

        let mut file_paths = Vec::new();
        search_dir(&self.base_path, query, &mut file_paths)?;

        // Now check each file for content matching
        for path in file_paths {
            if path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == self.config.extension)
                .unwrap_or(false)
            {
                if let Some(id) = path.file_name().and_then(|s| s.to_str()) {
                    if let Ok(Some(doc)) = self.get_document(id).await {
                        if doc.content.contains(query) || doc.title.contains(query) {
                            matches.push(doc);
                        }
                    }
                }
            }
        }

        Ok(matches)
    }

    fn config(&self) -> &super::config::ProviderConfig {
        // Return a default config reference
        // In practice, this should be stored during provider creation
        static DEFAULT_CONFIG: std::sync::OnceLock<super::config::ProviderConfig> =
            std::sync::OnceLock::new();
        DEFAULT_CONFIG.get_or_init(|| super::config::ProviderConfig {
            id: 0,
            provider_type: "markdown".to_string(),
            config: serde_json::to_value(&self.config).unwrap(),
            is_default: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_markdown_provider() {
        let temp_dir = TempDir::new().unwrap();
        let config = MarkdownConfig {
            base_path: temp_dir.path().to_string_lossy().to_string(),
            extension: "md".to_string(),
            auto_create_dirs: true,
        };

        let provider = MarkdownProvider::new(config);

        // Test health check
        assert!(provider.health_check().await.unwrap());

        // Test create document
        let doc = Document {
            id: "test".to_string(),
            title: "Test Document".to_string(),
            content: "# Test Content\n\nThis is a test.".to_string(),
            url: None,
            parent_id: None,
            metadata: DocumentMetadata::default(),
        };

        let result = provider.create_document(&doc).await.unwrap();
        assert!(matches!(result.status, PublishStatus::Created));

        // Test get document
        let retrieved = provider.get_document("test").await.unwrap().unwrap();
        assert_eq!(retrieved.title, "test");
        assert_eq!(retrieved.content, doc.content);

        // Test update document
        let new_content = "# Updated Content\n\nThis has been updated.";
        let result = provider.update_document("test", new_content).await.unwrap();
        assert!(matches!(result.status, PublishStatus::Updated));

        // Test update section
        let section_content = "Section content here.";
        let result = provider
            .update_section("test", "Section", section_content)
            .await
            .unwrap();
        assert!(matches!(result.status, PublishStatus::Updated));

        // Test delete document
        provider.delete_document("test").await.unwrap();
        let deleted = provider.get_document("test").await.unwrap();
        assert!(deleted.is_none());
    }
}
