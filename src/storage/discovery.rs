use crate::error::Result;
use walkdir::WalkDir;

pub struct ServiceDiscovery {
    base_path: String,
}

impl ServiceDiscovery {
    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }

    pub fn discover(&self) -> Result<Vec<DiscoveredService>> {
        tracing::info!("Discovering services in: {}", self.base_path);

        let mut services = Vec::new();

        for entry in WalkDir::new(&self.base_path)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let file_name = entry.file_name().to_string_lossy();
                if file_name == "README.md" {
                    if let Some(parent) = entry.path().parent() {
                        if let Some(service_name) = parent.file_name() {
                            services.push(DiscoveredService {
                                name: service_name.to_string_lossy().to_string(),
                                path: parent.to_string_lossy().to_string(),
                                docs: vec![entry.path().to_string_lossy().to_string()],
                            });
                        }
                    }
                }
            }
        }

        Ok(services)
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredService {
    pub name: String,
    pub path: String,
    pub docs: Vec<String>,
}
