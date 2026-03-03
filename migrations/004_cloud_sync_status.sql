-- KTME Cloud Sync Status
-- Version: 004
-- Description: Add cloud sync status tracking for Notion and Confluence

-- Cloud sync status table - tracks sync state for each document mapping
CREATE TABLE IF NOT EXISTS cloud_sync_status (
    id TEXT PRIMARY KEY,  -- UUID
    mapping_id INTEGER NOT NULL,
    provider TEXT NOT NULL,  -- 'notion', 'confluence'
    remote_id TEXT NOT NULL,  -- ID from the cloud provider
    content_hash_local TEXT NOT NULL,  -- SHA-256 hash of local content
    content_hash_remote TEXT,  -- SHA-256 hash of remote content (null if never fetched)
    sync_state TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'synced', 'conflict', 'error'
    conflict_data TEXT,  -- JSON with conflict details if applicable
    last_synced DATETIME,
    error_message TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mapping_id) REFERENCES document_mappings(id) ON DELETE CASCADE,
    UNIQUE(mapping_id, provider)
);

-- Notion-specific configuration storage
CREATE TABLE IF NOT EXISTS notion_configs (
    id TEXT PRIMARY KEY,  -- UUID
    workspace_id TEXT NOT NULL,
    workspace_name TEXT,
    api_key TEXT NOT NULL,  -- Should be encrypted in production
    parent_page_id TEXT,  -- Root page for syncing
    sync_enabled INTEGER DEFAULT 1,
    last_full_sync DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(workspace_id)
);

-- Sync history for audit trail
CREATE TABLE IF NOT EXISTS sync_history (
    id TEXT PRIMARY KEY,  -- UUID
    provider TEXT NOT NULL,
    direction TEXT NOT NULL,  -- 'fetch' (cloud to local), 'push' (local to cloud)
    mapping_id INTEGER,
    remote_id TEXT,
    status TEXT NOT NULL,  -- 'success', 'failed', 'conflict', 'skipped'
    changes_detected INTEGER DEFAULT 0,  -- Boolean: was there actually a change
    local_hash_before TEXT,
    local_hash_after TEXT,
    remote_hash_before TEXT,
    remote_hash_after TEXT,
    error_message TEXT,
    synced_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mapping_id) REFERENCES document_mappings(id) ON DELETE SET NULL
);

-- Insert schema version
INSERT OR IGNORE INTO schema_versions (version) VALUES (4);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_cloud_sync_mapping ON cloud_sync_status(mapping_id);
CREATE INDEX IF NOT EXISTS idx_cloud_sync_provider ON cloud_sync_status(provider);
CREATE INDEX IF NOT EXISTS idx_cloud_sync_state ON cloud_sync_status(sync_state);
CREATE INDEX IF NOT EXISTS idx_cloud_sync_remote ON cloud_sync_status(remote_id);

CREATE INDEX IF NOT EXISTS idx_notion_workspace ON notion_configs(workspace_id);
CREATE INDEX IF NOT EXISTS idx_notion_parent ON notion_configs(parent_page_id);

CREATE INDEX IF NOT EXISTS idx_sync_history_provider ON sync_history(provider);
CREATE INDEX IF NOT EXISTS idx_sync_history_direction ON sync_history(direction);
CREATE INDEX IF NOT EXISTS idx_sync_history_status ON sync_history(status);
CREATE INDEX IF NOT EXISTS idx_sync_history_synced ON sync_history(synced_at DESC);

-- Triggers
CREATE TRIGGER IF NOT EXISTS update_cloud_sync_timestamp
    AFTER UPDATE ON cloud_sync_status
    FOR EACH ROW
    BEGIN
        UPDATE cloud_sync_status SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_notion_config_timestamp
    AFTER UPDATE ON notion_configs
    FOR EACH ROW
    BEGIN
        UPDATE notion_configs SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

-- Views
CREATE VIEW IF NOT EXISTS sync_status_summary AS
SELECT
    css.provider,
    css.sync_state,
    COUNT(*) as count,
    MIN(css.last_synced) as earliest_sync,
    MAX(css.last_synced) as latest_sync
FROM cloud_sync_status css
GROUP BY css.provider, css.sync_state;

CREATE VIEW IF NOT EXISTS recent_sync_activity AS
SELECT
    sh.provider,
    sh.direction,
    sh.status,
    sh.changes_detected,
    sh.synced_at,
    dm.provider as local_provider,
    dm.location
FROM sync_history sh
LEFT JOIN document_mappings dm ON sh.mapping_id = dm.id
ORDER BY sh.synced_at DESC
LIMIT 100;
