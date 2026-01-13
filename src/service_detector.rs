use crate::ai::AIClient;
use crate::error::{KtmeError, Result};
use std::fs;
use std::path::PathBuf;

pub struct ServiceDetector {
    current_dir: PathBuf,
}

impl ServiceDetector {
    pub fn new() -> Result<Self> {
        let current_dir = std::env::current_dir().map_err(|e| KtmeError::Io(e))?;

        Ok(Self { current_dir })
    }

    pub fn from_directory(dir: PathBuf) -> Self {
        Self { current_dir: dir }
    }

    /// Detect service name using multiple strategies
    pub async fn detect_service_name(&self) -> Result<String> {
        // Strategy 1: Get directory name containing .git
        if let Some(name) = self.detect_from_git_repository()? {
            tracing::info!("Detected service name from Git repository: {}", name);
            return Ok(name);
        }

        // Strategy 2: Check for common project indicators
        if let Some(name) = self.detect_from_project_indicators()? {
            tracing::info!("Detected service name from project indicators: {}", name);
            return Ok(name);
        }

        // Strategy 3: Use current directory name as fallback
        let dir_name = self
            .current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown-service")
            .to_string();

        tracing::info!("Using directory name as service: {}", dir_name);
        Ok(dir_name)
    }

    /// Detect service name with AI agent fallback
    pub async fn detect_with_ai_fallback(&self) -> Result<String> {
        // First try automatic detection
        if let Ok(name) = self.detect_service_name().await {
            // Validate that this looks like a real service name
            if self.is_valid_service_name(&name) {
                return Ok(name);
            }
        }

        // Use AI agent as fallback
        self.ask_ai_for_service_name().await
    }

    /// Find the directory containing .git and return its name
    fn detect_from_git_repository(&self) -> Result<Option<String>> {
        let mut current_path = self.current_dir.clone();

        loop {
            let git_path = current_path.join(".git");

            if git_path.exists() {
                // Found .git directory, return the parent directory name
                if let Some(dir_name) = current_path.file_name() {
                    return Ok(dir_name.to_str().map(|s| s.to_string()));
                }
                break;
            }

            // Move up to parent directory
            if !current_path.pop() {
                // Reached root directory without finding .git
                break;
            }
        }

        Ok(None)
    }

    /// Detect service name from common project files
    fn detect_from_project_indicators(&self) -> Result<Option<String>> {
        // Check for Cargo.toml (Rust)
        let cargo_toml = self.current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Some(name) = self.extract_name_from_cargo_toml(&cargo_toml)? {
                return Ok(Some(name));
            }
        }

        // Check for package.json (Node.js)
        let package_json = self.current_dir.join("package.json");
        if package_json.exists() {
            if let Some(name) = self.extract_name_from_package_json(&package_json)? {
                return Ok(Some(name));
            }
        }

        // Check for go.mod (Go)
        let go_mod = self.current_dir.join("go.mod");
        if go_mod.exists() {
            if let Some(name) = self.extract_name_from_go_mod(&go_mod)? {
                return Ok(Some(name));
            }
        }

        // Check for pyproject.toml (Python)
        let pyproject_toml = self.current_dir.join("pyproject.toml");
        if pyproject_toml.exists() {
            if let Some(name) = self.extract_name_from_pyproject_toml(&pyproject_toml)? {
                return Ok(Some(name));
            }
        }

        Ok(None)
    }

    fn extract_name_from_cargo_toml(&self, path: &PathBuf) -> Result<Option<String>> {
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            if line.trim().starts_with("name = ") {
                let name_part = line.split('=').nth(1).unwrap_or("").trim();
                let name = name_part.trim_matches('"');
                if !name.is_empty() {
                    return Ok(Some(name.to_string()));
                }
            }
        }
        Ok(None)
    }

    fn extract_name_from_package_json(&self, path: &PathBuf) -> Result<Option<String>> {
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            if line.trim().starts_with("\"name\":") {
                let name_part = line.split(':').nth(1).unwrap_or("").trim();
                let name = name_part.trim_matches('"').trim_matches(',');
                if !name.is_empty() {
                    return Ok(Some(name.to_string()));
                }
            }
        }
        Ok(None)
    }

    fn extract_name_from_go_mod(&self, path: &PathBuf) -> Result<Option<String>> {
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            if line.trim().starts_with("module ") {
                let module_part = line.strip_prefix("module ").unwrap_or("").trim();
                if let Some(name) = module_part.split('/').last() {
                    let name = name.trim_matches('"');
                    if !name.is_empty() {
                        return Ok(Some(name.to_string()));
                    }
                }
            }
        }
        Ok(None)
    }

    fn extract_name_from_pyproject_toml(&self, path: &PathBuf) -> Result<Option<String>> {
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            if line.trim().starts_with("name = ") {
                let name_part = line.split('=').nth(1).unwrap_or("").trim();
                let name = name_part.trim_matches('"');
                if !name.is_empty() {
                    return Ok(Some(name.to_string()));
                }
            }
        }
        Ok(None)
    }

    fn is_valid_service_name(&self, name: &str) -> bool {
        // Basic validation: not empty, not generic, reasonable length
        !name.is_empty()
            && name != "unknown-service"
            && name != "app"
            && name != "project"
            && name.len() >= 2
            && name.len() <= 50
    }

    /// Ask AI agent to determine service name from project context
    async fn ask_ai_for_service_name(&self) -> Result<String> {
        let mut context_info = String::new();

        // Gather context about the current directory
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            context_info.push_str("Directory contents:\n");
            for entry in entries.take(10) {
                // Limit to first 10 files
                if let Ok(entry) = entry {
                    if let Some(name) = entry.file_name().to_str() {
                        context_info.push_str(&format!("  - {}\n", name));
                    }
                }
            }
        }

        // Check for README files
        for readme_file in ["README.md", "README.txt", "README"] {
            let readme_path = self.current_dir.join(readme_file);
            if readme_path.exists() {
                if let Ok(content) = fs::read_to_string(&readme_path) {
                    let preview = content.lines().take(5).collect::<Vec<_>>().join("\n");
                    context_info.push_str(&format!("\n{} preview:\n{}\n", readme_file, preview));
                    break;
                }
            }
        }

        let prompt = format!(
            "Based on the following project context, determine the most appropriate service name. \
            The service name should be concise, descriptive, and suitable for documentation purposes.\n\n\
            Current directory: {}\n\
            \n\
            Project context:\n{}\n\n\
            Respond with ONLY the service name, nothing else. Use kebab-case if appropriate.",
            self.current_dir.display(),
            context_info
        );

        match AIClient::new() {
            Ok(ai_client) => {
                let response = ai_client
                    .generate_documentation(&prompt)
                    .await
                    .map_err(|e| {
                        KtmeError::Storage(format!("AI service detection failed: {}", e))
                    })?;

                let service_name = response
                    .trim()
                    .lines()
                    .next()
                    .unwrap_or("unknown-service")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');

                if service_name.is_empty() {
                    return Ok("unknown-service".to_string());
                }

                Ok(service_name.to_string())
            }
            Err(_) => {
                tracing::warn!("AI client not available for service name detection");
                Ok("unknown-service".to_string())
            }
        }
    }

    /// Get the Git repository root directory
    pub fn get_git_repository_root(&self) -> Option<PathBuf> {
        let mut current_path = self.current_dir.clone();

        loop {
            let git_path = current_path.join(".git");

            if git_path.exists() {
                return Some(current_path);
            }

            if !current_path.pop() {
                break;
            }
        }

        None
    }

    /// Get repository information
    pub fn get_repository_info(&self) -> RepositoryInfo {
        let repo_root = self.get_git_repository_root();
        let is_git_repo = repo_root.is_some();
        let current_dir = self.current_dir.clone();

        RepositoryInfo {
            current_dir,
            repository_root: repo_root,
            is_git_repository: is_git_repo,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepositoryInfo {
    pub current_dir: PathBuf,
    pub repository_root: Option<PathBuf>,
    pub is_git_repository: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_detector_creation() {
        let detector = ServiceDetector::new();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_valid_service_name() {
        let detector = ServiceDetector::new().unwrap();

        assert!(detector.is_valid_service_name("my-service"));
        assert!(detector.is_valid_service_name("user-api"));

        assert!(!detector.is_valid_service_name(""));
        assert!(!detector.is_valid_service_name("unknown-service"));
        assert!(!detector.is_valid_service_name("app"));
    }
}
