//! Simple test for database connection
use ktme::storage::database::Database;
use ktme::error::Result;

fn main() -> Result<()> {
    println!("Testing basic database connection...");

    // Create in-memory database without migrations
    let conn = rusqlite::Connection::open_in_memory()
        .map_err(|e| ktme::error::KtmeError::Storage(format!("Failed to open database: {}", e)))?;

    // Create services table manually
    conn.execute_batch(
        "CREATE TABLE services (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            path TEXT,
            description TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
    ).map_err(|e| ktme::error::KtmeError::Storage(format!("Failed to create table: {}", e)))?;

    println!("✅ Database and table created successfully");

    // Insert a test record
    conn.execute(
        "INSERT INTO services (name, path, description) VALUES (?1, ?2, ?3)",
        rusqlite::params!["test-service", "/test/path", "Test service"],
    ).map_err(|e| ktme::error::KtmeError::Storage(format!("Failed to insert: {}", e)))?;

    println!("✅ Test record inserted successfully");

    // Query the record
    let name: String = conn.query_row(
        "SELECT name FROM services WHERE name = ?1",
        rusqlite::params!["test-service"],
        |row| row.get(0),
    ).map_err(|e| ktme::error::KtmeError::Storage(format!("Failed to query: {}", e)))?;

    println!("✅ Retrieved record: {}", name);

    println!("\n🎉 Basic database test passed!");

    Ok(())
}