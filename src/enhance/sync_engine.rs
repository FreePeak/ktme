use std::path::Path;

pub struct SyncResult {
    pub files_processed: usize,
    pub files_modified: usize,
    pub errors: Vec<String>,
}

pub struct SyncEngine;

impl SyncEngine {
    pub fn sync_docs(source: &Path, target: &Path, dry_run: bool) -> std::io::Result<SyncResult> {
        let mut result = SyncResult {
            files_processed: 0,
            files_modified: 0,
            errors: Vec::new(),
        };

        if !target.exists() {
            std::fs::create_dir_all(target)?;
        }

        if let Ok(entries) = std::fs::read_dir(source) {
            for entry in entries {
                let entry = entry?;
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
}
