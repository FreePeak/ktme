use std::path::{Path, PathBuf};

pub struct DocumentNode {
    pub path: PathBuf,
    pub title: String,
    pub sections: Vec<Section>,
    pub code_blocks: Vec<CodeBlock>,
    pub links: Vec<Link>,
    pub line_count: usize,
}

pub struct Section {
    pub title: String,
    pub level: usize,
    pub line_number: usize,
    pub content: String,
}

pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
    pub line_number: usize,
}

pub struct Link {
    pub text: String,
    pub url: String,
    pub line_number: usize,
}

pub struct DocParser;

impl DocParser {
    pub fn parse_directory(docs_dir: &Path) -> std::io::Result<Vec<DocumentNode>> {
        let mut nodes = Vec::new();
        if let Ok(entries) = std::fs::read_dir(docs_dir) {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(node) = Self::parse_file(&path) {
                        nodes.push(node);
                    }
                }
            }
        }
        Ok(nodes)
    }

    pub fn parse_file(path: &Path) -> std::io::Result<DocumentNode> {
        let content = std::fs::read_to_string(path)?;
        let line_count = content.lines().count();

        let mut title = String::new();
        let mut sections = Vec::new();
        let mut code_blocks = Vec::new();
        let mut links = Vec::new();

        let mut current_section: Option<Section> = None;
        let mut in_code_block = false;
        let mut current_code_lang = String::new();
        let mut current_code = String::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("```") {
                if !in_code_block {
                    in_code_block = true;
                    current_code_lang = trimmed[3..].trim().to_string();
                    current_code.clear();
                } else {
                    in_code_block = false;
                    code_blocks.push(CodeBlock {
                        language: if current_code_lang.is_empty() {
                            None
                        } else {
                            Some(current_code_lang.clone())
                        },
                        code: current_code.clone(),
                        line_number: line_num + 1,
                    });
                }
                continue;
            }

            if in_code_block {
                current_code.push_str(line);
                current_code.push('\n');
                continue;
            }

            if let Some(h1_pos) = trimmed.find("# ") {
                let hashes = trimmed[..h1_pos].chars().filter(|c| *c == '#').count();
                if hashes > 0 && (h1_pos == 0 || trimmed.chars().nth(h1_pos - 1) == Some(' ')) {
                    if let Some(section) = current_section.take() {
                        sections.push(section);
                    }

                    let section_title = trimmed
                        .trim_start_matches(|c: char| c == '#' || c == ' ')
                        .to_string();

                    if hashes == 1 {
                        title = section_title;
                    } else {
                        current_section = Some(Section {
                            title: section_title,
                            level: hashes,
                            line_number: line_num + 1,
                            content: String::new(),
                        });
                    }
                }
            }

            if !in_code_block {
                if let Some(ref mut section) = current_section {
                    section.content.push_str(line);
                    section.content.push('\n');
                }
            }

            if trimmed.starts_with('[') && trimmed.contains("](") {
                if let Some(link_start) = trimmed.find("](") {
                    let link_text = &trimmed[1..link_start];
                    let link_end = trimmed
                        .find(')')
                        .map(|p| &trimmed[link_start + 2..p])
                        .unwrap_or("");
                    if !link_text.is_empty() && !link_end.starts_with('#') {
                        links.push(Link {
                            text: link_text.to_string(),
                            url: link_end.to_string(),
                            line_number: line_num + 1,
                        });
                    }
                }
            }
        }

        if let Some(section) = current_section.take() {
            sections.push(section);
        }

        Ok(DocumentNode {
            path: PathBuf::from(path),
            title,
            sections,
            code_blocks,
            links,
            line_count,
        })
    }
}
