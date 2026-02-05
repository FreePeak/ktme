use std::path::Path;

pub struct TodoItem {
    pub line_number: usize,
    pub content: String,
    pub context: String,
}

pub fn find_todos(docs_dir: &Path) -> std::io::Result<Vec<TodoItem>> {
    let mut todos = Vec::new();

    if let Ok(entries) = std::fs::read_dir(docs_dir) {
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                let content = std::fs::read_to_string(&path)?;
                for (line_num, line) in content.lines().enumerate() {
                    if line.contains("TODO:") {
                        todos.push(TodoItem {
                            line_number: line_num + 1,
                            content: line.to_string(),
                            context: path.file_name().unwrap().to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(todos)
}

pub fn fill_todo(content: &str, _todo: &str, _replacement: &str) -> String {
    content.replace("TODO:", "DONE:")
}
