use std::path::Path;

pub struct TechStack {
    pub language: String,
    pub runtime: Option<String>,
    pub dependencies: Vec<Dependency>,
    pub patterns: Vec<ArchitecturePattern>,
}

pub struct Dependency {
    pub name: String,
    pub version: String,
    pub doc_url: Option<String>,
}

pub struct ArchitecturePattern {
    pub name: String,
    pub description: String,
}

pub fn detect_tech_stack(project_dir: &Path) -> std::io::Result<TechStack> {
    let mut tech_stack = TechStack {
        language: String::new(),
        runtime: None,
        dependencies: Vec::new(),
        patterns: Vec::new(),
    };

    let cargo_toml = project_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        tech_stack.language = "Rust".to_string();
        tech_stack.runtime = Some("tokio".to_string());

        let content = std::fs::read_to_string(&cargo_toml)?;
        for line in content.lines() {
            if line.starts_with("tokio") {
                tech_stack.dependencies.push(Dependency {
                    name: "tokio".to_string(),
                    version: String::new(),
                    doc_url: Some("https://docs.rs/tokio".to_string()),
                });
            }
            if line.starts_with("serde") {
                tech_stack.dependencies.push(Dependency {
                    name: "serde".to_string(),
                    version: String::new(),
                    doc_url: Some("https://serde.rs".to_string()),
                });
            }
            if line.starts_with("reqwest") {
                tech_stack.dependencies.push(Dependency {
                    name: "reqwest".to_string(),
                    version: String::new(),
                    doc_url: Some("https://docs.rs/reqwest".to_string()),
                });
            }
        }

        if content.contains("async-trait") {
            tech_stack.patterns.push(ArchitecturePattern {
                name: "Async Trait Pattern".to_string(),
                description: "Uses async-trait for async trait methods".to_string(),
            });
        }
        if content.contains("tracing") {
            tech_stack.patterns.push(ArchitecturePattern {
                name: "Structured Logging".to_string(),
                description: "Uses tracing crate for structured logging".to_string(),
            });
        }
    }

    let package_json = project_dir.join("package.json");
    if package_json.exists() {
        tech_stack.language = "JavaScript/TypeScript".to_string();
    }

    let go_mod = project_dir.join("go.mod");
    if go_mod.exists() {
        tech_stack.language = "Go".to_string();
    }

    let pom_xml = project_dir.join("pom.xml");
    if pom_xml.exists() {
        tech_stack.language = "Java".to_string();
    }

    Ok(tech_stack)
}
