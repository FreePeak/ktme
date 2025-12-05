-- KTME Initial Schema
-- Version: 001
-- Description: Initial database schema for ktme

-- Service registry
CREATE TABLE IF NOT EXISTS services (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    path TEXT,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Document locations for each service (supports multiple providers per service)
CREATE TABLE IF NOT EXISTS document_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER NOT NULL,
    provider TEXT NOT NULL,
    location TEXT NOT NULL,
    title TEXT,
    section TEXT,
    is_primary BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE CASCADE,
    UNIQUE(service_id, provider, location)
);

-- Provider configurations (Confluence, Google Docs, etc.)
CREATE TABLE IF NOT EXISTS provider_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_type TEXT NOT NULL UNIQUE,
    config_json TEXT NOT NULL,
    is_default BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Prompt templates for AI generation
CREATE TABLE IF NOT EXISTS prompt_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    template TEXT NOT NULL,
    variables_json TEXT,
    output_format TEXT DEFAULT 'markdown',
    is_builtin BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Document templates (api-doc, changelog, etc.)
CREATE TABLE IF NOT EXISTS document_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    content TEXT NOT NULL,
    template_type TEXT,
    is_builtin BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Generation history (audit log)
CREATE TABLE IF NOT EXISTS generation_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER,
    provider TEXT NOT NULL,
    document_id TEXT,
    document_url TEXT,
    action TEXT NOT NULL,
    source_type TEXT,
    source_identifier TEXT,
    content_hash TEXT,
    status TEXT NOT NULL,
    error_message TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE SET NULL
);

-- Diff cache (for performance)
CREATE TABLE IF NOT EXISTS diff_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_type TEXT NOT NULL,
    source_identifier TEXT NOT NULL,
    repository_path TEXT,
    diff_json TEXT NOT NULL,
    expires_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source_type, source_identifier, repository_path)
);

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_versions (
    version INTEGER PRIMARY KEY,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Insert schema version
INSERT OR IGNORE INTO schema_versions (version) VALUES (1);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_services_name ON services(name);
CREATE INDEX IF NOT EXISTS idx_document_mappings_service ON document_mappings(service_id);
CREATE INDEX IF NOT EXISTS idx_document_mappings_provider ON document_mappings(provider);
CREATE INDEX IF NOT EXISTS idx_generation_history_service ON generation_history(service_id);
CREATE INDEX IF NOT EXISTS idx_generation_history_created ON generation_history(created_at);
CREATE INDEX IF NOT EXISTS idx_diff_cache_lookup ON diff_cache(source_type, source_identifier);
CREATE INDEX IF NOT EXISTS idx_diff_cache_expires ON diff_cache(expires_at);
