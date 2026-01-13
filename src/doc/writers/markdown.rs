use crate::doc::generator::DocumentGenerator;
use crate::error::Result;
use std::path::Path;

pub struct MarkdownWriter {
    generator: DocumentGenerator,
}

impl MarkdownWriter {
    pub fn new(_base_path: Option<String>) -> Self {
        Self {
            generator: DocumentGenerator::new(None),
        }
    }

    pub async fn write(&self, path: &Path, content: &str) -> Result<()> {
        tracing::info!("Writing markdown to: {}", path.display());

        // Generate formatted content
        let formatted = self.generator.generate(content).await?;

        std::fs::write(path, formatted)?;
        Ok(())
    }

    pub async fn update(&self, path: &Path, content: &str, section: Option<&str>) -> Result<()> {
        tracing::info!("Updating markdown at: {}", path.display());

        // Read existing content
        let existing = std::fs::read_to_string(path).unwrap_or_default();

        let updated_content = if let Some(sec) = section {
            tracing::info!("Updating section: {}", sec);
            // Section-specific update: use smart merge for that specific section
            Self::update_specific_section(&existing, sec, content)
        } else {
            // Full document update: use DocumentGenerator's smart merge
            self.generator.update(&existing, content).await?
        };

        std::fs::write(path, updated_content)?;
        Ok(())
    }

    /// Update a specific section in the markdown document
    fn update_specific_section(existing: &str, section_name: &str, new_content: &str) -> String {
        let lines: Vec<&str> = existing.lines().collect();
        let mut result: Vec<String> = Vec::new();
        let mut in_target_section = false;
        let mut section_found = false;
        let section_header = format!("## {}", section_name);

        for line in lines {
            if line.trim() == section_header.trim()
                || (line.starts_with("##")
                    && line.to_lowercase().contains(&section_name.to_lowercase()))
            {
                // Found the target section
                in_target_section = true;
                section_found = true;
                result.push(line.to_string());
                result.push("".to_string());
                result.extend(new_content.lines().map(|l| l.to_string()));
                result.push("".to_string());
            } else if in_target_section && line.starts_with("##") {
                // Reached next section, stop replacing
                in_target_section = false;
                result.push(line.to_string());
            } else if !in_target_section {
                // Keep lines outside target section
                result.push(line.to_string());
            }
            // Skip lines inside the target section (they're being replaced)
        }

        if !section_found {
            // Section not found, append to end
            result.push("".to_string());
            result.push(section_header);
            result.push("".to_string());
            result.extend(new_content.lines().map(|l| l.to_string()));
        }

        result.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_specific_section() {
        let existing = r#"# Document

## Introduction
Old intro content

## Features
Old features

## Conclusion
End"#;

        let new_content = "New features content\nWith multiple lines";
        let result = MarkdownWriter::update_specific_section(existing, "Features", new_content);

        assert!(result.contains("## Introduction"));
        assert!(result.contains("Old intro content"));
        assert!(result.contains("New features content"));
        assert!(!result.contains("Old features"));
        assert!(result.contains("## Conclusion"));
    }

    #[test]
    fn test_update_specific_section_not_found() {
        let existing = r#"# Document

## Introduction
Content"#;

        let new_content = "New section content";
        let result = MarkdownWriter::update_specific_section(existing, "NewSection", new_content);

        assert!(result.contains("## Introduction"));
        assert!(result.contains("## NewSection"));
        assert!(result.contains("New section content"));
    }
}
