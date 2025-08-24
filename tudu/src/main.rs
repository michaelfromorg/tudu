use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf,Path};
use std::process;

#[derive(Debug, Clone)]
enum TodoRef {
    Untracked,                    // Plain TODO: without ID
    Tracked(String),              // TODO(TASK-123): 
    New { title: Option<String> } // TODO(new="Create user service"):
}

#[derive(Debug, Clone)]
struct TodoAttrs {
    priority: Option<String>, // e.g., "high", "medium", "low"
    tags: Vec<String>,        // e.g., ["backend", "urgent"]
}

#[derive(Debug, Clone)]
struct TodoItem {
    file_path: PathBuf,
    line_number: usize,
    line_content: String,
    reference: Option<TodoRef>,
    attrs: Option<TodoAttrs>
}

#[derive(Parser)]
struct Args {
    /// File or directory to scan
    #[arg(value_name = "PATH")]
    path: PathBuf,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if !args.path.exists() {
        eprintln!("Error: Path '{}' does not exist.", args.path.display());
        process::exit(1);
    }

    let mut all_todos = Vec::new();

    if args.path.is_file() {
        scan_file(&args.path, &mut all_todos);
    } else if args.path.is_dir() {
        scan_directory(&args.path, &mut all_todos);
    } else {
        eprintln!("Error: '{}' is neither a file nor a directory.", args.path.display());
        process::exit(1);
    }

    process_results(&all_todos);

    print_results(&all_todos, args.verbose);    
}

fn scan_file(file_path: &Path, todos: &mut Vec<TodoItem>) {
    let filename = file_path.to_str().unwrap_or("unknown file");
    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(error) => {
            match error.kind() {
                std::io::ErrorKind::NotFound => {
                    eprintln!("File '{}' not found", filename);
                }
                std::io::ErrorKind::PermissionDenied => {
                    eprintln!("Permission denied reading '{}'", filename);
                }
                _ => {
                    eprintln!("Error reading file '{}': {}", filename, error);
                }
            }
            process::exit(1);
        }
    };

    find_todos_in_content(&contents, file_path, todos);
}

fn scan_directory(dir_path: &Path, todos: &mut Vec<TodoItem>) {
    let walker = ignore::WalkBuilder::new(dir_path)
        .add_custom_ignore_filename(".tuduignore")
        .build();

    for result in walker {
        let entry = match result {
            Ok(entry) => entry,
            Err(error) => {
                eprintln!("Error reading directory '{}': {}", dir_path.display(), error);
                continue;
            }
        };

        if entry.file_type().map_or(false, |ft| ft.is_file()) {
            if should_scan_file(entry.path()) {
                scan_file(entry.path(), todos);
            }
        }
    }
}

fn should_scan_file(path: &Path) -> bool {
    // Check file extension
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(ext,
                "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "h" | 
                "go" | "rb" | "php" | "swift" | "kt" | "scala" | "cs" |
                "sh" | "bash" | "zsh" | "yaml" | "yml" | "toml" | "md" |
                "html" | "css" | "scss" | "less"
            )
        }).unwrap_or(false)
}

fn find_todos_in_content(contents: &str, file_path: &Path, todos: &mut Vec<TodoItem>) {
    // This matches
    // - // TODO
    // - /* TODO */
    // - # TODO
    // - <!-- TODO
    let todo_regex = Regex::new(r"(?i)(//|/\*|#|<!--)\s*(TODO|FIXME).*").unwrap();

    for (line_number, line) in contents.lines().enumerate() {
        if todo_regex.is_match(line) {
            todos.push(TodoItem {
                file_path: file_path.to_path_buf(),
                line_number: line_number + 1,
                line_content: line.trim().to_string(),
                reference: parse_todo_reference(line),
                attrs: None,
            });
        }
    }
}

fn parse_todo_reference(line: &str) -> Option<TodoRef> {
    if !line.contains("TODO") && !line.contains("FIXME") {
        return None;
    }

    // Check if it has parentheses
    if let Some(open_paren) = line.find('(') {
        if let Some(close_paren) = line.find(')') {
            // Make sure ) comes after (
            if close_paren > open_paren {
                let inside = &line[open_paren + 1..close_paren].trim();

                // Check for comma (attributes present)
                if let Some(comma_pos) = inside.find(',') {
                    let id_part = &inside[..comma_pos].trim();
                    if is_valid_id(id_part) {
                        // TODO: Parse attributes later
                        return Some(TodoRef::Tracked(id_part.to_string()));
                    }
                } else if is_valid_id(inside) {
                    // Just an ID, no attributes
                    return Some(TodoRef::Tracked(inside.to_string()));
                }

                // Has parens but not a valid ID - treat as untracked
                // This handles TODO(john), TODO(wip), etc.
                return Some(TodoRef::Untracked);
            }
        }
    }

    Some(TodoRef::Untracked)
}

#[cfg(test)]
mod parse_todo_reference_tests {
    use super::*;

    #[test]
    fn test_parse_untracked() {
        let line = "// TODO: Refactor this function";
        let result = parse_todo_reference(line);
        match result {
            Some(TodoRef::Untracked) => {}
            _ => panic!("Expected Untracked"),
        }
    }

    #[test]
    fn test_parse_tracked() {
        let line = "// TODO(BUG-123): Refactor this function";
        let result = parse_todo_reference(line);
        match result {
            Some(TodoRef::Tracked(id)) => {
                assert_eq!(id, "BUG-123");
            }
            _ => panic!("Expected Tracked with ID BUG-123"),
        }
    }

    #[test]
    fn test_parse_with_attributes() {
        let line = "// TODO(TASK-123, bidir): Implement feature";
        let result = parse_todo_reference(line);
        match result {
            Some(TodoRef::Tracked(id)) => {
                assert_eq!(id, "TASK-123");
            }
            _ => panic!("Expected Tracked with ID TASK-123"),
        }
    }

    #[test]
    fn test_parse_empty_parens() {
        let line = "// TODO(): Empty parens";
        let result = parse_todo_reference(line);
        assert!(matches!(result, Some(TodoRef::Untracked)));
    }

    #[test]
    fn test_parse_person_name() {
        let line = "// TODO(alice): Review this";
        let result = parse_todo_reference(line);
        assert!(matches!(result, Some(TodoRef::Untracked)));
    }
}

fn is_valid_id(s: &str) -> bool {
    // Something of the form ABC-123 (at least one letter, a dash, at least one digit)
    // TODO(michaelfromyeg): stop compiling regex every time
    let id_regex = Regex::new(r"^[A-Z]+-\d+$").unwrap();
    id_regex.is_match(s)
}

#[cfg(test)]
mod is_valid_id_tests {
    use super::*;

    #[test]
    fn test_valid_ids() {
        let valid_ids = vec!["TASK-1", "BUG-123", "FEATURE-4567"];
        for id in valid_ids {
            assert!(is_valid_id(id), "Expected '{}' to be valid", id);
        }
    }

    #[test]
    fn test_invalid_ids() {
        let invalid_ids = vec!["task-1", "BUG123", "FEATURE_", "123-ABC", "BUG-"];
        for id in invalid_ids {
            assert!(!is_valid_id(id), "Expected '{}' to be invalid", id);
        }
    }
}

fn process_results(todos: &[TodoItem]) {
    for todo in todos {
        match &todo.reference {
            Some(TodoRef::Untracked) => println!("Not synced"),
            Some(TodoRef::Tracked(id)) => println!("Tracking issue {}", id),
            Some(TodoRef::New { title }) => println!("Will create: {:?}", title),
            None => println!("No reference found"),
        }
    }
}

fn print_results(todos: &[TodoItem], verbose: bool) {
    if todos.is_empty() {
        println!("No TODOs found.");
        return;
    }

    println!("\nFound {} TODOs:", todos.len());

    let mut todos_by_file: HashMap<&PathBuf, Vec<&TodoItem>> = HashMap::new();

    for todo in todos {
        todos_by_file
            .entry(&todo.file_path)
            .or_insert_with(Vec::new)
            .push(todo);
    }

    let mut sorted_files: Vec<_> = todos_by_file.keys().collect();
    sorted_files.sort();

    for file_path in sorted_files {
        let file_todos = &todos_by_file[file_path];
        println!("📁 {}:", file_path.display());

        for todo in file_todos {
            if verbose {
                println!("  Line {}: {}", todo.line_number, todo.line_content);
            } else {
                println!("  Line {}", todo.line_number);
            }
        }
        println!();
    }

    println!("Total: {} TODOs across {} file(s)", 
            todos.len(), 
            todos_by_file.len());
}
