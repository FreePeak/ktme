use crate::analysis::doc_parser::DocumentNode;
use std::collections::HashMap;

pub struct CoverageReport {
    pub total_files: usize,
    pub total_sections: usize,
    pub documented_items: HashMap<String, usize>,
    pub undocumented_items: Vec<String>,
    pub coverage_percentage: f64,
}

pub struct CodeItem {
    pub name: String,
    pub item_type: String,
    pub line_number: usize,
}

pub fn calculate_coverage(docs: &[DocumentNode], code_items: &[CodeItem]) -> CoverageReport {
    let total_files = docs.len();
    let total_sections: usize = docs.iter().map(|d| d.sections.len()).sum();

    let mut documented_items: HashMap<String, usize> = HashMap::new();
    let mut undocumented_items = Vec::new();

    let documented_sections: std::collections::HashSet<String> = docs
        .iter()
        .flat_map(|d| d.sections.iter().map(|s| s.title.to_lowercase()))
        .collect();

    for item in code_items {
        let normalized_name = item.name.to_lowercase();
        if documented_sections.contains(&normalized_name) {
            *documented_items.entry(item.item_type.clone()).or_insert(0) += 1;
        } else {
            undocumented_items.push(item.name.clone());
        }
    }

    let total_items = code_items.len();
    let coverage_percentage = if total_items > 0 {
        (total_items.saturating_sub(undocumented_items.len()) as f64 / total_items as f64) * 100.0
    } else {
        100.0
    };

    CoverageReport {
        total_files,
        total_sections,
        documented_items,
        undocumented_items,
        coverage_percentage,
    }
}
