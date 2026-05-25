use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::tools::Tool;

#[derive(Deserialize)]
pub struct SearchArgs {
    pub pattern: String,
    pub path_filter: Option<String>,
}

#[derive(Serialize)]
pub struct MatchInstance {
    pub file_path: String,
    pub line_number: usize,
    pub matched_line: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

#[derive(Serialize)]
pub struct SearchResultPayload {
    pub total_matches_found: usize,
    pub matches_returned: Vec<MatchInstance>,
    pub budget_exceeded: bool,
}

pub struct CodebaseSearchTool {
    project_root: PathBuf,
}

impl CodebaseSearchTool {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }
}

#[async_trait]
impl Tool for CodebaseSearchTool {
    fn name(&self) -> &'static str {
        "codebase_search"
    }

    fn description(&self) -> &'static str {
        "Performs an incremental high-speed keyword search across the workspace code directory. Returns matching lines along with surrounding context lines."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The textual query substring or keyword pattern to scan for (e.g., 'auth_utils', 'User struct'). Case-insensitive."
                },
                "path_filter": {
                    "type": "string",
                    "description": "Optional directory or filename substring pattern to scope down search depth (e.g., 'src/models')."
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        // Safe parameter unmarshaling using standard serde formats
        let search_args: SearchArgs = serde_json::from_value(args.clone())
            .map_err(|e| format!("Parsing arguments failed: {}", e))?;

        let target_pattern = search_args.pattern.to_lowercase();
        let filter = search_args.path_filter.unwrap_or_default().to_lowercase();

        let mut matches = Vec::new();
        let mut total_matches = 0;
        let max_matches_budget = 40;
        let context_padding = 2;

        let mut directories_to_visit = vec![self.project_root.clone()];

        while let Some(current_dir) = directories_to_visit.pop() {
            let entries = match fs::read_dir(&current_dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if path.is_dir() {
                    if file_name == ".git"
                        || file_name == "target"
                        || file_name == "node_modules"
                        || file_name == "__pycache__"
                    {
                        continue;
                    }
                    directories_to_visit.push(path);
                    continue;
                }

                if file_name == "Cargo.lock"
                    || file_name == "package-lock.json"
                    || file_name.ends_with(".log")
                {
                    continue;
                }

                let relative_path = path
                    .strip_prefix(&self.project_root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                if !filter.is_empty() && !relative_path.to_lowercase().contains(&filter) {
                    continue;
                }

                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let lines: Vec<String> = reader.lines().flatten().collect();

                    for (idx, line) in lines.iter().enumerate() {
                        if line.to_lowercase().contains(&target_pattern) {
                            total_matches += 1;

                            if matches.len() < max_matches_budget {
                                let start_idx = idx.saturating_sub(context_padding);
                                let end_idx = std::cmp::min(lines.len(), idx + context_padding + 1);

                                let context_before: Vec<String> = lines[start_idx..idx].to_vec();
                                let context_after: Vec<String> = lines[(idx + 1)..end_idx].to_vec();

                                matches.push(MatchInstance {
                                    file_path: relative_path.clone(),
                                    line_number: idx + 1,
                                    matched_line: line.trim().to_string(),
                                    context_before,
                                    context_after,
                                });
                            }
                        }
                    }
                }
            }
        }

        let report = SearchResultPayload {
            total_matches_found: total_matches,
            budget_exceeded: total_matches > max_matches_budget,
            matches_returned: matches,
        };

        serde_json::to_string_pretty(&report)
            .map_err(|e| format!("Failed serializing match structural report: {}", e))
    }
}
