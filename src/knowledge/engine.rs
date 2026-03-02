use crate::error::{KtmeError, Result};
use crate::storage::database::Database;
use crate::storage::models::{
    DocumentMapping, Feature, FeatureRelation, KnowledgeEdge, KnowledgeGraph, KnowledgeNode,
    KnowledgeNodeType, Service,
};
use crate::storage::repository::{
    DocumentMappingRepository, FeatureRelationRepository, FeatureRepository, ServiceRepository,
};
use serde::{Deserialize, Serialize};

/// All context pertaining to a single feature for AI agent consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureContext {
    pub feature: Feature,
    pub service: Service,
    pub documents: Vec<DocumentMapping>,
    /// Direct children with their relation metadata.
    pub children: Vec<(FeatureRelation, Feature)>,
    /// Direct parents with their relation metadata.
    pub parents: Vec<(FeatureRelation, Feature)>,
}

/// Engine for assembling and traversing the knowledge graph.
pub struct KnowledgeGraphEngine {
    service_repo: ServiceRepository,
    feature_repo: FeatureRepository,
    relation_repo: FeatureRelationRepository,
    mapping_repo: DocumentMappingRepository,
}

impl KnowledgeGraphEngine {
    pub fn new(db: Database) -> Self {
        Self {
            service_repo: ServiceRepository::new(db.clone()),
            feature_repo: FeatureRepository::new(db.clone()),
            relation_repo: FeatureRelationRepository::new(db.clone()),
            mapping_repo: DocumentMappingRepository::new(db),
        }
    }

    /// Build a knowledge graph optionally filtered to one service.
    ///
    /// `depth` controls how many levels of feature-relation edges are followed:
    ///   - 0: service nodes only (no features)
    ///   - 1: service + their direct features (no sub-feature edges)
    ///   - 2+: features + their children up to `depth` levels
    pub fn get_tree(&self, service_name: Option<&str>, depth: u32) -> Result<KnowledgeGraph> {
        let services = match service_name {
            Some(name) => {
                let s = self
                    .service_repo
                    .get_by_name(name)?
                    .ok_or_else(|| KtmeError::NotFound(format!("Service '{}' not found", name)))?;
                vec![s]
            }
            None => self.service_repo.list()?,
        };

        let mut nodes: Vec<KnowledgeNode> = Vec::new();
        let mut edges: Vec<KnowledgeEdge> = Vec::new();
        let mut root_node_ids: Vec<String> = Vec::new();

        for service in &services {
            let service_node_id = format!("service:{}", service.id);
            nodes.push(KnowledgeNode {
                id: service_node_id.clone(),
                node_type: KnowledgeNodeType::Service,
                name: service.name.clone(),
                description: service.description.clone(),
                metadata: serde_json::json!({ "path": service.path }),
                embedding: None,
                relevance_score: 1.0,
            });
            root_node_ids.push(service_node_id.clone());

            if depth == 0 {
                continue;
            }

            let features = self.feature_repo.list_by_service(service.id)?;
            for feature in &features {
                let feature_node_id = format!("feature:{}", feature.id);
                nodes.push(KnowledgeNode {
                    id: feature_node_id.clone(),
                    node_type: KnowledgeNodeType::Feature,
                    name: feature.name.clone(),
                    description: feature.description.clone(),
                    metadata: serde_json::json!({
                        "feature_type": feature.feature_type.to_string(),
                        "tags": feature.tags,
                    }),
                    embedding: None,
                    relevance_score: feature.relevance_score,
                });

                // Edge: service -> feature
                edges.push(KnowledgeEdge {
                    id: format!("edge:{}:{}", service_node_id, feature_node_id),
                    source_id: service_node_id.clone(),
                    target_id: feature_node_id.clone(),
                    edge_type: crate::storage::models::RelationType::Other,
                    strength: 1.0,
                    metadata: serde_json::json!({ "type": "contains" }),
                });

                if depth >= 2 {
                    self.add_relation_edges(
                        &feature.id,
                        &feature_node_id,
                        &mut nodes,
                        &mut edges,
                        depth - 1,
                        &mut std::collections::HashSet::new(),
                    )?;
                }
            }
        }

        Ok(KnowledgeGraph {
            nodes,
            edges,
            root_nodes: root_node_ids,
            metadata: serde_json::json!({
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "service_filter": service_name,
                "depth": depth,
            }),
        })
    }

    /// Recursively follow feature_relations up to `remaining_depth` levels,
    /// adding new nodes and edges. `visited` prevents infinite loops in cycles.
    fn add_relation_edges(
        &self,
        feature_id: &str,
        feature_node_id: &str,
        nodes: &mut Vec<KnowledgeNode>,
        edges: &mut Vec<KnowledgeEdge>,
        remaining_depth: u32,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        if remaining_depth == 0 || visited.contains(feature_id) {
            return Ok(());
        }
        visited.insert(feature_id.to_string());

        let relations = self.relation_repo.list_for_parent(feature_id)?;
        for rel in &relations {
            let child_node_id = format!("feature:{}", rel.child_feature_id);

            // Add child node if not already present
            if !nodes.iter().any(|n| n.id == child_node_id) {
                if let Some(child_feature) = self.feature_repo.get_by_id(&rel.child_feature_id)? {
                    nodes.push(KnowledgeNode {
                        id: child_node_id.clone(),
                        node_type: KnowledgeNodeType::Feature,
                        name: child_feature.name.clone(),
                        description: child_feature.description.clone(),
                        metadata: serde_json::json!({
                            "feature_type": child_feature.feature_type.to_string(),
                            "tags": child_feature.tags,
                        }),
                        embedding: None,
                        relevance_score: child_feature.relevance_score,
                    });
                }
            }

            edges.push(KnowledgeEdge {
                id: format!("edge:{}:{}", feature_node_id, child_node_id),
                source_id: feature_node_id.to_string(),
                target_id: child_node_id.clone(),
                edge_type: rel.relation_type,
                strength: rel.strength,
                metadata: rel.metadata.clone(),
            });

            self.add_relation_edges(
                &rel.child_feature_id,
                &child_node_id,
                nodes,
                edges,
                remaining_depth - 1,
                visited,
            )?;
        }

        Ok(())
    }

    /// Get all context for a single feature by ID.
    pub fn get_feature_context(&self, feature_id: &str) -> Result<FeatureContext> {
        let feature = self
            .feature_repo
            .get_by_id(feature_id)?
            .ok_or_else(|| KtmeError::NotFound(format!("Feature '{}' not found", feature_id)))?;

        let service = self
            .service_repo
            .get_by_id(feature.service_id)?
            .ok_or_else(|| {
                KtmeError::NotFound(format!("Service {} not found", feature.service_id))
            })?;

        let documents = self.mapping_repo.get_for_service(feature.service_id)?;

        let child_relations = self.relation_repo.list_for_parent(feature_id)?;
        let mut children: Vec<(FeatureRelation, Feature)> = Vec::new();
        for rel in child_relations {
            if let Some(child) = self.feature_repo.get_by_id(&rel.child_feature_id)? {
                children.push((rel, child));
            }
        }

        let parent_relations = self.relation_repo.list_for_child(feature_id)?;
        let mut parents: Vec<(FeatureRelation, Feature)> = Vec::new();
        for rel in parent_relations {
            if let Some(parent) = self.feature_repo.get_by_id(&rel.parent_feature_id)? {
                parents.push((rel, parent));
            }
        }

        Ok(FeatureContext {
            feature,
            service,
            documents,
            children,
            parents,
        })
    }

    /// Render a `KnowledgeGraph` as a Mermaid flowchart string.
    pub fn to_mermaid(&self, graph: &KnowledgeGraph) -> String {
        let mut out = String::from("graph TB\n");

        for node in &graph.nodes {
            let label = node.name.replace('"', "'");
            let shape = match node.node_type {
                KnowledgeNodeType::Service => format!("  {}[\"{}\"]\n", node.id, label),
                KnowledgeNodeType::Feature => format!("  {}(\"{}\")\n", node.id, label),
                _ => format!("  {}[\"{}\"]\n", node.id, label),
            };
            out.push_str(&shape);
        }

        for edge in &graph.edges {
            let relation_label = &edge.edge_type.to_string();
            out.push_str(&format!(
                "  {} -->|{}| {}\n",
                edge.source_id, relation_label, edge.target_id
            ));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;
    use crate::storage::models::FeatureType;
    use crate::storage::models::RelationType;
    use crate::storage::repository::{
        FeatureRelationRepository, FeatureRepository, ServiceRepository,
    };

    fn setup_engine() -> (KnowledgeGraphEngine, Database) {
        let db = Database::in_memory().expect("Failed to create test DB");
        let engine = KnowledgeGraphEngine::new(db.clone());
        (engine, db)
    }

    #[test]
    fn test_get_tree_empty_db() {
        let (engine, _db) = setup_engine();
        let graph = engine.get_tree(None, 2).expect("Failed to get tree");
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_get_tree_with_service() {
        let db = Database::in_memory().expect("Failed to create test DB");
        let service_repo = ServiceRepository::new(db.clone());
        let feature_repo = FeatureRepository::new(db.clone());
        let engine = KnowledgeGraphEngine::new(db.clone());

        let service = service_repo
            .create("auth-service", Some("/src/auth"), Some("Auth"))
            .expect("Failed to create service");
        feature_repo
            .create(
                "feat-jwt",
                service.id,
                "JWT Token Management",
                Some("Manages JWT tokens"),
                FeatureType::Security,
                vec!["jwt".to_string()],
                serde_json::json!({}),
            )
            .expect("Failed to create feature");

        let graph = engine
            .get_tree(Some("auth-service"), 2)
            .expect("Failed to get tree");

        // Should have at least service node + feature node
        assert!(graph.nodes.len() >= 2);
        assert!(!graph.edges.is_empty());
        assert!(graph.nodes.iter().any(|n| n.name == "auth-service"));
        assert!(graph.nodes.iter().any(|n| n.name == "JWT Token Management"));
    }

    #[test]
    fn test_get_feature_context() {
        let db = Database::in_memory().expect("Failed to create test DB");
        let service_repo = ServiceRepository::new(db.clone());
        let feature_repo = FeatureRepository::new(db.clone());
        let relation_repo = FeatureRelationRepository::new(db.clone());
        let engine = KnowledgeGraphEngine::new(db.clone());

        let service = service_repo
            .create("test-svc", None, None)
            .expect("Failed to create service");
        feature_repo
            .create(
                "parent-feat",
                service.id,
                "Parent Feature",
                None,
                FeatureType::BusinessLogic,
                vec![],
                serde_json::json!({}),
            )
            .expect("Failed to create parent feature");
        feature_repo
            .create(
                "child-feat",
                service.id,
                "Child Feature",
                None,
                FeatureType::Api,
                vec![],
                serde_json::json!({}),
            )
            .expect("Failed to create child feature");
        relation_repo
            .create(
                "rel-001",
                "parent-feat",
                "child-feat",
                RelationType::DependsOn,
                0.9,
                serde_json::json!({}),
            )
            .expect("Failed to create relation");

        let ctx = engine
            .get_feature_context("parent-feat")
            .expect("Failed to get feature context");

        assert_eq!(ctx.feature.name, "Parent Feature");
        assert_eq!(ctx.service.name, "test-svc");
        assert_eq!(ctx.children.len(), 1);
        assert_eq!(ctx.children[0].1.name, "Child Feature");
        assert!(ctx.parents.is_empty());
    }

    #[test]
    fn test_to_mermaid_output() {
        let db = Database::in_memory().expect("Failed to create test DB");
        let service_repo = ServiceRepository::new(db.clone());
        let feature_repo = FeatureRepository::new(db.clone());
        let engine = KnowledgeGraphEngine::new(db.clone());

        let service = service_repo
            .create("svc", None, None)
            .expect("Failed to create service");
        feature_repo
            .create(
                "f1",
                service.id,
                "Feature One",
                None,
                FeatureType::Api,
                vec![],
                serde_json::json!({}),
            )
            .expect("Failed to create feature");

        let graph = engine.get_tree(None, 1).expect("Failed to get tree");
        let mermaid = engine.to_mermaid(&graph);

        assert!(mermaid.contains("graph TB"));
        assert!(mermaid.contains("svc"));
        assert!(mermaid.contains("Feature One"));
    }
}
