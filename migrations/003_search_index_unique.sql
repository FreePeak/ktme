-- KTME Search Index Unique Constraint
-- Version: 003
-- Description: Add unique constraint on (feature_id, content_type) in search_index
--              to enable idempotent upserts via ON CONFLICT clause.

-- First, drop the dependent view so we can safely modify the table
DROP VIEW IF EXISTS search_results_view;

-- This migration handles both fresh installs (where search_index doesn't exist yet)
-- and upgrades (where search_index exists from migration 2).

-- Create the new table with unique constraint
CREATE TABLE IF NOT EXISTS search_index_new (
    id TEXT PRIMARY KEY,
    feature_id TEXT NOT NULL,
    content_type TEXT NOT NULL,
    content TEXT NOT NULL,
    embedding BLOB,
    indexed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE,
    UNIQUE(feature_id, content_type)
);

-- For upgrades: copy data from old table if it exists
-- The subquery checks if the old table exists, making this safe for fresh installs
INSERT OR IGNORE INTO search_index_new (id, feature_id, content_type, content, embedding, indexed_at)
SELECT id, feature_id, content_type, content, embedding, indexed_at 
FROM search_index 
WHERE (SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='search_index') > 0;

-- Drop old table if it exists (from migration 2)
DROP TABLE IF EXISTS search_index;

-- Rename new table to original name
ALTER TABLE search_index_new RENAME TO search_index;

-- Restore indexes
CREATE INDEX IF NOT EXISTS idx_search_index_feature ON search_index(feature_id);
CREATE INDEX IF NOT EXISTS idx_search_index_type ON search_index(content_type);
CREATE INDEX IF NOT EXISTS idx_search_index_content ON search_index(content);

-- Recreate the view (dropped above)
CREATE VIEW IF NOT EXISTS search_results_view AS
SELECT
    si.id,
    si.feature_id,
    f.service_id,
    s.name as service_name,
    f.name as feature_name,
    f.feature_type,
    si.content_type,
    si.content,
    f.relevance_score,
    si.indexed_at
FROM search_index si
JOIN features f ON si.feature_id = f.id
JOIN services s ON f.service_id = s.id;

INSERT OR IGNORE INTO schema_versions (version) VALUES (3);
