use crate::error::{KtmeError, Result};
use rusqlite::{Connection, OpenFlags};
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
            let config_dir = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("ktme");
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
        let conn = Connection::open_in_memory()
            .map_err(|e| KtmeError::Storage(format!("Failed to open in-memory database: {}", e)))?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| KtmeError::Storage(format!("Failed to set pragmas: {}", e)))?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            path: PathBuf::from(":memory:"),
        };

        db.migrate()?;
        Ok(db)
    }

    /// Run database migrations
    pub fn migrate(&self) -> Result<()> {
        let conn = self.connection()?;

        // Run initial migration
        conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
            .map_err(|e| KtmeError::Storage(format!("Migration failed: {}", e)))?;

        tracing::debug!("Database migrations completed");
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

        Ok(DatabaseStats {
            path: self.path.clone(),
            service_count: service_count as u64,
            mapping_count: mapping_count as u64,
            history_count: history_count as u64,
            provider_count: provider_count as u64,
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
    }
}
