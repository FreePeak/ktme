use crate::error::Result;
use crate::storage::mapping::StorageManager;
use std::path::Path;

pub async fn add(service: Option<String>, url: Option<String>, file: Option<String>) -> Result<()> {
    // Auto-detect service name if not provided
    let service_name = if let Some(s) = service {
        s
    } else {
        tracing::info!("Auto-detecting service name...");
        let detector = crate::service_detector::ServiceDetector::new()?;
        detector.detect_with_ai_fallback().await?
    };

    tracing::info!("Adding mapping for service: {}", service_name);

    let storage = StorageManager::new()?;

    if let Some(location) = url {
        // Add URL mapping (typically Confluence)
        let location_clone = location.clone();
        storage.add_mapping(service_name.clone(), "confluence".to_string(), location)?;
        println!("✓ Added mapping: {} -> {}", service_name, location_clone);
    } else if let Some(location) = file {
        // Add file mapping (local markdown)
        let path = Path::new(&location);
        if !path.exists() {
            return Err(crate::error::KtmeError::Config(
                format!("File does not exist: {}", location)
            ));
        }
        let location_clone = location.clone();
        storage.add_mapping(service_name.clone(), "markdown".to_string(), location)?;
        println!("✓ Added mapping: {} -> {}", service_name, location_clone);
    } else {
        return Err(crate::error::KtmeError::Config(
            "Either --url or --file must be provided".to_string()
        ));
    }

    Ok(())
}

pub async fn list(service: Option<String>) -> Result<()> {
    tracing::info!("Listing service mappings");

    let storage = StorageManager::new()?;
    let mappings = storage.load_mappings()?;

    if mappings.services.is_empty() {
        println!("No service mappings found.");
        println!("Use 'ktme mapping add <service> --url <url>' or '--file <file>' to add mappings.");
        return Ok(());
    }

    if let Some(filter) = service {
        // Show specific service
        if let Some(service_mapping) = mappings.services.iter().find(|s| s.name == filter) {
            println!("Service: {}", service_mapping.name);
            if let Some(path) = &service_mapping.path {
                println!("  Path: {}", path);
            }
            println!("  Documentation:");
            for doc in &service_mapping.docs {
                println!("    - {} ({})", doc.location, doc.r#type);
            }
        } else {
            println!("No mapping found for service: {}", filter);
        }
    } else {
        // Show all services
        println!("Service Mappings:");
        println!("=================");
        for service_mapping in &mappings.services {
            println!("\nService: {}", service_mapping.name);
            if let Some(path) = &service_mapping.path {
                println!("  Path: {}", path);
            }
            println!("  Documentation:");
            for doc in &service_mapping.docs {
                println!("    - {} ({})", doc.location, doc.r#type);
            }
        }
    }

    Ok(())
}

pub async fn get(service: String) -> Result<()> {
    tracing::info!("Getting mapping for service: {}", service);

    let storage = StorageManager::new()?;
    let mapping = storage.get_mapping(&service)?;

    println!("Service: {}", mapping.name);
    if let Some(path) = mapping.path {
        println!("Path: {}", path);
    }

    if mapping.docs.is_empty() {
        println!("No documentation locations mapped for this service.");
    } else {
        println!("Documentation locations:");
        for doc in mapping.docs {
            println!("  - {} ({})", doc.location, doc.r#type);
        }
    }

    Ok(())
}

pub async fn remove(service: String) -> Result<()> {
    tracing::info!("Removing mapping for service: {}", service);

    let storage = StorageManager::new()?;

    // Check if mapping exists
    let mappings = storage.load_mappings()?;
    if !mappings.services.iter().any(|s| s.name == service) {
        return Err(crate::error::KtmeError::MappingNotFound(service));
    }

    storage.remove_mapping(&service)?;
    println!("✓ Removed mapping for service: {}", service);

    Ok(())
}

pub async fn discover(directory: String) -> Result<()> {
    tracing::info!("Discovering services in directory: {}", directory);

    let storage = StorageManager::new()?;
    let discovered = storage.discover_services(&directory)?;

    if discovered.is_empty() {
        println!("No services discovered in directory: {}", directory);
        return Ok(());
    }

    println!("Discovered services:");
    for service in discovered {
        println!("  - {}", service.name);
        println!("    Path: {}", service.path);
    }

    // TODO: Ask user if they want to add these services
    println!("\nUse 'ktme mapping add <service> --file <path>' to add documentation mappings.");

    Ok(())
}

pub async fn edit() -> Result<()> {
    tracing::info!("Opening mappings file for editing");

    let storage = StorageManager::new()?;
    let mappings_file = storage.mappings_file_path();

    println!("Opening mappings file for editing: {}", mappings_file.display());

    // Try to open with default editor
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&mappings_file)
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(editor) = std::env::var("EDITOR") {
            std::process::Command::new(editor)
                .arg(&mappings_file)
                .spawn()?;
        } else {
            println!("Set EDITOR environment variable to use this command");
        }
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("notepad")
            .arg(&mappings_file)
            .spawn()?;
    }

    Ok(())
}
