use crate::config::Config;
use crate::error::{KtmeError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mappings {
    pub version: String,
    pub last_updated: DateTime<Utc>,
    pub services: Vec<ServiceMapping>,
}

impl Default for Mappings {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            last_updated: Utc::now(),
            services: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMapping {
    pub name: String,
    pub path: Option<String>,
    pub docs: Vec<DocumentLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLocation {
    pub r#type: String,
    pub location: String,
}

pub struct StorageManager {
    mappings_file: PathBuf,
}

impl StorageManager {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let mappings_file = if let Some(path) = config.storage.mappings_file {
            path
        } else {
            let config_dir = Config::config_dir()?;
            config_dir.join("mappings.toml")
        };

        Ok(Self { mappings_file })
    }

    pub fn load_mappings(&self) -> Result<Mappings> {
        if !self.mappings_file.exists() {
            return Ok(Mappings::default());
        }

        let content = fs::read_to_string(&self.mappings_file)?;
        let mappings: Mappings = toml::from_str(&content)?;

        Ok(mappings)
    }

    pub fn save_mappings(&self, mappings: &Mappings) -> Result<()> {
        if let Some(parent) = self.mappings_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(mappings)?;
        fs::write(&self.mappings_file, content)?;

        tracing::info!("Mappings saved to {}", self.mappings_file.display());

        Ok(())
    }

    pub fn add_mapping(&self, service: String, doc_type: String, location: String) -> Result<()> {
        let mut mappings = self.load_mappings()?;

        if let Some(existing) = mappings.services.iter_mut().find(|s| s.name == service) {
            existing.docs.push(DocumentLocation {
                r#type: doc_type,
                location,
            });
        } else {
            mappings.services.push(ServiceMapping {
                name: service,
                path: None,
                docs: vec![DocumentLocation {
                    r#type: doc_type,
                    location,
                }],
            });
        }

        mappings.last_updated = Utc::now();
        self.save_mappings(&mappings)?;

        Ok(())
    }

    pub fn get_mapping(&self, service: &str) -> Result<ServiceMapping> {
        let mappings = self.load_mappings()?;

        mappings
            .services
            .into_iter()
            .find(|s| s.name == service)
            .ok_or_else(|| KtmeError::MappingNotFound(service.to_string()))
    }

    pub fn remove_mapping(&self, service: &str) -> Result<()> {
        let mut mappings = self.load_mappings()?;

        mappings.services.retain(|s| s.name != service);
        mappings.last_updated = Utc::now();

        self.save_mappings(&mappings)?;

        Ok(())
    }

    
    pub fn discover_services(&self, directory: &str) -> Result<Vec<crate::storage::discovery::DiscoveredService>> {
        use crate::storage::discovery::ServiceDiscovery;
        let discovery = ServiceDiscovery::new(directory.to_string());
        discovery.discover()
    }

    pub fn mappings_file_path(&self) -> PathBuf {
        self.mappings_file.clone()
    }

    pub fn list_services(&self) -> Result<Vec<String>> {
        let mappings = self.load_mappings()?;
        Ok(mappings.services.iter().map(|s| s.name.clone()).collect())
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new().expect("Failed to create StorageManager")
    }
}
