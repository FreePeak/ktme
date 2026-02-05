use std::collections::HashMap;

pub struct ReferenceDatabase {
    pub mappings: HashMap<String, String>,
}

impl ReferenceDatabase {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();

        mappings.insert(
            "tokio".to_string(),
            "https://docs.rs/tokio/latest/tokio/".to_string(),
        );
        mappings.insert("serde".to_string(), "https://serde.rs/".to_string());
        mappings.insert(
            "reqwest".to_string(),
            "https://docs.rs/reqwest/latest/reqwest/".to_string(),
        );
        mappings.insert(
            "tracing".to_string(),
            "https://docs.rs/tracing/latest/tracing/".to_string(),
        );
        mappings.insert(
            "clap".to_string(),
            "https://docs.rs/clap/latest/clap/".to_string(),
        );
        mappings.insert(
            "anyhow".to_string(),
            "https://docs.rs/anyhow/latest/anyhow/".to_string(),
        );
        mappings.insert(
            "thiserror".to_string(),
            "https://docs.rs/thiserror/latest/thiserror/".to_string(),
        );
        mappings.insert(
            "rusqlite".to_string(),
            "https://docs.rs/rusqlite/latest/rusqlite/".to_string(),
        );
        mappings.insert(
            "chrono".to_string(),
            "https://docs.rs/chrono/latest/chrono/".to_string(),
        );
        mappings.insert(
            "walkdir".to_string(),
            "https://docs.rs/walkdir/latest/walkdir/".to_string(),
        );

        ReferenceDatabase { mappings }
    }

    pub fn lookup(&self, dep_name: &str) -> Option<&str> {
        self.mappings.get(dep_name).map(|s| s.as_str())
    }

    pub fn generate_references_md(&self, tech_stack: &super::tech_detector::TechStack) -> String {
        let mut md = String::from("# Reference Documentation\n\n");

        md.push_str("## Dependencies\n\n");
        md.push_str("| Dependency | Version | Documentation |\n");
        md.push_str("|------------|---------|---------------|\n");

        for dep in &tech_stack.dependencies {
            let doc_url = dep.doc_url.as_deref().unwrap_or("-");
            md.push_str(&format!(
                "| `{}` | {} | [docs]({}) |\n",
                dep.name, dep.version, doc_url
            ));
        }

        md.push_str("\n## Architecture Patterns\n\n");
        for pattern in &tech_stack.patterns {
            md.push_str(&format!(
                "### {}\n\n{}\n\n",
                pattern.name, pattern.description
            ));
        }

        md
    }
}

impl Default for ReferenceDatabase {
    fn default() -> Self {
        Self::new()
    }
}
