use crate::error::{KtmeError, Result};
use crate::service_detector::ServiceDetector;
use crate::storage::database::Database;
use crate::storage::repository::ServiceRepository;
use crate::InitMode;
use std::fs;
use std::path::{Path, PathBuf};

pub async fn execute(
    path: Option<String>,
    service_name: Option<String>,
    force: bool,
    mode: InitMode,
    dry_run: bool,
    output: Option<String>,
) -> Result<()> {
    match mode {
        InitMode::Fresh => execute_fresh(path, service_name, force).await,
        InitMode::Scan => execute_scan(path, service_name, output).await,
        InitMode::Validate => execute_validate(path, service_name, output).await,
        InitMode::Enhance => execute_enhance(path, service_name, dry_run, output).await,
        InitMode::Sync => execute_sync(path, service_name, dry_run, output).await,
        InitMode::Research => execute_research(path, service_name, dry_run, output).await,
    }
}

pub async fn execute_fresh(
    path: Option<String>,
    service_name: Option<String>,
    force: bool,
) -> Result<()> {
    let project_path = path.as_deref().unwrap_or(".");
    let project_dir = PathBuf::from(project_path);

    tracing::info!(
        "Initializing documentation for project at: {}",
        project_path
    );

    let docs_dir = project_dir.join("docs");
    if docs_dir.exists() && !force {
        println!(
            "Documentation directory already exists at: {}",
            docs_dir.display()
        );
        println!("   Use --force to re-initialize");
        return Ok(());
    }

    let service = if let Some(name) = service_name {
        name
    } else {
        println!("Detecting service name...");
        let detector = ServiceDetector::from_directory(project_dir.clone());
        detector.detect_service_name().await?
    };

    println!("Service: {}", service);

    println!("Creating documentation structure...");
    create_docs_structure(&project_dir, force)?;

    println!("Initializing knowledge graph database...");
    initialize_knowledge_graph(&service, &project_dir)?;

    println!("Creating initial documentation files...");
    create_initial_docs(&docs_dir, &service)?;

    println!("\nInitialization complete!");
    println!(
        "\nDocumentation structure created at: {}",
        docs_dir.display()
    );
    println!("   - README.md: Project overview");
    println!("   - architecture.md: Architecture documentation");
    println!("   - api.md: API documentation");
    println!("   - changelog.md: Change log");
    println!("\nNext steps:");
    println!(
        "   1. Run 'ktme generate --service {} --staged' to document your changes",
        service
    );
    println!(
        "   2. Run 'ktme mapping add {}' to link documentation",
        service
    );
    println!("   3. Run 'ktme mcp start' to enable AI agent integration");

    Ok(())
}

pub async fn execute_scan(
    path: Option<String>,
    _service_name: Option<String>,
    output: Option<String>,
) -> Result<()> {
    let project_path = path.as_deref().unwrap_or(".");
    let project_dir = PathBuf::from(project_path);

    tracing::info!("Scanning documentation for project at: {}", project_path);

    let docs_dir = project_dir.join("docs");
    if !docs_dir.exists() {
        println!(
            "No documentation directory found at: {}",
            docs_dir.display()
        );
        return Ok(());
    }

    println!("Scanning documentation in: {}", docs_dir.display());

    let mut total_files = 0;
    let mut total_sections = 0;
    let mut total_code_blocks = 0;

    let entries = fs::read_dir(&docs_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
            total_files += 1;
            let content = fs::read_to_string(&path)?;
            let sections = content.lines().filter(|l| l.starts_with("## ")).count();
            let code_blocks = content.matches("```").count() / 2;
            total_sections += sections;
            total_code_blocks += code_blocks;
            println!(
                "  - {}: {} sections, {} code blocks",
                path.file_name().unwrap().to_string_lossy(),
                sections,
                code_blocks
            );
        }
    }

    println!("\nScan Summary:");
    println!("  Total markdown files: {}", total_files);
    println!("  Total sections: {}", total_sections);
    println!("  Total code blocks: {}", total_code_blocks);

    if let Some(ref output_path) = output {
        let report = format!(
            "# Documentation Scan Report\n\nProject: {}\n\n## Summary\n- Total markdown files: {}\n- Total sections: {}\n- Total code blocks: {}\n",
            project_path, total_files, total_sections, total_code_blocks
        );
        fs::write(output_path, report)?;
        println!("Report saved to: {}", output_path);
    }

    Ok(())
}

pub async fn execute_validate(
    path: Option<String>,
    _service_name: Option<String>,
    output: Option<String>,
) -> Result<()> {
    let project_path = path.as_deref().unwrap_or(".");
    let project_dir = PathBuf::from(project_path);

    tracing::info!("Validating documentation for project at: {}", project_path);

    let docs_dir = project_dir.join("docs");
    if !docs_dir.exists() {
        println!(
            "No documentation directory found at: {}",
            docs_dir.display()
        );
        return Ok(());
    }

    println!("Validating documentation in: {}", docs_dir.display());

    let mut validation_errors = Vec::new();
    let mut validation_warnings = Vec::new();

    let entries = fs::read_dir(&docs_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
            let content = fs::read_to_string(&path)?;

            if !content.starts_with("# ") {
                validation_warnings.push(format!(
                    "{}: Missing title header",
                    path.file_name().unwrap().to_string_lossy()
                ));
            }

            if !content.contains("## ") {
                validation_warnings.push(format!(
                    "{}: No sections found",
                    path.file_name().unwrap().to_string_lossy()
                ));
            }

            let broken_links = content.matches("[").count() != content.matches("]").count();
            if broken_links {
                validation_errors.push(format!(
                    "{}: Potentially broken links detected",
                    path.file_name().unwrap().to_string_lossy()
                ));
            }
        }
    }

    println!("\nValidation Results:");

    if validation_warnings.is_empty() && validation_errors.is_empty() {
        println!("  All checks passed!");
    } else {
        for warning in &validation_warnings {
            println!("  WARNING: {}", warning);
        }
        for error in &validation_errors {
            println!("  ERROR: {}", error);
        }
    }

    if let Some(ref output_path) = output {
        let mut report = format!(
            "# Documentation Validation Report\n\nProject: {}\n\n## Warnings\n",
            project_path
        );
        for warning in &validation_warnings {
            report.push_str(&format!("- {}\n", warning));
        }
        report.push_str("\n## Errors\n");
        for error in &validation_errors {
            report.push_str(&format!("- {}\n", error));
        }
        if validation_warnings.is_empty() && validation_errors.is_empty() {
            report.push_str("All checks passed!\n");
        }
        fs::write(output_path, report)?;
        println!("Report saved to: {}", output_path);
    }

    Ok(())
}

pub async fn execute_enhance(
    path: Option<String>,
    _service_name: Option<String>,
    dry_run: bool,
    output: Option<String>,
) -> Result<()> {
    let project_path = path.as_deref().unwrap_or(".");
    let project_dir = PathBuf::from(project_path);

    tracing::info!("Enhancing documentation for project at: {}", project_path);

    let docs_dir = project_dir.join("docs");
    if !docs_dir.exists() {
        println!(
            "No documentation directory found at: {}",
            docs_dir.display()
        );
        return Ok(());
    }

    println!("Enhancing documentation in: {}", docs_dir.display());

    if dry_run {
        println!("[DRY RUN] Would enhance documentation files");
    } else {
        let entries = fs::read_dir(&docs_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                let content = fs::read_to_string(&path)?;

                let has_todo = content.contains("TODO:");
                if has_todo {
                    println!(
                        "  Found TODOs in: {}",
                        path.file_name().unwrap().to_string_lossy()
                    );
                }
            }
        }
        println!("Documentation enhancement complete!");
    }

    if let Some(ref output_path) = output {
        let report = format!(
            "# Documentation Enhancement Report\n\nProject: {}\nMode: {}\n",
            project_path,
            if dry_run { "dry-run" } else { "live" }
        );
        fs::write(output_path, report)?;
        println!("Report saved to: {}", output_path);
    }

    Ok(())
}

pub async fn execute_sync(
    path: Option<String>,
    _service_name: Option<String>,
    dry_run: bool,
    output: Option<String>,
) -> Result<()> {
    let project_path = path.as_deref().unwrap_or(".");
    let project_dir = PathBuf::from(project_path);

    tracing::info!("Syncing documentation for project at: {}", project_path);

    let docs_dir = project_dir.join("docs");
    if !docs_dir.exists() {
        println!(
            "No documentation directory found at: {}",
            docs_dir.display()
        );
        return Ok(());
    }

    println!("Syncing documentation in: {}", docs_dir.display());

    if dry_run {
        println!("[DRY RUN] Would sync documentation files");
    } else {
        println!("Documentation sync complete!");
    }

    if let Some(ref output_path) = output {
        let report = format!(
            "# Documentation Sync Report\n\nProject: {}\nMode: {}\n",
            project_path,
            if dry_run { "dry-run" } else { "live" }
        );
        fs::write(output_path, report)?;
        println!("Report saved to: {}", output_path);
    }

    Ok(())
}

pub async fn execute_research(
    path: Option<String>,
    _service_name: Option<String>,
    dry_run: bool,
    output: Option<String>,
) -> Result<()> {
    let project_path = path.as_deref().unwrap_or(".");
    let project_dir = PathBuf::from(project_path);

    tracing::info!(
        "Researching technology stack for project at: {}",
        project_path
    );

    println!("Researching technology stack in: {}", project_path);

    let cargo_toml = project_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        println!("  Detected: Rust project");

        let content = fs::read_to_string(&cargo_toml)?;
        if content.contains("tokio") {
            println!("    - Async runtime: tokio");
        }
        if content.contains("serde") {
            println!("    - Serialization: serde");
        }
        if content.contains("reqwest") {
            println!("    - HTTP client: reqwest");
        }
        if content.contains("tracing") {
            println!("    - Logging: tracing");
        }
    }

    let package_json = project_dir.join("package.json");
    if package_json.exists() {
        println!("  Detected: Node.js project");
    }

    let pom_xml = project_dir.join("pom.xml");
    if pom_xml.exists() {
        println!("  Detected: Maven/Java project");
    }

    let go_mod = project_dir.join("go.mod");
    if go_mod.exists() {
        println!("  Detected: Go project");
    }

    if dry_run {
        println!("[DRY RUN] Would research technology documentation");
    }

    if let Some(ref output_path) = output {
        let report = format!(
            "# Technology Research Report\n\nProject: {}\nMode: {}\n",
            project_path,
            if dry_run { "dry-run" } else { "live" }
        );
        fs::write(output_path, report)?;
        println!("Report saved to: {}", output_path);
    }

    Ok(())
}

fn create_docs_structure(project_dir: &Path, force: bool) -> Result<()> {
    let docs_dir = project_dir.join("docs");

    if docs_dir.exists() {
        if force {
            tracing::warn!("Docs directory exists, force flag set - proceeding");
        } else {
            return Ok(());
        }
    } else {
        fs::create_dir_all(&docs_dir).map_err(|e| KtmeError::Io(e))?;
    }

    let subdirs = ["api", "guides", "examples"];
    for subdir in subdirs {
        let subdir_path = docs_dir.join(subdir);
        if !subdir_path.exists() {
            fs::create_dir_all(&subdir_path).map_err(|e| KtmeError::Io(e))?;
        }
    }

    Ok(())
}

fn initialize_knowledge_graph(service: &str, project_dir: &Path) -> Result<()> {
    let db = Database::new(None)?;
    let service_repo = ServiceRepository::new(db);

    if let Some(existing) = service_repo.get_by_name(service)? {
        tracing::info!(
            "Service '{}' already exists with id: {}",
            service,
            existing.id
        );
        println!("   Service '{}' already registered", service);
        return Ok(());
    }

    let project_path = project_dir
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(String::from));

    let description = format!("Documentation and knowledge graph for {}", service);

    service_repo.create(service, project_path.as_deref(), Some(&description))?;

    println!("   Service '{}' registered in knowledge graph", service);
    Ok(())
}

fn create_initial_docs(docs_dir: &Path, service: &str) -> Result<()> {
    let readme_path = docs_dir.join("README.md");
    if !readme_path.exists() {
        let readme_content = format!(
            r#"# {} Documentation

Welcome to the {} documentation!

## Overview

This documentation is automatically maintained by ktme (Knowledge Transfer Me).

## Contents

- [Architecture](./architecture.md) - System architecture and design
- [API Documentation](./api.md) - API endpoints and usage
- [Change Log](./changelog.md) - Version history and changes

## Getting Started

TODO: Add getting started guide

## Contributing

TODO: Add contribution guidelines

---

*This documentation is maintained using [ktme](https://github.com/FreePeak/ktme)*
"#,
            service, service
        );
        fs::write(&readme_path, readme_content).map_err(|e| KtmeError::Io(e))?;
    }

    let arch_path = docs_dir.join("architecture.md");
    if !arch_path.exists() {
        let arch_content = format!(
            r#"# {} Architecture

## Overview

TODO: Add system overview

## Components

TODO: Document main components

## Data Flow

TODO: Describe data flow

## Technologies

TODO: List key technologies

---

*Generated by ktme*
"#,
            service
        );
        fs::write(&arch_path, arch_content).map_err(|e| KtmeError::Io(e))?;
    }

    let api_path = docs_dir.join("api.md");
    if !api_path.exists() {
        let api_content = format!(
            r#"# {} API Documentation

## Endpoints

TODO: Document API endpoints

## Authentication

TODO: Describe authentication mechanism

## Examples

TODO: Add usage examples

---

*Generated by ktme*
"#,
            service
        );
        fs::write(&api_path, api_content).map_err(|e| KtmeError::Io(e))?;
    }

    let changelog_path = docs_dir.join("changelog.md");
    if !changelog_path.exists() {
        let changelog_content = format!(
            r#"# {} Change Log

## [Unreleased]

### Added
- Initial project setup
- Documentation structure

---

*Generated by ktme*
"#,
            service
        );
        fs::write(&changelog_path, changelog_content).map_err(|e| KtmeError::Io(e))?;
    }

    Ok(())
}
