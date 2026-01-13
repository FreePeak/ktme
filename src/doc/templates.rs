use crate::error::{KtmeError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct TemplateEngine {
    templates: HashMap<String, String>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Load a template from a string content
    pub fn load_template(&mut self, name: &str, content: String) {
        self.templates.insert(name.to_string(), content);
    }

    /// Load a template from a file
    pub fn load_template_from_file(&mut self, name: &str, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path).map_err(|e| KtmeError::Io(e))?;
        self.load_template(name, content);
        Ok(())
    }

    /// Load all templates from a directory
    pub fn load_templates_from_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Err(KtmeError::InvalidInput(format!(
                "Template directory does not exist: {}",
                dir.display()
            )));
        }

        for entry in fs::read_dir(dir).map_err(|e| KtmeError::Io(e))? {
            let entry = entry.map_err(|e| KtmeError::Io(e))?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                let template_name = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
                    KtmeError::InvalidInput(format!(
                        "Invalid template filename: {}",
                        path.display()
                    ))
                })?;

                self.load_template_from_file(template_name, &path)?;
                tracing::debug!("Loaded template: {}", template_name);
            }
        }

        Ok(())
    }

    /// Render a template with variable substitution
    /// Variables are specified as {{variable_name}} in the template
    pub fn render(
        &self,
        template_name: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String> {
        tracing::info!("Rendering template: {}", template_name);

        let template = self.templates.get(template_name).ok_or_else(|| {
            KtmeError::InvalidInput(format!("Template not found: {}", template_name))
        })?;

        let mut result = template.clone();

        // Replace all {{variable}} occurrences with their values
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Replace any remaining placeholders with empty strings
        // This handles optional variables that weren't provided
        result = Self::remove_empty_placeholders(&result);

        Ok(result)
    }

    /// Render a template directly from content (without loading)
    pub fn render_content(content: &str, variables: &HashMap<String, String>) -> String {
        let mut result = content.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Self::remove_empty_placeholders(&result)
    }

    /// Remove any remaining {{placeholder}} that weren't replaced
    fn remove_empty_placeholders(content: &str) -> String {
        // Use regex to find and remove any remaining {{...}} patterns
        let re = regex::Regex::new(r"\{\{[^}]+\}\}").unwrap();
        re.replace_all(content, "").to_string()
    }

    /// Get list of loaded template names
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Check if a template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }

    /// Get the default template directory path
    pub fn default_template_directory() -> PathBuf {
        PathBuf::from("templates")
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_variable_substitution() {
        let engine = TemplateEngine::new();
        let template = "Hello {{name}}! Welcome to {{place}}.";

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("place".to_string(), "Wonderland".to_string());

        let result = TemplateEngine::render_content(template, &vars);
        assert_eq!(result, "Hello Alice! Welcome to Wonderland.");
    }

    #[test]
    fn test_missing_variables_removed() {
        let engine = TemplateEngine::new();
        let template = "Hello {{name}}! You have {{count}} messages.";

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Bob".to_string());
        // count is missing

        let result = TemplateEngine::render_content(template, &vars);
        assert_eq!(result, "Hello Bob! You have  messages.");
    }

    #[test]
    fn test_load_and_render_template() {
        let mut engine = TemplateEngine::new();
        engine.load_template(
            "test",
            "Service: {{service}}\nVersion: {{version}}".to_string(),
        );

        let mut vars = HashMap::new();
        vars.insert("service".to_string(), "ktme".to_string());
        vars.insert("version".to_string(), "0.1.0".to_string());

        let result = engine.render("test", &vars).unwrap();
        assert_eq!(result, "Service: ktme\nVersion: 0.1.0");
    }

    #[test]
    fn test_template_not_found() {
        let engine = TemplateEngine::new();
        let vars = HashMap::new();

        let result = engine.render("nonexistent", &vars);
        assert!(result.is_err());
        assert!(matches!(result, Err(KtmeError::InvalidInput(_))));
    }

    #[test]
    fn test_has_template() {
        let mut engine = TemplateEngine::new();
        engine.load_template("test", "content".to_string());

        assert!(engine.has_template("test"));
        assert!(!engine.has_template("other"));
    }

    #[test]
    fn test_list_templates() {
        let mut engine = TemplateEngine::new();
        engine.load_template("template1", "content1".to_string());
        engine.load_template("template2", "content2".to_string());

        let templates = engine.list_templates();
        assert_eq!(templates.len(), 2);
        assert!(templates.contains(&"template1".to_string()));
        assert!(templates.contains(&"template2".to_string()));
    }
}
