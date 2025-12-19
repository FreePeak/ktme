use super::database::Database;
use super::models::*;
use crate::error::{KtmeError, Result};
use chrono::{DateTime, Utc};
use rusqlite::params;

// ============================================================================
// Service Repository
// ============================================================================

pub struct ServiceRepository {
    db: Database,
}

impl ServiceRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, name: &str, path: Option<&str>, description: Option<&str>) -> Result<Service> {
        let conn = self.db.connection()?;

        conn.execute(
            "INSERT INTO services (name, path, description) VALUES (?1, ?2, ?3)",
            params![name, path, description],
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to create service: {}", e)))?;

        self.get_by_name(name)?
            .ok_or_else(|| KtmeError::Storage("Failed to retrieve created service".into()))
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<Service>> {
        let conn = self.db.connection()?;

        let result = conn.query_row(
            "SELECT id, name, path, description, created_at, updated_at
             FROM services WHERE id = ?1",
            params![id],
            |row| {
                Ok(Service {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        );

        match result {
            Ok(service) => Ok(Some(service)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(KtmeError::Storage(format!("Failed to get service: {}", e))),
        }
    }

    pub fn get_by_name(&self, name: &str) -> Result<Option<Service>> {
        let conn = self.db.connection()?;

        let result = conn.query_row(
            "SELECT id, name, path, description, created_at, updated_at
             FROM services WHERE name = ?1",
            params![name],
            |row| {
                Ok(Service {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        );

        match result {
            Ok(service) => Ok(Some(service)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(KtmeError::Storage(format!("Failed to get service: {}", e))),
        }
    }

    pub fn list(&self) -> Result<Vec<Service>> {
        let conn = self.db.connection()?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, path, description, created_at, updated_at
                 FROM services ORDER BY name",
            )
            .map_err(|e| KtmeError::Storage(format!("Failed to prepare query: {}", e)))?;

        let services = stmt
            .query_map([], |row| {
                Ok(Service {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| KtmeError::Storage(format!("Failed to query services: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| KtmeError::Storage(format!("Failed to collect services: {}", e)))?;

        Ok(services)
    }

    pub fn update(&self, id: i64, path: Option<&str>, description: Option<&str>) -> Result<()> {
        let conn = self.db.connection()?;

        conn.execute(
            "UPDATE services SET path = ?1, description = ?2, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?3",
            params![path, description, id],
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to update service: {}", e)))?;

        Ok(())
    }

    pub fn delete(&self, name: &str) -> Result<bool> {
        let conn = self.db.connection()?;

        let rows = conn
            .execute("DELETE FROM services WHERE name = ?1", params![name])
            .map_err(|e| KtmeError::Storage(format!("Failed to delete service: {}", e)))?;

        Ok(rows > 0)
    }

    pub fn list_all_names(&self) -> Result<Vec<String>> {
        let conn = self.db.connection()?;

        let mut stmt = conn
            .prepare("SELECT name FROM services ORDER BY name")
            .map_err(|e| KtmeError::Storage(format!("Failed to prepare query: {}", e)))?;

        let names: std::result::Result<Vec<String>, rusqlite::Error> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| KtmeError::Storage(format!("Failed to execute query: {}", e)))?
            .collect();

        names.map_err(|e| KtmeError::Storage(format!("Failed to collect results: {}", e)))
    }
}

// ============================================================================
// Document Mapping Repository
// ============================================================================

pub struct DocumentMappingRepository {
    db: Database,
}

impl DocumentMappingRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn add(
        &self,
        service_id: i64,
        provider: &str,
        location: &str,
        title: Option<&str>,
        section: Option<&str>,
        is_primary: bool,
    ) -> Result<DocumentMapping> {
        let conn = self.db.connection()?;

        conn.execute(
            "INSERT INTO document_mappings (service_id, provider, location, title, section, is_primary)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![service_id, provider, location, title, section, is_primary],
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to add mapping: {}", e)))?;

        let id = conn.last_insert_rowid();
        self.get_by_id(id)?
            .ok_or_else(|| KtmeError::Storage("Failed to retrieve created mapping".into()))
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<DocumentMapping>> {
        let conn = self.db.connection()?;

        let result = conn.query_row(
            "SELECT id, service_id, provider, location, title, section, is_primary, created_at, updated_at
             FROM document_mappings WHERE id = ?1",
            params![id],
            |row| {
                Ok(DocumentMapping {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    provider: row.get(2)?,
                    location: row.get(3)?,
                    title: row.get(4)?,
                    section: row.get(5)?,
                    is_primary: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        );

        match result {
            Ok(mapping) => Ok(Some(mapping)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(KtmeError::Storage(format!("Failed to get mapping: {}", e))),
        }
    }

    pub fn get_for_service(&self, service_id: i64) -> Result<Vec<DocumentMapping>> {
        let conn = self.db.connection()?;

        let mut stmt = conn
            .prepare(
                "SELECT id, service_id, provider, location, title, section, is_primary, created_at, updated_at
                 FROM document_mappings WHERE service_id = ?1 ORDER BY is_primary DESC, provider",
            )
            .map_err(|e| KtmeError::Storage(format!("Failed to prepare query: {}", e)))?;

        let mappings = stmt
            .query_map(params![service_id], |row| {
                Ok(DocumentMapping {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    provider: row.get(2)?,
                    location: row.get(3)?,
                    title: row.get(4)?,
                    section: row.get(5)?,
                    is_primary: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(|e| KtmeError::Storage(format!("Failed to query mappings: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| KtmeError::Storage(format!("Failed to collect mappings: {}", e)))?;

        Ok(mappings)
    }

    pub fn get_by_provider(&self, service_id: i64, provider: &str) -> Result<Option<DocumentMapping>> {
        let conn = self.db.connection()?;

        let result = conn.query_row(
            "SELECT id, service_id, provider, location, title, section, is_primary, created_at, updated_at
             FROM document_mappings WHERE service_id = ?1 AND provider = ?2",
            params![service_id, provider],
            |row| {
                Ok(DocumentMapping {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    provider: row.get(2)?,
                    location: row.get(3)?,
                    title: row.get(4)?,
                    section: row.get(5)?,
                    is_primary: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        );

        match result {
            Ok(mapping) => Ok(Some(mapping)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(KtmeError::Storage(format!("Failed to get mapping: {}", e))),
        }
    }

    pub fn get_primary(&self, service_id: i64) -> Result<Option<DocumentMapping>> {
        let conn = self.db.connection()?;

        let result = conn.query_row(
            "SELECT id, service_id, provider, location, title, section, is_primary, created_at, updated_at
             FROM document_mappings WHERE service_id = ?1 AND is_primary = TRUE LIMIT 1",
            params![service_id],
            |row| {
                Ok(DocumentMapping {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    provider: row.get(2)?,
                    location: row.get(3)?,
                    title: row.get(4)?,
                    section: row.get(5)?,
                    is_primary: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        );

        match result {
            Ok(mapping) => Ok(Some(mapping)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(KtmeError::Storage(format!("Failed to get mapping: {}", e))),
        }
    }

    pub fn delete(&self, id: i64) -> Result<bool> {
        let conn = self.db.connection()?;

        let rows = conn
            .execute("DELETE FROM document_mappings WHERE id = ?1", params![id])
            .map_err(|e| KtmeError::Storage(format!("Failed to delete mapping: {}", e)))?;

        Ok(rows > 0)
    }

    pub fn set_primary(&self, id: i64, service_id: i64) -> Result<()> {
        let conn = self.db.connection()?;

        // Clear existing primary
        conn.execute(
            "UPDATE document_mappings SET is_primary = FALSE WHERE service_id = ?1",
            params![service_id],
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to clear primary: {}", e)))?;

        // Set new primary
        conn.execute(
            "UPDATE document_mappings SET is_primary = TRUE, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![id],
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to set primary: {}", e)))?;

        Ok(())
    }
}

// ============================================================================
// Provider Config Repository
// ============================================================================

pub struct ProviderConfigRepository {
    db: Database,
}

impl ProviderConfigRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn save(&self, provider_type: &str, config: &serde_json::Value, is_default: bool) -> Result<()> {
        let conn = self.db.connection()?;

        conn.execute(
            "INSERT INTO provider_configs (provider_type, config_json, is_default)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(provider_type) DO UPDATE SET
                config_json = excluded.config_json,
                is_default = excluded.is_default,
                updated_at = CURRENT_TIMESTAMP",
            params![provider_type, config.to_string(), is_default],
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to save provider config: {}", e)))?;

        Ok(())
    }

    pub fn get(&self, provider_type: &str) -> Result<Option<ProviderConfig>> {
        let conn = self.db.connection()?;

        let result = conn.query_row(
            "SELECT id, provider_type, config_json, is_default, created_at, updated_at
             FROM provider_configs WHERE provider_type = ?1",
            params![provider_type],
            |row| {
                let config_str: String = row.get(2)?;
                Ok(ProviderConfig {
                    id: row.get(0)?,
                    provider_type: row.get(1)?,
                    config: serde_json::from_str(&config_str).unwrap_or(serde_json::Value::Null),
                    is_default: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        );

        match result {
            Ok(config) => Ok(Some(config)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(KtmeError::Storage(format!("Failed to get provider config: {}", e))),
        }
    }

    pub fn get_default(&self) -> Result<Option<ProviderConfig>> {
        let conn = self.db.connection()?;

        let result = conn.query_row(
            "SELECT id, provider_type, config_json, is_default, created_at, updated_at
             FROM provider_configs WHERE is_default = TRUE LIMIT 1",
            [],
            |row| {
                let config_str: String = row.get(2)?;
                Ok(ProviderConfig {
                    id: row.get(0)?,
                    provider_type: row.get(1)?,
                    config: serde_json::from_str(&config_str).unwrap_or(serde_json::Value::Null),
                    is_default: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        );

        match result {
            Ok(config) => Ok(Some(config)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(KtmeError::Storage(format!("Failed to get default provider: {}", e))),
        }
    }

    pub fn list(&self) -> Result<Vec<ProviderConfig>> {
        let conn = self.db.connection()?;

        let mut stmt = conn
            .prepare(
                "SELECT id, provider_type, config_json, is_default, created_at, updated_at
                 FROM provider_configs ORDER BY is_default DESC, provider_type",
            )
            .map_err(|e| KtmeError::Storage(format!("Failed to prepare query: {}", e)))?;

        let configs = stmt
            .query_map([], |row| {
                let config_str: String = row.get(2)?;
                Ok(ProviderConfig {
                    id: row.get(0)?,
                    provider_type: row.get(1)?,
                    config: serde_json::from_str(&config_str).unwrap_or(serde_json::Value::Null),
                    is_default: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| KtmeError::Storage(format!("Failed to query configs: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| KtmeError::Storage(format!("Failed to collect configs: {}", e)))?;

        Ok(configs)
    }

    pub fn set_default(&self, provider_type: &str) -> Result<()> {
        let conn = self.db.connection()?;

        // Clear existing default
        conn.execute(
            "UPDATE provider_configs SET is_default = FALSE",
            [],
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to clear default: {}", e)))?;

        // Set new default
        conn.execute(
            "UPDATE provider_configs SET is_default = TRUE, updated_at = CURRENT_TIMESTAMP
             WHERE provider_type = ?1",
            params![provider_type],
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to set default: {}", e)))?;

        Ok(())
    }

    pub fn delete(&self, provider_type: &str) -> Result<bool> {
        let conn = self.db.connection()?;

        let rows = conn
            .execute(
                "DELETE FROM provider_configs WHERE provider_type = ?1",
                params![provider_type],
            )
            .map_err(|e| KtmeError::Storage(format!("Failed to delete provider: {}", e)))?;

        Ok(rows > 0)
    }
}

// ============================================================================
// Generation History Repository
// ============================================================================

pub struct GenerationHistoryRepository {
    db: Database,
}

impl GenerationHistoryRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn record(
        &self,
        service_id: Option<i64>,
        provider: &str,
        document_id: Option<&str>,
        document_url: Option<&str>,
        action: &str,
        source_type: Option<&str>,
        source_identifier: Option<&str>,
        content_hash: Option<&str>,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<i64> {
        let conn = self.db.connection()?;

        conn.execute(
            "INSERT INTO generation_history
             (service_id, provider, document_id, document_url, action, source_type,
              source_identifier, content_hash, status, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                service_id,
                provider,
                document_id,
                document_url,
                action,
                source_type,
                source_identifier,
                content_hash,
                status,
                error_message
            ],
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to record history: {}", e)))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<GenerationRecord>> {
        let conn = self.db.connection()?;

        let mut stmt = conn
            .prepare(
                "SELECT id, service_id, provider, document_id, document_url, action,
                        source_type, source_identifier, content_hash, status, error_message, created_at
                 FROM generation_history
                 ORDER BY created_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| KtmeError::Storage(format!("Failed to prepare query: {}", e)))?;

        let records = stmt
            .query_map(params![limit as i64], |row| {
                Ok(GenerationRecord {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    provider: row.get(2)?,
                    document_id: row.get(3)?,
                    document_url: row.get(4)?,
                    action: row.get(5)?,
                    source_type: row.get(6)?,
                    source_identifier: row.get(7)?,
                    content_hash: row.get(8)?,
                    status: row.get(9)?,
                    error_message: row.get(10)?,
                    created_at: row.get(11)?,
                })
            })
            .map_err(|e| KtmeError::Storage(format!("Failed to query history: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| KtmeError::Storage(format!("Failed to collect history: {}", e)))?;

        Ok(records)
    }

    pub fn get_for_service(&self, service_id: i64, limit: usize) -> Result<Vec<GenerationRecord>> {
        let conn = self.db.connection()?;

        let mut stmt = conn
            .prepare(
                "SELECT id, service_id, provider, document_id, document_url, action,
                        source_type, source_identifier, content_hash, status, error_message, created_at
                 FROM generation_history
                 WHERE service_id = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| KtmeError::Storage(format!("Failed to prepare query: {}", e)))?;

        let records = stmt
            .query_map(params![service_id, limit as i64], |row| {
                Ok(GenerationRecord {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    provider: row.get(2)?,
                    document_id: row.get(3)?,
                    document_url: row.get(4)?,
                    action: row.get(5)?,
                    source_type: row.get(6)?,
                    source_identifier: row.get(7)?,
                    content_hash: row.get(8)?,
                    status: row.get(9)?,
                    error_message: row.get(10)?,
                    created_at: row.get(11)?,
                })
            })
            .map_err(|e| KtmeError::Storage(format!("Failed to query history: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| KtmeError::Storage(format!("Failed to collect history: {}", e)))?;

        Ok(records)
    }
}

// ============================================================================
// Diff Cache Repository
// ============================================================================

pub struct DiffCacheRepository {
    db: Database,
}

impl DiffCacheRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn set(
        &self,
        source_type: &str,
        source_identifier: &str,
        repository_path: Option<&str>,
        diff_json: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let conn = self.db.connection()?;

        conn.execute(
            "INSERT INTO diff_cache (source_type, source_identifier, repository_path, diff_json, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(source_type, source_identifier, repository_path) DO UPDATE SET
                diff_json = excluded.diff_json,
                expires_at = excluded.expires_at,
                created_at = CURRENT_TIMESTAMP",
            params![source_type, source_identifier, repository_path, diff_json, expires_at],
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to cache diff: {}", e)))?;

        Ok(())
    }

    pub fn get(
        &self,
        source_type: &str,
        source_identifier: &str,
        repository_path: Option<&str>,
    ) -> Result<Option<DiffCache>> {
        let conn = self.db.connection()?;

        let result = conn.query_row(
            "SELECT id, source_type, source_identifier, repository_path, diff_json, expires_at, created_at
             FROM diff_cache
             WHERE source_type = ?1 AND source_identifier = ?2
               AND (repository_path = ?3 OR (repository_path IS NULL AND ?3 IS NULL))
               AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)",
            params![source_type, source_identifier, repository_path],
            |row| {
                Ok(DiffCache {
                    id: row.get(0)?,
                    source_type: row.get(1)?,
                    source_identifier: row.get(2)?,
                    repository_path: row.get(3)?,
                    diff_json: row.get(4)?,
                    expires_at: row.get(5)?,
                    created_at: row.get(6)?,
                })
            },
        );

        match result {
            Ok(cache) => Ok(Some(cache)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(KtmeError::Storage(format!("Failed to get cache: {}", e))),
        }
    }

    pub fn clear_expired(&self) -> Result<u64> {
        let conn = self.db.connection()?;

        let rows = conn
            .execute(
                "DELETE FROM diff_cache WHERE expires_at IS NOT NULL AND expires_at <= CURRENT_TIMESTAMP",
                [],
            )
            .map_err(|e| KtmeError::Storage(format!("Failed to clear cache: {}", e)))?;

        Ok(rows as u64)
    }

    pub fn clear_all(&self) -> Result<u64> {
        let conn = self.db.connection()?;

        let rows = conn
            .execute("DELETE FROM diff_cache", [])
            .map_err(|e| KtmeError::Storage(format!("Failed to clear cache: {}", e)))?;

        Ok(rows as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Database {
        Database::in_memory().expect("Failed to create test database")
    }

    #[test]
    fn test_service_crud() {
        let db = setup_db();
        let repo = ServiceRepository::new(db);

        // Create
        let service = repo
            .create("test-service", Some("/path/to/service"), Some("Test description"))
            .expect("Failed to create service");
        assert_eq!(service.name, "test-service");

        // Get by name
        let found = repo
            .get_by_name("test-service")
            .expect("Failed to get service")
            .expect("Service not found");
        assert_eq!(found.id, service.id);

        // List
        let services = repo.list().expect("Failed to list services");
        assert_eq!(services.len(), 1);

        // Delete
        let deleted = repo.delete("test-service").expect("Failed to delete service");
        assert!(deleted);

        // Verify deleted
        let not_found = repo.get_by_name("test-service").expect("Query failed");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_document_mapping() {
        let db = setup_db();
        let service_repo = ServiceRepository::new(db.clone());
        let mapping_repo = DocumentMappingRepository::new(db);

        // Create service first
        let service = service_repo
            .create("test-service", None, None)
            .expect("Failed to create service");

        // Add mapping
        let mapping = mapping_repo
            .add(service.id, "confluence", "12345", Some("Test Doc"), None, true)
            .expect("Failed to add mapping");
        assert_eq!(mapping.provider, "confluence");
        assert!(mapping.is_primary);

        // Get mappings for service
        let mappings = mapping_repo
            .get_for_service(service.id)
            .expect("Failed to get mappings");
        assert_eq!(mappings.len(), 1);

        // Get primary
        let primary = mapping_repo
            .get_primary(service.id)
            .expect("Failed to get primary")
            .expect("Primary not found");
        assert_eq!(primary.id, mapping.id);
    }

    #[test]
    fn test_provider_config() {
        let db = setup_db();
        let repo = ProviderConfigRepository::new(db);

        let config = serde_json::json!({
            "base_url": "https://test.atlassian.net",
            "space_key": "TEST"
        });

        // Save config
        repo.save("confluence", &config, true)
            .expect("Failed to save config");

        // Get config
        let found = repo
            .get("confluence")
            .expect("Failed to get config")
            .expect("Config not found");
        assert!(found.is_default);

        // Get default
        let default = repo
            .get_default()
            .expect("Failed to get default")
            .expect("Default not found");
        assert_eq!(default.provider_type, "confluence");
    }
}
