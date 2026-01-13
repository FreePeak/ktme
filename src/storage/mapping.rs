use crate::config::Config;
use crate::error::{KtmeError, Result};
use crate::storage::database::Database;
use crate::storage::models::{FeatureType, SearchQuery, SearchResult};
use crate::storage::repository::{DocumentMappingRepository, FeatureRepository, ServiceRepository};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSearchResult {
    pub name: String,
    pub path: Option<String>,
    pub description: Option<String>,
    pub docs: Vec<String>,
    pub relevance_score: f32,
}

pub struct StorageManager {
    mappings_file: PathBuf,
    database: Option<Database>,
    pub use_sqlite: bool,
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

        let use_sqlite = config.storage.use_sqlite;
        let database = if use_sqlite {
            Some(Database::new(config.storage.database_file)?)
        } else {
            None
        };

        Ok(Self {
            mappings_file,
            database,
            use_sqlite,
        })
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
        if self.use_sqlite {
            // Use SQLite when enabled
            if let Some(ref db) = self.database {
                let service_repo = ServiceRepository::new(db.clone());
                let mapping_repo = DocumentMappingRepository::new(db.clone());

                // Create or get service
                let service_entity = match service_repo.get_by_name(&service)? {
                    Some(s) => s,
                    None => service_repo.create(
                        &service,
                        Some(&location),
                        Some(&format!("Service for {}", service)),
                    )?,
                };

                // Add document mapping
                mapping_repo.add(
                    service_entity.id,
                    &doc_type,
                    &location,
                    Some(&format!("Documentation for {}", service)),
                    None,
                    true, // Set as primary mapping
                )?;

                tracing::info!("Added mapping to SQLite: {} -> {}", service, location);
            } else {
                return Err(KtmeError::Storage("SQLite not initialized".to_string()));
            }
        } else {
            // Use TOML file storage
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
        }

        Ok(())
    }

    pub fn get_mapping(&self, service: &str) -> Result<ServiceMapping> {
        if self.use_sqlite {
            // Use SQLite when enabled
            if let Some(ref db) = self.database {
                let service_repo = ServiceRepository::new(db.clone());
                let mapping_repo = DocumentMappingRepository::new(db.clone());

                let service_entity = service_repo
                    .get_by_name(service)?
                    .ok_or_else(|| KtmeError::MappingNotFound(service.to_string()))?;

                let mappings = mapping_repo.get_for_service(service_entity.id)?;
                let docs = mappings
                    .into_iter()
                    .map(|m| DocumentLocation {
                        r#type: m.provider,
                        location: m.location,
                    })
                    .collect();

                Ok(ServiceMapping {
                    name: service_entity.name,
                    path: service_entity.path,
                    docs,
                })
            } else {
                Err(KtmeError::Storage("SQLite not initialized".to_string()))
            }
        } else {
            // Use TOML file storage
            let mappings = self.load_mappings()?;

            mappings
                .services
                .into_iter()
                .find(|s| s.name == service)
                .ok_or_else(|| KtmeError::MappingNotFound(service.to_string()))
        }
    }

    pub fn remove_mapping(&self, service: &str) -> Result<()> {
        let mut mappings = self.load_mappings()?;

        mappings.services.retain(|s| s.name != service);
        mappings.last_updated = Utc::now();

        self.save_mappings(&mappings)?;

        Ok(())
    }

    pub fn discover_services(
        &self,
        directory: &str,
    ) -> Result<Vec<crate::storage::discovery::DiscoveredService>> {
        use crate::storage::discovery::ServiceDiscovery;
        let discovery = ServiceDiscovery::new(directory.to_string());
        discovery.discover()
    }

    pub fn mappings_file_path(&self) -> PathBuf {
        self.mappings_file.clone()
    }

    pub fn list_services(&self) -> Result<Vec<String>> {
        if self.use_sqlite {
            if let Some(ref db) = self.database {
                let service_repo = ServiceRepository::new(db.clone());
                service_repo.list_all_names()
            } else {
                Ok(vec![])
            }
        } else {
            let mappings = self.load_mappings()?;
            Ok(mappings.services.iter().map(|s| s.name.clone()).collect())
        }
    }

    /// Search services by name, feature, or keyword
    pub fn search_services(&self, query: &str) -> Result<Vec<ServiceSearchResult>> {
        if self.use_sqlite {
            if let Some(ref db) = self.database {
                let service_repo = ServiceRepository::new(db.clone());
                let mapping_repo = DocumentMappingRepository::new(db.clone());

                let all_services = service_repo.list()?;
                let mut results = Vec::new();

                for service in all_services {
                    let relevance_score = self.calculate_relevance(&service, query);
                    if relevance_score > 0.0 {
                        let mappings = mapping_repo.get_for_service(service.id)?;
                        let docs: Vec<String> = mappings
                            .into_iter()
                            .map(|m| format!("{}: {}", m.provider, m.location))
                            .collect();

                        results.push(ServiceSearchResult {
                            name: service.name,
                            path: service.path,
                            description: service.description,
                            docs,
                            relevance_score,
                        });
                    }
                }

                // Sort by relevance score
                results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
                Ok(results)
            } else {
                Err(KtmeError::Storage("SQLite not initialized".to_string()))
            }
        } else {
            // Simple text search in TOML mappings
            let mappings = self.load_mappings()?;
            let mut results = Vec::new();

            for service in mappings.services {
                let relevance_score = self.calculate_service_relevance(&service, query);
                if relevance_score > 0.0 {
                    let docs: Vec<String> = service
                        .docs
                        .into_iter()
                        .map(|d| format!("{}: {}", d.r#type, d.location))
                        .collect();

                    results.push(ServiceSearchResult {
                        name: service.name,
                        path: service.path,
                        description: None,
                        docs,
                        relevance_score,
                    });
                }
            }

            results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
            Ok(results)
        }
    }

    /// Search services by feature (looking for specific functionality in documentation)
    pub fn search_by_feature(&self, feature: &str) -> Result<Vec<ServiceSearchResult>> {
        self.search_services(feature)
    }

    /// Search services by keyword (more flexible search)
    pub fn search_by_keyword(&self, keyword: &str) -> Result<Vec<ServiceSearchResult>> {
        self.search_services(keyword)
    }

    fn calculate_relevance(&self, service: &crate::storage::models::Service, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let mut score = 0.0;

        // Exact name match
        if service.name.to_lowercase().contains(&query_lower) {
            score += 10.0;
        }

        // Partial name match
        if service.name.to_lowercase().starts_with(&query_lower) {
            score += 5.0;
        }

        // Description match
        if let Some(ref desc) = service.description {
            if desc.to_lowercase().contains(&query_lower) {
                score += 3.0;
            }
        }

        // Path match
        if let Some(ref path) = service.path {
            if path.to_lowercase().contains(&query_lower) {
                score += 2.0;
            }
        }

        score
    }

    fn calculate_service_relevance(&self, service: &ServiceMapping, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let mut score = 0.0;

        // Name match
        if service.name.to_lowercase().contains(&query_lower) {
            score += 10.0;
        }

        // Path match
        if let Some(ref path) = service.path {
            if path.to_lowercase().contains(&query_lower) {
                score += 2.0;
            }
        }

        // Document location matches
        for doc in &service.docs {
            if doc.location.to_lowercase().contains(&query_lower) {
                score += 1.0;
            }
        }

        score
    }

    /// Initialize the database with SQLite enabled
    pub fn initialize_database(&self) -> Result<()> {
        if self.use_sqlite {
            if self.database.is_some() {
                // Database is already initialized via migrations in Database::new()
                tracing::info!("SQLite database initialized successfully");
                Ok(())
            } else {
                Err(KtmeError::Storage("SQLite not initialized".to_string()))
            }
        } else {
            Err(KtmeError::Storage(
                "SQLite not enabled in configuration".to_string(),
            ))
        }
    }

    /// Get database statistics (only available when SQLite is enabled)
    pub fn get_database_stats(&self) -> Result<crate::storage::database::DatabaseStats> {
        if let Some(ref db) = self.database {
            db.stats()
        } else {
            Err(KtmeError::Storage("Database not initialized".to_string()))
        }
    }

    // ============================================================================
    // Feature Management Methods (SQLite only)
    // ============================================================================

    /// Create a new feature
    pub fn create_feature(
        &self,
        service_name: &str,
        feature_name: &str,
        description: Option<&str>,
        feature_type: FeatureType,
        tags: Vec<String>,
        metadata: serde_json::Value,
    ) -> Result<crate::storage::models::Feature> {
        if !self.use_sqlite {
            return Err(KtmeError::Storage(
                "Features require SQLite storage".to_string(),
            ));
        }

        let db = self
            .database
            .as_ref()
            .ok_or_else(|| KtmeError::Storage("Database not initialized".to_string()))?;
        let service_repo = ServiceRepository::new(db.clone());
        let feature_repo = FeatureRepository::new(db.clone());

        // Find or create service
        let service = match service_repo.get_by_name(service_name)? {
            Some(service) => service,
            None => service_repo.create(
                service_name,
                None,
                Some(&format!("Auto-created service for {}", service_name)),
            )?,
        };

        // Create feature with UUID
        let feature_id = Uuid::new_v4().to_string();
        feature_repo.create(
            &feature_id,
            service.id,
            feature_name,
            description,
            feature_type,
            tags,
            metadata,
        )
    }

    /// Get features for a service
    pub fn get_service_features(
        &self,
        service_name: &str,
    ) -> Result<Vec<crate::storage::models::Feature>> {
        if !self.use_sqlite {
            return Err(KtmeError::Storage(
                "Features require SQLite storage".to_string(),
            ));
        }

        let db = self
            .database
            .as_ref()
            .ok_or_else(|| KtmeError::Storage("Database not initialized".to_string()))?;
        let service_repo = ServiceRepository::new(db.clone());
        let feature_repo = FeatureRepository::new(db.clone());

        let service = service_repo
            .get_by_name(service_name)?
            .ok_or_else(|| KtmeError::Storage(format!("Service '{}' not found", service_name)))?;

        feature_repo.list_by_service(service.id)
    }

    /// Search features across all services
    pub fn search_features(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        if !self.use_sqlite {
            return Err(KtmeError::Storage(
                "Feature search requires SQLite storage".to_string(),
            ));
        }

        let db = self
            .database
            .as_ref()
            .ok_or_else(|| KtmeError::Storage("Database not initialized".to_string()))?;
        let feature_repo = FeatureRepository::new(db.clone());

        feature_repo.search(query)
    }

    /// Simple feature search by text
    pub fn search_features_by_text(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: query.to_string(),
            service_ids: None,
            feature_types: None,
            content_types: None,
            limit,
            similarity_threshold: None,
            include_related: false,
            depth: None,
        };

        self.search_features(&search_query)
    }

    /// Update feature relevance score
    pub fn update_feature_relevance(&self, feature_id: &str, score: f64) -> Result<()> {
        if !self.use_sqlite {
            return Err(KtmeError::Storage(
                "Feature management requires SQLite storage".to_string(),
            ));
        }

        let db = self
            .database
            .as_ref()
            .ok_or_else(|| KtmeError::Storage("Database not initialized".to_string()))?;
        let feature_repo = FeatureRepository::new(db.clone());

        feature_repo.update_relevance_score(feature_id, score)
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new().expect("Failed to create StorageManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::models::FeatureType;

    #[test]
    fn test_storage_manager_feature_creation() {
        let storage = StorageManager::new().expect("Failed to create StorageManager");

        if !storage.use_sqlite {
            // Skip test if SQLite is not enabled
            return;
        }

        // Test feature creation
        let feature = storage
            .create_feature(
                "test-service",
                "Test Feature",
                Some("Test feature description"),
                FeatureType::Api,
                vec!["test".to_string(), "api".to_string()],
                serde_json::json!({"test": true}),
            )
            .expect("Failed to create feature");

        assert_eq!(feature.name, "Test Feature");
        assert_eq!(feature.feature_type, FeatureType::Api);
        assert_eq!(feature.tags, vec!["test", "api"]);
    }

    #[test]
    fn test_storage_manager_feature_search() {
        let storage = StorageManager::new().expect("Failed to create StorageManager");

        if !storage.use_sqlite {
            // Skip test if SQLite is not enabled
            return;
        }

        // Create test feature first
        storage
            .create_feature(
                "search-test",
                "Authentication Service",
                Some("User authentication and authorization"),
                FeatureType::Api,
                vec!["auth".to_string(), "security".to_string()],
                serde_json::json!({"endpoint": "/api/auth"}),
            )
            .expect("Failed to create feature");

        // Test search
        let results = storage
            .search_features_by_text("authentication", Some(10))
            .expect("Failed to search features");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].feature_name, "Authentication Service");
        assert_eq!(results[0].service_name, "search-test");
    }

    #[test]
    fn test_storage_manager_feature_list() {
        let storage = StorageManager::new().expect("Failed to create StorageManager");

        if !storage.use_sqlite {
            // Skip test if SQLite is not enabled
            return;
        }

        // Create multiple features
        storage
            .create_feature(
                "list-test",
                "Feature 1",
                None,
                FeatureType::Api,
                vec![],
                serde_json::json!({}),
            )
            .expect("Failed to create feature 1");

        storage
            .create_feature(
                "list-test",
                "Feature 2",
                None,
                FeatureType::Database,
                vec![],
                serde_json::json!({}),
            )
            .expect("Failed to create feature 2");

        // List features
        let features = storage
            .get_service_features("list-test")
            .expect("Failed to get service features");

        assert_eq!(features.len(), 2);
        assert!(features.iter().any(|f| f.name == "Feature 1"));
        assert!(features.iter().any(|f| f.name == "Feature 2"));
    }
}
