use crate::error::{KtmeError, Result};
use crate::service_detector::ServiceDetector;
use crate::storage::database::Database;
use crate::storage::repository::ServiceRepository;
use std::fs;
use std::path::{Path, PathBuf};

/// Initialize project documentation and knowledge graph
pub async fn execute(
    path: Option<String>,
    service_name: Option<String>,
    force: bool,
) -> Result<()> {
    let project_path = path.as_deref().unwrap_or(".");
    let project_dir = PathBuf::from(project_path);

    tracing::info!("Initializing documentation for project at: {}", project_path);

    // Check if already initialized (unless force flag is set)
    let docs_dir = project_dir.join("docs");
    if docs_dir.exists() && !force {
        println!("⚠️  Documentation directory already exists at: {}", docs_dir.display());
        println!("   Use --force to re-initialize");
        return Ok(());
    }

    // Detect or use provided service name
    let service = if let Some(name) = service_name {
        name
    } else {
        println!("🔍 Detecting service name...");
        let detector = ServiceDetector::from_directory(project_dir.clone());
        detector
            .detect_service_name()
            .await?
    };

    println!("📦 Service: {}", service);

    // Create documentation directory structure
    println!("📁 Creating documentation structure...");
    create_docs_structure(&project_dir, force)?;

    // Initialize database and knowledge graph
    println!("🗄️  Initializing knowledge graph database...");
    initialize_knowledge_graph(&service, &project_dir)?;

    // Create initial documentation
    println!("📝 Creating initial documentation files...");
    create_initial_docs(&docs_dir, &service)?;

    println!("\n✅ Initialization complete!");
    println!("\n📚 Documentation structure created at: {}", docs_dir.display());
    println!("   - README.md: Project overview");
    println!("   - architecture.md: Architecture documentation");
    println!("   - api.md: API documentation");
    println!("   - changelog.md: Change log");
    println!("\n💡 Next steps:");
    println!("   1. Run 'ktme generate --service {} --staged' to document your changes", service);
    println!("   2. Run 'ktme mapping add {}' to link documentation", service);
    println!("   3. Run 'ktme mcp start' to enable AI agent integration");

    Ok(())
}

/// Create the documentation directory structure
fn create_docs_structure(project_dir: &Path, force: bool) -> Result<()> {
    let docs_dir = project_dir.join("docs");

    if docs_dir.exists() {
        if force {
            tracing::warn!("Docs directory exists, force flag set - proceeding");
        } else {
            return Ok(());
        }
    } else {
        fs::create_dir_all(&docs_dir)
            .map_err(|e| KtmeError::Io(e))?;
    }

    // Create subdirectories
    let subdirs = ["api", "guides", "examples"];
    for subdir in subdirs {
        let subdir_path = docs_dir.join(subdir);
        if !subdir_path.exists() {
            fs::create_dir_all(&subdir_path)
                .map_err(|e| KtmeError::Io(e))?;
        }
    }

    Ok(())
}

/// Initialize the knowledge graph in the database
fn initialize_knowledge_graph(service: &str, project_dir: &Path) -> Result<()> {
    // Get or create database
    let db = Database::new(None)?;
    let service_repo = ServiceRepository::new(db);

    // Check if service already exists
    if let Some(existing) = service_repo.get_by_name(service)? {
        tracing::info!("Service '{}' already exists with id: {}", service, existing.id);
        println!("   Service '{}' already registered", service);
        return Ok(());
    }

    // Create service entry
    let project_path = project_dir
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(String::from));

    let description = format!("Documentation and knowledge graph for {}", service);
    
    service_repo.create(
        service,
        project_path.as_deref(),
        Some(&description),
    )?;

    println!("   Service '{}' registered in knowledge graph", service);
    Ok(())
}

/// Create initial documentation files
fn create_initial_docs(docs_dir: &Path, service: &str) -> Result<()> {
    // README.md
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
        fs::write(&readme_path, readme_content)
            .map_err(|e| KtmeError::Io(e))?;
    }

    // architecture.md
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
        fs::write(&arch_path, arch_content)
            .map_err(|e| KtmeError::Io(e))?;
    }

    // api.md
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
        fs::write(&api_path, api_content)
            .map_err(|e| KtmeError::Io(e))?;
    }

    // changelog.md
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
        fs::write(&changelog_path, changelog_content)
            .map_err(|e| KtmeError::Io(e))?;
    }

    Ok(())
}
