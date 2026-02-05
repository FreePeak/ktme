use ktme::mcp::tools::McpTools;

fn main() {
    let test_path = "/Users/linh.doan/work/harvey/freepeak/ktme";

    println!("=== Testing MCP Tools ===\n");

    println!("1. scan_documentation:");
    match McpTools::scan_documentation(Some(test_path)) {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("ERROR: {:?}\n", e),
    }

    println!("2. validate_documentation:");
    match McpTools::validate_documentation(Some(test_path)) {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("ERROR: {:?}\n", e),
    }

    println!("3. detect_tech_stack:");
    match McpTools::detect_tech_stack(Some(test_path)) {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("ERROR: {:?}\n", e),
    }

    println!("4. find_documentation_todos:");
    match McpTools::find_documentation_todos(Some(test_path)) {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("ERROR: {:?}\n", e),
    }
}
