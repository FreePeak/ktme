-- KTME Features and Search Enhancement
-- Version: 002
-- Description: Add feature management, relationships, and search capabilities

-- Features table - represents software features within services
CREATE TABLE IF NOT EXISTS features (
    id TEXT PRIMARY KEY,  -- UUID for distributed compatibility
    service_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    feature_type TEXT NOT NULL,  -- 'api', 'ui', 'business_logic', etc.
    tags TEXT,  -- JSON array of tags
    metadata TEXT,  -- JSON object for additional metadata
    relevance_score REAL DEFAULT 0.0,
    embedding BLOB,  -- Vector embedding for semantic search
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE CASCADE,
    UNIQUE(service_id, name)
);

-- Feature relationships table
CREATE TABLE IF NOT EXISTS feature_relations (
    id TEXT PRIMARY KEY,  -- UUID
    parent_feature_id TEXT NOT NULL,
    child_feature_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,  -- 'depends_on', 'implements', 'extends', etc.
    strength REAL DEFAULT 1.0,  -- Relationship strength (0.0-1.0)
    metadata TEXT,  -- JSON object for additional metadata
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_feature_id) REFERENCES features(id) ON DELETE CASCADE,
    FOREIGN KEY (child_feature_id) REFERENCES features(id) ON DELETE CASCADE,
    CHECK (parent_feature_id != child_feature_id)
);

-- Search index for semantic and keyword search
CREATE TABLE IF NOT EXISTS search_index (
    id TEXT PRIMARY KEY,  -- UUID
    feature_id TEXT NOT NULL,
    content_type TEXT NOT NULL,  -- 'feature_name', 'documentation', 'code_example', etc.
    content TEXT NOT NULL,
    embedding BLOB,  -- Vector embedding for semantic search
    indexed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
);

-- Document mappings enhancement - add feature relationship
ALTER TABLE document_mappings ADD COLUMN feature_id TEXT;

-- Search results cache for performance
CREATE TABLE IF NOT EXISTS search_cache (
    id TEXT PRIMARY KEY,
    query_hash TEXT NOT NULL,
    query_params TEXT,  -- JSON of search parameters
    results TEXT NOT NULL,  -- JSON of search results
    expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Knowledge graph nodes cache
CREATE TABLE IF NOT EXISTS knowledge_graph_cache (
    id TEXT PRIMARY KEY,
    service_filter TEXT,  -- JSON array of service IDs
    graph_data TEXT NOT NULL,  -- JSON of knowledge graph
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL
);

-- Insert schema version
INSERT OR IGNORE INTO schema_versions (version) VALUES (2);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_features_service ON features(service_id);
CREATE INDEX IF NOT EXISTS idx_features_type ON features(feature_type);
CREATE INDEX IF NOT EXISTS idx_features_name ON features(name);
CREATE INDEX IF NOT EXISTS idx_features_relevance ON features(relevance_score DESC);
-- Note: idx_features_tags skipped for SQLite compatibility

CREATE INDEX IF NOT EXISTS idx_feature_relations_parent ON feature_relations(parent_feature_id);
CREATE INDEX IF NOT EXISTS idx_feature_relations_child ON feature_relations(child_feature_id);
CREATE INDEX IF NOT EXISTS idx_feature_relations_type ON feature_relations(relation_type);
CREATE INDEX IF NOT EXISTS idx_feature_relations_strength ON feature_relations(strength DESC);

CREATE INDEX IF NOT EXISTS idx_search_index_feature ON search_index(feature_id);
CREATE INDEX IF NOT EXISTS idx_search_index_type ON search_index(content_type);
CREATE INDEX IF NOT EXISTS idx_search_index_content ON search_index(content);

CREATE INDEX IF NOT EXISTS idx_search_cache_query ON search_cache(query_hash);
CREATE INDEX IF NOT EXISTS idx_search_cache_expires ON search_cache(expires_at);

CREATE INDEX IF NOT EXISTS idx_knowledge_graph_service ON knowledge_graph_cache(service_filter);
CREATE INDEX IF NOT EXISTS idx_knowledge_graph_expires ON knowledge_graph_cache(expires_at);

-- Triggers to update timestamps
CREATE TRIGGER IF NOT EXISTS update_features_timestamp
    AFTER UPDATE ON features
    FOR EACH ROW
    BEGIN
        UPDATE features SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

-- Helper views for common queries
CREATE VIEW IF NOT EXISTS feature_details AS
SELECT
    f.id,
    f.service_id,
    s.name as service_name,
    f.name,
    f.description,
    f.feature_type,
    f.tags,
    f.metadata,
    f.relevance_score,
    f.created_at,
    f.updated_at,
    COUNT(DISTINCT fr_child.id) as child_count,
    COUNT(DISTINCT fr_parent.id) as parent_count
FROM features f
LEFT JOIN services s ON f.service_id = s.id
LEFT JOIN feature_relations fr_child ON f.id = fr_child.parent_feature_id
LEFT JOIN feature_relations fr_parent ON f.id = fr_parent.child_feature_id
GROUP BY f.id;

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