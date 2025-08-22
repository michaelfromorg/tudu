use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
// use std::env;
use std::fs;
use std::path::{PathBuf,Path};
use std::process;

#[derive(Debug, Clone)]
struct TodoItem {
    file_path: PathBuf,
    line_number: usize,
    line_content: String,
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
            });
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
