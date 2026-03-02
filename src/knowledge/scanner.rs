use crate::error::Result;
use crate::storage::models::FeatureType;
use std::path::{Path, PathBuf};

/// A feature candidate extracted from codebase scanning.
#[derive(Debug, Clone)]
pub struct ScannedFeature {
    pub name: String,
    pub description: Option<String>,
    pub feature_type: FeatureType,
    pub file_path: String,
    /// Confidence score 0.0-1.0 indicating how certain we are this is a real feature.
    pub confidence: f64,
}

/// Heuristic scanner that walks a directory tree and extracts feature candidates.
pub struct CodebaseScanner {
    root: PathBuf,
}

impl CodebaseScanner {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Walk the root directory and return all detected feature candidates.
    pub fn scan(&self) -> Result<Vec<ScannedFeature>> {
        let mut features: Vec<ScannedFeature> = Vec::new();
        self.walk_dir(&self.root.clone(), &mut features)?;
        Ok(features)
    }

    fn walk_dir(&self, dir: &Path, features: &mut Vec<ScannedFeature>) -> Result<()> {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // Skip hidden directories, target/, node_modules/, .git/
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
            }

            if path.is_dir() {
                self.walk_dir(&path, features)?;
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext {
                    "rs" => self.scan_rust_file(&path, features),
                    "go" => self.scan_go_file(&path, features),
                    "ts" | "js" => self.scan_ts_file(&path, features),
                    "py" => self.scan_py_file(&path, features),
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn scan_rust_file(&self, path: &Path, features: &mut Vec<ScannedFeature>) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let path_str = path.to_string_lossy().to_string();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Public impl blocks suggest a feature boundary
            if trimmed.starts_with("pub struct ") || trimmed.starts_with("pub enum ") {
                let name = extract_rust_name(trimmed);
                if let Some(name) = name {
                    let description = extract_doc_comment_above(&content, i);
                    features.push(ScannedFeature {
                        name: to_feature_name(&name),
                        description,
                        feature_type: classify_by_name(&name),
                        file_path: path_str.clone(),
                        confidence: 0.7,
                    });
                }
            }

            // Feature/TODO comments
            if let Some(feat) = extract_comment_feature(trimmed) {
                features.push(ScannedFeature {
                    name: feat.clone(),
                    description: Some(format!("Extracted from comment in {}", path_str)),
                    feature_type: FeatureType::Other,
                    file_path: path_str.clone(),
                    confidence: 0.5,
                });
            }
        }
    }

    fn scan_go_file(&self, path: &Path, features: &mut Vec<ScannedFeature>) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let path_str = path.to_string_lossy().to_string();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("type ") && trimmed.contains(" struct") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[1];
                    let description = extract_doc_comment_above(&content, i);
                    features.push(ScannedFeature {
                        name: to_feature_name(name),
                        description,
                        feature_type: classify_by_name(name),
                        file_path: path_str.clone(),
                        confidence: 0.65,
                    });
                }
            }
            if let Some(feat) = extract_comment_feature(trimmed) {
                features.push(ScannedFeature {
                    name: feat,
                    description: Some(format!("Extracted from comment in {}", path_str)),
                    feature_type: FeatureType::Other,
                    file_path: path_str.clone(),
                    confidence: 0.5,
                });
            }
        }
    }

    fn scan_ts_file(&self, path: &Path, features: &mut Vec<ScannedFeature>) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let path_str = path.to_string_lossy().to_string();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("export class ")
                || trimmed.starts_with("export interface ")
                || trimmed.starts_with("class ")
            {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let idx = if parts[0] == "export" { 2 } else { 1 };
                    if let Some(name) = parts.get(idx) {
                        let description = extract_doc_comment_above(&content, i);
                        features.push(ScannedFeature {
                            name: to_feature_name(name),
                            description,
                            feature_type: classify_by_name(name),
                            file_path: path_str.clone(),
                            confidence: 0.65,
                        });
                    }
                }
            }
            if let Some(feat) = extract_comment_feature(trimmed) {
                features.push(ScannedFeature {
                    name: feat,
                    description: Some(format!("Extracted from comment in {}", path_str)),
                    feature_type: FeatureType::Other,
                    file_path: path_str.clone(),
                    confidence: 0.5,
                });
            }
        }
    }

    fn scan_py_file(&self, path: &Path, features: &mut Vec<ScannedFeature>) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let path_str = path.to_string_lossy().to_string();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("class ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[1].trim_end_matches(':').trim_end_matches('(');
                    let description = extract_doc_comment_above(&content, i);
                    features.push(ScannedFeature {
                        name: to_feature_name(name),
                        description,
                        feature_type: classify_by_name(name),
                        file_path: path_str.clone(),
                        confidence: 0.65,
                    });
                }
            }
            if let Some(feat) = extract_comment_feature(trimmed) {
                features.push(ScannedFeature {
                    name: feat,
                    description: Some(format!("Extracted from comment in {}", path_str)),
                    feature_type: FeatureType::Other,
                    file_path: path_str.clone(),
                    confidence: 0.5,
                });
            }
        }
    }
}

// ---- helpers ----

fn extract_rust_name(line: &str) -> Option<String> {
    let after = line
        .trim_start_matches("pub struct ")
        .trim_start_matches("pub enum ")
        .trim_start_matches("struct ")
        .trim_start_matches("enum ");
    let name = after.split_whitespace().next()?.trim_end_matches('{');
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// Convert PascalCase / snake_case identifier to a human-readable feature name.
fn to_feature_name(name: &str) -> String {
    // Insert space before uppercase after lowercase (PascalCase)
    let mut result = String::new();
    let mut prev_lower = false;
    for ch in name.chars() {
        if ch.is_uppercase() && prev_lower {
            result.push(' ');
        }
        // Replace underscores with spaces
        if ch == '_' {
            result.push(' ');
            prev_lower = false;
        } else {
            result.push(ch);
            prev_lower = ch.is_lowercase();
        }
    }
    result
}

/// Look back up to 5 lines for a `///` or `#` doc comment above line `idx`.
fn extract_doc_comment_above(content: &str, idx: usize) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let start = idx.saturating_sub(5);
    let mut doc_lines: Vec<String> = Vec::new();
    for line in &lines[start..idx] {
        let t = line.trim();
        if let Some(stripped) = t.strip_prefix("///") {
            doc_lines.push(stripped.trim().to_string());
        } else if let Some(stripped) = t.strip_prefix("//") {
            doc_lines.push(stripped.trim().to_string());
        } else if let Some(stripped) = t.strip_prefix('#') {
            doc_lines.push(stripped.trim().to_string());
        }
    }
    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join(" "))
    }
}

/// Extract FEATURE: or TODO: markers as feature names.
fn extract_comment_feature(line: &str) -> Option<String> {
    for prefix in &["FEATURE:", "// FEATURE:", "# FEATURE:"] {
        if let Some(rest) = line.to_uppercase().find("FEATURE:") {
            let raw = &line[rest + "FEATURE:".len()..].trim().to_string();
            if !raw.is_empty() {
                return Some(raw.clone());
            }
        }
        let _ = prefix; // used via the find above
    }
    None
}

/// Classify a feature's type based on common naming patterns.
fn classify_by_name(name: &str) -> FeatureType {
    let lower = name.to_lowercase();
    if lower.contains("api")
        || lower.contains("endpoint")
        || lower.contains("route")
        || lower.contains("handler")
    {
        FeatureType::Api
    } else if lower.contains("auth")
        || lower.contains("security")
        || lower.contains("token")
        || lower.contains("jwt")
        || lower.contains("oauth")
    {
        FeatureType::Security
    } else if lower.contains("db")
        || lower.contains("database")
        || lower.contains("repo")
        || lower.contains("storage")
        || lower.contains("model")
    {
        FeatureType::Database
    } else if lower.contains("config") || lower.contains("setting") || lower.contains("env") {
        FeatureType::Config
    } else if lower.contains("test")
        || lower.contains("spec")
        || lower.contains("mock")
        || lower.contains("fixture")
    {
        FeatureType::Testing
    } else if lower.contains("deploy")
        || lower.contains("docker")
        || lower.contains("k8s")
        || lower.contains("ci")
    {
        FeatureType::Deployment
    } else if lower.contains("ui")
        || lower.contains("component")
        || lower.contains("view")
        || lower.contains("page")
    {
        FeatureType::Ui
    } else if lower.contains("cache") || lower.contains("perf") || lower.contains("optim") {
        FeatureType::Performance
    } else {
        FeatureType::BusinessLogic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_on_self() {
        // Run the scanner on the ktme src/ directory. We only validate it doesn't panic
        // and finds at least one feature from the Rust files there.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let scanner = CodebaseScanner::new(root);
        let features = scanner.scan().expect("Scanner failed");
        assert!(
            !features.is_empty(),
            "Scanner should find at least one feature in src/"
        );
    }

    #[test]
    fn test_classify_by_name() {
        assert_eq!(classify_by_name("UserRepository"), FeatureType::Database);
        assert_eq!(classify_by_name("AuthService"), FeatureType::Security);
        assert_eq!(classify_by_name("ApiHandler"), FeatureType::Api);
        assert_eq!(classify_by_name("AppConfig"), FeatureType::Config);
    }

    #[test]
    fn test_to_feature_name() {
        assert_eq!(to_feature_name("UserRepository"), "User Repository");
        assert_eq!(to_feature_name("snake_case_name"), "snake case name");
        assert_eq!(to_feature_name("SimpleFeature"), "Simple Feature");
    }
}
