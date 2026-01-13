use crate::doc::templates::TemplateEngine;
use crate::error::Result;
use regex::Regex;
use std::collections::HashMap;

pub struct DocumentGenerator {
    template: Option<String>,
    template_engine: TemplateEngine,
}

impl DocumentGenerator {
    pub fn new(template: Option<String>) -> Self {
        Self {
            template,
            template_engine: TemplateEngine::new(),
        }
    }

    /// Create a new generator and load templates from the default directory
    pub fn with_templates(template: Option<String>) -> Result<Self> {
        let mut generator = Self::new(template);

        // Try to load templates from the templates directory
        let template_dir = TemplateEngine::default_template_directory();
        if template_dir.exists() {
            generator
                .template_engine
                .load_templates_from_directory(&template_dir)?;
            tracing::info!("Loaded templates from: {}", template_dir.display());
        } else {
            tracing::warn!("Template directory not found: {}", template_dir.display());
        }

        Ok(generator)
    }

    /// Generate documentation with optional template
    pub async fn generate(&self, content: &str) -> Result<String> {
        tracing::info!("Generating documentation");

        if let Some(template_name) = &self.template {
            // Use specified template
            if self.template_engine.has_template(template_name) {
                let mut vars = HashMap::new();
                vars.insert("content".to_string(), content.to_string());
                vars.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

                self.template_engine.render(template_name, &vars)
            } else {
                tracing::warn!(
                    "Template '{}' not found, using default formatting",
                    template_name
                );
                Ok(Self::default_format(content))
            }
        } else {
            // Use default formatting
            Ok(Self::default_format(content))
        }
    }

    /// Update existing documentation with new content
    /// This implements smart merge logic based on section headers
    pub async fn update(&self, existing: &str, new_content: &str) -> Result<String> {
        tracing::info!("Updating documentation");

        // Parse sections from existing document
        let existing_sections = Self::parse_sections(existing);

        // Parse sections from new content
        let new_sections = Self::parse_sections(new_content);

        // Merge strategy:
        // 1. Keep all existing sections
        // 2. Update sections that exist in both (replace content)
        // 3. Add new sections at the end
        let mut merged_sections = existing_sections.clone();

        for (section_name, section_content) in new_sections {
            if section_name.is_empty() {
                // Preamble content - append to existing preamble
                if let Some(preamble) = merged_sections.get_mut("") {
                    preamble.push_str("\n\n");
                    preamble.push_str(&section_content);
                } else {
                    merged_sections.insert(String::new(), section_content);
                }
            } else {
                // Named section - replace or add
                merged_sections.insert(section_name, section_content);
            }
        }

        // Reconstruct document
        Ok(Self::reconstruct_document(&merged_sections))
    }

    /// Parse a markdown document into sections based on H2 headers (##)
    fn parse_sections(content: &str) -> HashMap<String, String> {
        let mut sections = HashMap::new();
        let mut current_section = String::new();
        let mut current_content = String::new();

        // Regex to match ## headers
        let header_re = Regex::new(r"^##\s+(.+)$").unwrap();

        for line in content.lines() {
            if let Some(captures) = header_re.captures(line) {
                // Save previous section
                if !current_section.is_empty() || !current_content.is_empty() {
                    sections.insert(current_section.clone(), current_content.trim().to_string());
                }

                // Start new section
                current_section = captures.get(1).unwrap().as_str().to_string();
                current_content = String::new();
            } else {
                // Add line to current section
                if !current_content.is_empty() {
                    current_content.push('\n');
                }
                current_content.push_str(line);
            }
        }

        // Save last section
        if !current_section.is_empty() || !current_content.is_empty() {
            sections.insert(current_section, current_content.trim().to_string());
        }

        sections
    }

    /// Reconstruct a markdown document from sections
    fn reconstruct_document(sections: &HashMap<String, String>) -> String {
        let mut result = String::new();

        // Add preamble first (empty key)
        if let Some(preamble) = sections.get("") {
            result.push_str(preamble);
            result.push_str("\n\n");
        }

        // Add all other sections (sorted for consistency)
        let mut section_names: Vec<_> = sections.keys().filter(|k| !k.is_empty()).collect();
        section_names.sort();

        for section_name in section_names {
            if let Some(content) = sections.get(section_name) {
                result.push_str(&format!("## {}\n\n", section_name));
                result.push_str(content);
                result.push_str("\n\n");
            }
        }

        result.trim().to_string()
    }

    /// Default formatting when no template is specified
    fn default_format(content: &str) -> String {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");
        format!(
            "# Generated Documentation\n\n**Generated**: {}\n\n---\n\n{}",
            timestamp, content
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sections_simple() {
        let content = r#"# Title

Preamble content

## Section 1

Content for section 1

## Section 2

Content for section 2"#;

        let sections = DocumentGenerator::parse_sections(content);

        assert!(sections.contains_key("Section 1"));
        assert!(sections.contains_key("Section 2"));
        assert_eq!(
            sections.get("Section 1").unwrap().trim(),
            "Content for section 1"
        );
    }

    #[test]
    fn test_parse_sections_with_preamble() {
        let content = r#"Some preamble text

## First Section

Section content"#;

        let sections = DocumentGenerator::parse_sections(content);

        assert!(sections.contains_key(""));
        assert!(sections.contains_key("First Section"));
        assert!(sections.get("").unwrap().contains("preamble"));
    }

    #[test]
    fn test_reconstruct_document() {
        let mut sections = HashMap::new();
        sections.insert(String::new(), "Preamble".to_string());
        sections.insert("Section A".to_string(), "Content A".to_string());
        sections.insert("Section B".to_string(), "Content B".to_string());

        let result = DocumentGenerator::reconstruct_document(&sections);

        assert!(result.contains("Preamble"));
        assert!(result.contains("## Section A"));
        assert!(result.contains("Content A"));
        assert!(result.contains("## Section B"));
        assert!(result.contains("Content B"));
    }

    #[tokio::test]
    async fn test_generate_default_format() {
        let generator = DocumentGenerator::new(None);
        let result = generator.generate("Test content").await.unwrap();

        assert!(result.contains("# Generated Documentation"));
        assert!(result.contains("Test content"));
        assert!(result.contains("Generated"));
    }

    #[tokio::test]
    async fn test_update_merges_sections() {
        let generator = DocumentGenerator::new(None);

        let existing = r#"# Document

## Section 1

Old content

## Section 2

Unchanged content"#;

        let new_content = r#"## Section 1

New content

## Section 3

Brand new section"#;

        let result = generator.update(existing, new_content).await.unwrap();

        // Should have updated Section 1
        assert!(result.contains("New content"));
        // Should keep Section 2
        assert!(result.contains("Unchanged content"));
        // Should add Section 3
        assert!(result.contains("Brand new section"));
    }
}
