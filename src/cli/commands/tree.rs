use crate::error::Result;
use crate::knowledge::engine::KnowledgeGraphEngine;
use crate::storage::database::Database;

/// Pretty-print the knowledge tree to stdout.
///
/// - `service`: optional service name filter
/// - `depth`: traversal depth (default 2)
/// - `mermaid`: also print a Mermaid flowchart block
pub async fn execute(
    service: Option<String>,
    depth: Option<u32>,
    mermaid: bool,
) -> Result<()> {
    let db = Database::new(None)?;
    let engine = KnowledgeGraphEngine::new(db);
    let effective_depth = depth.unwrap_or(2);
    let graph = engine.get_tree(service.as_deref(), effective_depth)?;

    if graph.nodes.is_empty() {
        println!("Knowledge graph is empty. Run `ktme init` to populate it.");
        return Ok(());
    }

    println!("Knowledge Tree (depth={})", effective_depth);
    println!("{}", "=".repeat(60));

    for root_id in &graph.root_nodes {
        let root = graph.nodes.iter().find(|n| &n.id == root_id);
        if let Some(node) = root {
            println!("\n[{}] {}", format_type(&node.node_type), node.name);
            if let Some(ref desc) = node.description {
                println!("    {}", desc);
            }

            // Print direct children
            let child_edges: Vec<_> = graph
                .edges
                .iter()
                .filter(|e| &e.source_id == root_id)
                .collect();
            for edge in &child_edges {
                let child = graph.nodes.iter().find(|n| n.id == edge.target_id);
                if let Some(child_node) = child {
                    println!(
                        "  |-- [{}] {} ({})",
                        format_type(&child_node.node_type),
                        child_node.name,
                        edge.edge_type
                    );
                    if let Some(ref desc) = child_node.description {
                        println!("  |       {}", desc);
                    }

                    // Print grandchildren
                    let grandchild_edges: Vec<_> = graph
                        .edges
                        .iter()
                        .filter(|e| &e.source_id == &edge.target_id)
                        .collect();
                    for gc_edge in &grandchild_edges {
                        let gc = graph.nodes.iter().find(|n| n.id == gc_edge.target_id);
                        if let Some(gc_node) = gc {
                            println!(
                                "  |     |-- [{}] {} ({})",
                                format_type(&gc_node.node_type),
                                gc_node.name,
                                gc_edge.edge_type
                            );
                        }
                    }
                }
            }
        }
    }

    println!("\n{}", "-".repeat(60));
    println!(
        "Totals: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    if mermaid {
        println!("\n```mermaid");
        println!("{}", engine.to_mermaid(&graph));
        println!("```");
    }

    Ok(())
}

fn format_type(node_type: &crate::storage::models::KnowledgeNodeType) -> &'static str {
    use crate::storage::models::KnowledgeNodeType;
    match node_type {
        KnowledgeNodeType::Service => "SVC",
        KnowledgeNodeType::Feature => "FEAT",
        KnowledgeNodeType::Document => "DOC",
        KnowledgeNodeType::Api => "API",
        KnowledgeNodeType::Example => "EX",
        KnowledgeNodeType::Concept => "CONCEPT",
    }
}
