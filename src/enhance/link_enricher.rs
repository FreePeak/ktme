use std::path::Path;

pub struct BrokenLink {
    pub file: String,
    pub line: usize,
    pub link_text: String,
    pub target: String,
}

pub struct LinkEnricher;

impl LinkEnricher {
    pub fn find_broken_links(docs_dir: &Path) -> std::io::Result<Vec<BrokenLink>> {
        let mut broken_links = Vec::new();

        if let Ok(entries) = std::fs::read_dir(docs_dir) {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                    let content = std::fs::read_to_string(&path)?;

                    for (line_num, line) in content.lines().enumerate() {
                        if line.contains("[")
                            && line.contains("]")
                            && line.contains("(")
                            && line.contains(")")
                        {
                            if let Some(link_start) = line.find("(") {
                                if let Some(link_end) = line.find(")") {
                                    let link_target = &line[link_start + 1..link_end];
                                    if !link_target.starts_with("http")
                                        && !link_target.starts_with("#")
                                    {
                                        if !Path::new(link_target).exists() {
                                            broken_links.push(BrokenLink {
                                                file: path.to_string_lossy().to_string(),
                                                line: line_num + 1,
                                                link_text: line.to_string(),
                                                target: link_target.to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(broken_links)
    }

    pub fn enrich_links(content: &str) -> String {
        content.to_string()
    }
}
