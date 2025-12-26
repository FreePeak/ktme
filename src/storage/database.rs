use crate::error::{KtmeError, Result};
use rusqlite::{Connection, OpenFlags, params};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Database wrapper for SQLite connection management
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl Database {
    /// Create a new database connection
    ///
    /// If path is None, uses default location: ~/.config/ktme/ktme.db
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let db_path = path.unwrap_or_else(|| {
            // Use ~/.config/ktme/ktme.db explicitly
            let home_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."));
            let config_dir = home_dir.join(".config").join("ktme");
            if let Err(e) = std::fs::create_dir_all(&config_dir) {
                tracing::warn!("Failed to create config directory: {}", e);
            }
            config_dir.join("ktme.db")
        });

        tracing::info!("Opening database at: {}", db_path.display());

        let conn = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to open database: {}", e)))?;

        // Enable foreign keys and WAL mode for better concurrency
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )
        .map_err(|e| KtmeError::Storage(format!("Failed to set pragmas: {}", e)))?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            path: db_path,
        };

        db.migrate()?;
        Ok(db)
    }

    /// Create an in-memory database (for testing)
    pub fn in_memory() -> Result<Self> {
        // First create the connection and run migrations
        let conn = Connection::open_in_memory()
            .map_err(|e| KtmeError::Storage(format!("Failed to open in-memory database: {}", e)))?;

        // Only set foreign keys for in-memory (WAL mode not supported)
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| KtmeError::Storage(format!("Failed to set pragmas: {}", e)))?;

        // Create schema_versions table if it doesn't exist
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_versions (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );"
        ).map_err(|e| KtmeError::Storage(format!("Failed to create schema_versions table: {}", e)))?;

        // Get current version (should be 0 for new in-memory DB)
        let current_version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| row.get(0))
            .unwrap_or(0);

        // Run migrations in order directly (no mutex needed)
        let migrations = vec![
            (1, include_str!("../../migrations/001_initial.sql")),
            (2, include_str!("../../migrations/002_features_and_search.sql")),
        ];

        for (version, sql) in &migrations {
            if *version > current_version {
                conn.execute_batch(sql)
                    .map_err(|e| KtmeError::Storage(format!("Migration {} failed: {}", version, e)))?;

                // Record the migration
                conn.execute(
                    "INSERT OR IGNORE INTO schema_versions (version) VALUES (?1)",
                    rusqlite::params![version],
                ).map_err(|e| KtmeError::Storage(format!("Failed to record migration {}: {}", version, e)))?;
            }
        }

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            path: PathBuf::from(":memory:"),
        };

        Ok(db)
    }

    /// Run database migrations
    pub fn migrate(&self) -> Result<()> {
        let conn = self.connection()?;

        // Create schema_versions table if it doesn't exist
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_versions (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );"
        ).map_err(|e| KtmeError::Storage(format!("Failed to create schema_versions table: {}", e)))?;

        // Get current version
        let current_version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_versions", [], |row| row.get(0))
            .unwrap_or(0);

        tracing::info!("Current database schema version: {}", current_version);

        // Run migrations in order
        let migrations = vec![
            (1, include_str!("../../migrations/001_initial.sql")),
            (2, include_str!("../../migrations/002_features_and_search.sql")),
        ];

        let latest_version = migrations.last().map(|(v, _)| *v).unwrap_or(0);

        for (version, sql) in &migrations {
            if *version > current_version {
                tracing::info!("Running migration version: {}", version);
                conn.execute_batch(sql)
                    .map_err(|e| KtmeError::Storage(format!("Migration {} failed: {}", version, e)))?;

                // Record the migration
                conn.execute(
                    "INSERT OR IGNORE INTO schema_versions (version) VALUES (?1)",
                    params![version],
                ).map_err(|e| KtmeError::Storage(format!("Failed to record migration {}: {}", version, e)))?;

                tracing::debug!("Migration {} completed successfully", version);
            }
        }

        tracing::info!("Database migrations completed. Latest version: {}", latest_version);
        Ok(())
    }

    /// Get a connection guard for executing queries
    pub fn connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| KtmeError::Storage(format!("Failed to acquire database lock: {}", e)))
    }

    /// Get the database file path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Check if the database is healthy
    pub fn health_check(&self) -> Result<bool> {
        let conn = self.connection()?;
        conn.execute_batch("SELECT 1")
            .map_err(|e| KtmeError::Storage(format!("Health check failed: {}", e)))?;
        Ok(true)
    }

    /// Get database statistics
    pub fn stats(&self) -> Result<DatabaseStats> {
        let conn = self.connection()?;

        let service_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM services", [], |row| row.get(0))
            .unwrap_or(0);

        let mapping_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM document_mappings", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        let history_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM generation_history", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        let provider_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM provider_configs", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        let feature_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM features", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        Ok(DatabaseStats {
            path: self.path.clone(),
            service_count: service_count as u64,
            mapping_count: mapping_count as u64,
            history_count: history_count as u64,
            provider_count: provider_count as u64,
            feature_count: feature_count as u64,
        })
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            path: self.path.clone(),
        }
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub path: PathBuf,
    pub service_count: u64,
    pub mapping_count: u64,
    pub history_count: u64,
    pub provider_count: u64,
    pub feature_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_database() {
        let db = Database::in_memory().expect("Failed to create in-memory database");
        assert!(db.health_check().unwrap());
    }

    #[test]
    fn test_database_stats() {
        let db = Database::in_memory().expect("Failed to create in-memory database");
        let stats = db.stats().expect("Failed to get stats");
        assert_eq!(stats.service_count, 0);
        assert_eq!(stats.mapping_count, 0);
        assert_eq!(stats.feature_count, 0);
    }

    #[test]
    fn test_migration_system() {
        let db = Database::in_memory().expect("Failed to create test database");

        // Check that migrations were applied
        let conn = db.connection().expect("Failed to get connection");

        // Check if schema versions table exists
        let mut stmt = conn
            .prepare("SELECT MAX(version) FROM schema_versions")
            .expect("Failed to prepare version query");

        let version: i64 = stmt
            .query_row([], |row| row.get(0))
            .expect("Failed to get version");

        assert!(version >= 2, "Schema version should be at least 2");
    }

    #[test]
    fn test_new_tables_exist() {
        let db = Database::in_memory().expect("Failed to create test database");

        // Check that new tables exist
        let conn = db.connection().expect("Failed to get connection");

        let tables = ["features", "feature_relations", "search_index"];

        for table in &tables {
            let mut stmt = conn
                .prepare(&format!("SELECT COUNT(*) FROM {}", table))
                .expect(&format!("Failed to prepare query for {}", table));

            let count: i64 = stmt
                .query_row([], |row| row.get(0))
                .expect(&format!("Failed to query {}", table));

            // Should not fail even if table is empty
            assert!(count >= 0, "Table {} should exist", table);
        }
    }

    #[test]
    fn test_feature_stats_field() {
        let db = Database::in_memory().expect("Failed to create test database");
        let stats = db.stats().expect("Failed to get stats");

        // The feature_count field should be available
        assert!(stats.feature_count >= 0, "Feature count should be available in stats");
    }
}
