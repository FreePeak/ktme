use crate::error::Result;
use crate::storage::mapping::StorageManager;

pub async fn execute(query: String, feature: bool, keyword: bool) -> Result<()> {
    let storage = StorageManager::new()?;

    let results = if feature {
        storage.search_by_feature(&query)?
    } else if keyword {
        storage.search_by_keyword(&query)?
    } else {
        storage.search_services(&query)?
    };

    if results.is_empty() {
        let search_type = if feature {
            "feature"
        } else if keyword {
            "keyword"
        } else {
            "services"
        };
        println!("No {} found matching: {}", search_type, query);
        return Ok(());
    }

    let search_type = if feature {
        "Features"
    } else if keyword {
        "Keywords"
    } else {
        "Services"
    };
    println!("{} matching '{}':\n", search_type, query);

    for (idx, result) in results.iter().enumerate() {
        println!(
            "{}. **{}** (Relevance: {:.1})",
            idx + 1,
            result.name,
            result.relevance_score
        );

        if let Some(ref desc) = result.description {
            println!("   📝 {}", desc);
        }

        if let Some(ref path) = result.path {
            println!("   📁 {}", path);
        }

        if !result.docs.is_empty() {
            println!("   📚 Documentation ({} files):", result.docs.len());
            for doc in &result.docs {
                println!("      • {}", doc);
            }
        }

        if idx < results.len() - 1 {
            println!();
        }
    }

    println!("\n📊 Summary: Found {} result(s)", results.len());

    Ok(())
}
