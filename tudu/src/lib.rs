use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod providers;

#[derive(Debug, Clone)]
pub enum TodoReference {
    Untracked,                     // Plain TODO: without ID
    Tracked(String),               // TODO(TASK-123):
    New { title: Option<String> }, // TODO(new="Create user service"):
}

#[derive(Debug, Clone, PartialEq)]
pub enum TodoAttributeValue {
    Flag(bool),        // bidir
    Text(String),      // assignee=alice
    List(Vec<String>), // labels=urgent,backend
}

#[derive(Debug, Clone)]
pub struct TodoItem {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub reference: Option<TodoReference>,
    pub attributes: Option<HashMap<String, TodoAttributeValue>>,
}

#[derive(Parser)]
pub struct Args {
    /// File or directory to scan
    #[arg(value_name = "PATH")]
    pub path: PathBuf,

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Output format: standard or json (overrides config)  
    #[arg(long)]
    pub format: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub scan: ScanConfig,
    #[serde(default = "default_mode")]
    pub mode: String,
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(serde::Deserialize, Debug, Default)]
#[serde(default)]
pub struct ScanConfig {
    pub ignore: Vec<String>,
    pub include: Vec<String>,
    pub match_case_insensitive: bool,
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ProviderConfig {
    Notion(NotionConfig),
    Jira(JiraConfig),
    Github(GithubConfig),
}

#[derive(serde::Deserialize, Debug)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default)]
    pub verbose: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct NotionConfig {
    pub database_id: String,
    // other notion-specific fields
}

#[derive(serde::Deserialize, Debug)]
pub struct JiraConfig {
    pub server: String,
    pub project: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct GithubConfig {
    pub owner: String,
    pub repo: String,
}

fn default_mode() -> String {
    "validate".to_string()
}

fn default_format() -> String {
    "standard".to_string()
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            verbose: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan: ScanConfig::default(),
            mode: default_mode(),
            providers: HashMap::new(),
            output: OutputConfig::default(),
        }
    }
}
