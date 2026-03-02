-- KTME Search Index Unique Constraint
-- Version: 003
-- Description: Add unique constraint on (feature_id, content_type) in search_index
--              to enable idempotent upserts via ON CONFLICT clause.

-- SQLite does not support ADD CONSTRAINT after table creation, so we recreate
-- the table with the new constraint then repopulate from the old data.

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

INSERT OR IGNORE INTO search_index_new (id, feature_id, content_type, content, embedding, indexed_at)
SELECT id, feature_id, content_type, content, embedding, indexed_at FROM search_index;

DROP TABLE search_index;
ALTER TABLE search_index_new RENAME TO search_index;

-- Restore indexes
CREATE INDEX IF NOT EXISTS idx_search_index_feature ON search_index(feature_id);
CREATE INDEX IF NOT EXISTS idx_search_index_type ON search_index(content_type);
CREATE INDEX IF NOT EXISTS idx_search_index_content ON search_index(content);

INSERT OR IGNORE INTO schema_versions (version) VALUES (3);
