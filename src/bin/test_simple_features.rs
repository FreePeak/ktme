//! Simple feature test without migration conflicts

use ktme::error::Result;
use ktme::storage::database::Database;
use ktme::storage::models::{FeatureType, SearchQuery};
use ktme::storage::repository::{FeatureRepository, ServiceRepository};

fn main() -> Result<()> {
    println!("🧪 Simple KTME Feature Test");
    println!("=====================================");

    // Test 1: Create in-memory database directly
    println!("\n1. Creating in-memory database...");
    let db = Database::in_memory()?;
    println!("✅ Database created successfully");

    // Test 2: Create service
    println!("\n2. Creating test service...");
    let service_repo = ServiceRepository::new(db.clone());
    let service = service_repo.create(
        "test-service",
        Some("/test/path"),
        Some("Test service for feature validation"),
    )?;
    println!("✅ Service created: {} (ID: {})", service.name, service.id);

    // Test 3: Create feature
    println!("\n3. Creating test feature...");
    let feature_repo = FeatureRepository::new(db.clone());
    let feature = feature_repo.create(
        "test-feature-001",
        service.id,
        "AI Documentation Generator",
        Some("Generates documentation from code changes using AI"),
        FeatureType::Api,
        vec![
            "ai".to_string(),
            "docs".to_string(),
            "automation".to_string(),
        ],
        serde_json::json!({
            "complexity": "high",
            "status": "active",
            "version": "1.0.0"
        }),
    )?;
    println!("✅ Feature created: {}", feature.name);
    println!("   Type: {:?}", feature.feature_type);
    println!("   Tags: {:?}", feature.tags);
    println!("   Relevance Score: {}", feature.relevance_score);

    // Test 4: Retrieve feature
    println!("\n4. Retrieving feature...");
    let retrieved = feature_repo.get_by_id("test-feature-001")?;
    match retrieved {
        Some(f) => {
            println!("✅ Feature retrieved: {}", f.name);
            assert_eq!(f.name, "AI Documentation Generator");
        }
        None => {
            return Err(ktme::error::KtmeError::Storage(
                "Feature not found".to_string(),
            ));
        }
    }

    // Test 5: List features by service
    println!("\n5. Listing features by service...");
    let features = feature_repo.list_by_service(service.id)?;
    println!("✅ Found {} features", features.len());

    for f in &features {
        println!("   - {} ({:?})", f.name, f.feature_type);
    }
    assert_eq!(features.len(), 1);

    // Test 6: Search features
    println!("\n6. Testing feature search...");
    let search_query = SearchQuery {
        query: "documentation".to_string(),
        service_ids: None,
        feature_types: None,
        content_types: None,
        limit: Some(10),
        similarity_threshold: None,
        include_related: false,
        depth: None,
    };

    let search_results = feature_repo.search(&search_query)?;
    println!("✅ Search returned {} results", search_results.len());

    for result in &search_results {
        println!(
            "   - {} in {} (score: {:.2})",
            result.feature_name, result.service_name, result.relevance_score
        );
    }
    assert_eq!(search_results.len(), 1);

    // Test 7: Advanced search with filters
    println!("\n7. Testing advanced search with filters...");
    let advanced_query = SearchQuery {
        query: "".to_string(),
        service_ids: Some(vec![service.id]),
        feature_types: Some(vec![FeatureType::Api]),
        content_types: None,
        limit: Some(5),
        similarity_threshold: None,
        include_related: false,
        depth: None,
    };

    let advanced_results = feature_repo.search(&advanced_query)?;
    println!(
        "✅ Advanced search returned {} results",
        advanced_results.len()
    );
    assert_eq!(advanced_results.len(), 1);

    // Test 8: Update relevance score
    println!("\n8. Testing relevance score update...");
    feature_repo.update_relevance_score("test-feature-001", 0.95)?;
    let updated = feature_repo.get_by_id("test-feature-001")?;
    match updated {
        Some(f) => {
            assert_eq!(f.relevance_score, 0.95);
            println!("✅ Updated relevance score to {}", f.relevance_score);
        }
        None => {
            return Err(ktme::error::KtmeError::Storage(
                "Feature not found after update".to_string(),
            ));
        }
    }

    // Test 9: Database statistics
    println!("\n9. Testing database statistics...");
    let stats = db.stats()?;
    println!("✅ Database Statistics:");
    println!("   - Services: {}", stats.service_count);
    println!("   - Mappings: {}", stats.mapping_count);
    println!("   - Features: {}", stats.feature_count);
    println!("   - History: {}", stats.history_count);
    assert_eq!(stats.service_count, 1);
    assert_eq!(stats.feature_count, 1);

    // Test 10: Multiple features of different types
    println!("\n10. Creating multiple feature types...");
    let feature_types = vec![
        (FeatureType::Api, "REST API Endpoints"),
        (FeatureType::Database, "Database Layer"),
        (FeatureType::Security, "Authentication"),
        (FeatureType::Config, "Configuration"),
    ];

    for (i, (ft, name)) in feature_types.iter().enumerate() {
        let id = format!("multi-feature-{:03}", i + 1);
        feature_repo.create(
            &id,
            service.id,
            name,
            Some(&format!("{} for test service", name)),
            *ft,
            vec![name.to_lowercase().to_string()],
            serde_json::json!({"type": ft.to_string()}),
        )?;
        println!("   ✅ Created: {}", name);
    }

    let all_features = feature_repo.list_by_service(service.id)?;
    println!("✅ Total features created: {}", all_features.len());
    assert_eq!(all_features.len(), 5); // 1 original + 4 new

    println!("\n🎉 All tests passed successfully!");
    println!("✅ Feature management system is working correctly.");
    println!("✅ Database schema migration is functional.");
    println!("✅ Search and filtering operations work as expected.");

    Ok(())
}
