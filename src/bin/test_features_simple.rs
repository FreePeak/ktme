//! Simple test for KTME feature management using in-memory database

use ktme::storage::database::Database;
use ktme::storage::repository::{ServiceRepository, FeatureRepository};
use ktme::storage::models::{FeatureType, SearchQuery};
use ktme::error::Result;

fn main() -> Result<()> {
    println!("🧪 Testing KTME Feature Management (In-Memory)");

    // Create in-memory database
    println!("\n1. Creating in-memory database...");
    let db = Database::in_memory()?;
    println!("✅ In-memory database created successfully");

    // Test service creation
    println!("\n2. Testing Service Creation...");
    let service_repo = ServiceRepository::new(db.clone());
    let service = service_repo.create(
        "test-service",
        Some("/test/path"),
        Some("Test service for feature management")
    )?;
    println!("✅ Created service: {} (ID: {})", service.name, service.id);

    // Test feature creation
    println!("\n3. Testing Feature Creation...");
    let feature_repo = FeatureRepository::new(db.clone());
    let feature = feature_repo.create(
        "feature-001",
        service.id,
        "AI Documentation Generation",
        Some("Automatic documentation generation from code changes using AI"),
        FeatureType::Api,
        vec!["ai".to_string(), "documentation".to_string()],
        serde_json::json!({"complexity": "high", "status": "active"}),
    )?;
    println!("✅ Created feature: {} (ID: {})", feature.name, feature.id);
    println!("   Type: {:?}", feature.feature_type);
    println!("   Tags: {:?}", feature.tags);
    println!("   Relevance Score: {}", feature.relevance_score);

    // Test feature retrieval
    println!("\n4. Testing Feature Retrieval...");
    let retrieved_feature = feature_repo.get_by_id("feature-001")?;
    match retrieved_feature {
        Some(f) => {
            println!("✅ Retrieved feature: {}", f.name);
            assert_eq!(f.name, "AI Documentation Generation");
        }
        None => {
            println!("❌ Failed to retrieve feature");
            return Err(ktme::error::KtmeError::Storage("Feature not found".to_string()));
        }
    }

    // Test feature list by service
    println!("\n5. Testing Feature List by Service...");
    let features = feature_repo.list_by_service(service.id)?;
    println!("✅ Retrieved {} features for service", features.len());
    assert_eq!(features.len(), 1);

    // Test feature search
    println!("\n6. Testing Feature Search...");
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
    assert_eq!(search_results.len(), 1);

    for result in &search_results {
        println!("   - {} in {}: {:.2}", result.feature_name, result.service_name, result.relevance_score);
        assert_eq!(result.feature_name, "AI Documentation Generation");
        assert_eq!(result.service_name, "test-service");
    }

    // Test advanced search with filters
    println!("\n7. Testing Advanced Search with Filters...");
    let advanced_query = SearchQuery {
        query: "ai".to_string(),
        service_ids: Some(vec![service.id]),
        feature_types: Some(vec![FeatureType::Api]),
        content_types: None,
        limit: Some(5),
        similarity_threshold: None,
        include_related: false,
        depth: None,
    };

    let advanced_results = feature_repo.search(&advanced_query)?;
    println!("✅ Advanced search returned {} results", advanced_results.len());
    assert_eq!(advanced_results.len(), 1);

    // Test relevance score update
    println!("\n8. Testing Relevance Score Update...");
    feature_repo.update_relevance_score("feature-001", 0.95)?;
    let updated_feature = feature_repo.get_by_id("feature-001")?;
    match updated_feature {
        Some(f) => {
            assert_eq!(f.relevance_score, 0.95);
            println!("✅ Updated relevance score to {}", f.relevance_score);
        }
        None => {
            println!("❌ Failed to retrieve updated feature");
            return Err(ktme::error::KtmeError::Storage("Feature not found after update".to_string()));
        }
    }

    // Test database statistics
    println!("\n9. Testing Database Statistics...");
    let stats = db.stats()?;
    println!("✅ Database Statistics:");
    println!("   - Services: {}", stats.service_count);
    println!("   - Features: {}", stats.feature_count);
    assert_eq!(stats.service_count, 1);
    assert_eq!(stats.feature_count, 1);

    println!("\n🎉 All tests passed! Feature management is working correctly.");
    println!("✅ Database schema with features is working as expected.");

    Ok(())
}