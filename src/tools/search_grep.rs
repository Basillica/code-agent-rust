use crate::tools::Tool;
use async_trait::async_trait;
use regex::Regex;
use serde_json::Value;

pub struct SearchGrepTool;

#[async_trait]
impl Tool for SearchGrepTool {
    fn name(&self) -> &'static str {
        "search_grep"
    }

    fn description(&self) -> &'static str {
        "Searches the workspace files for text matches using a regex pattern. Returns file paths, line numbers, and matching text snippets. Use this to locate where features, variables, or functions are defined."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "search": { "type": "string" },
                "replace": { "type": "string" }
            },
            "required": ["path", "search", "replace"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let pattern_str = args["pattern"]
            .as_str()
            .ok_or_else(|| "Missing required parameter: 'pattern'".to_string())?;

        let max_results = args["max_results"].as_u64().unwrap_or(100) as usize;

        let re = Regex::new(pattern_str).map_err(|e| format!("Invalid regex pattern: {}", e))?;

        let mut matches = Vec::new();

        // Walk through the workspace directory, ignoring common build artifact folders
        for entry in walkdir::WalkDir::new(".")
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != "target" && name != "node_modules" && name != ".git" && name != ".github"
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                // Read the file contents safely
                if let Ok(content) = std::fs::read_to_string(path) {
                    for (line_idx, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            matches.push(format!(
                                "{}:{}: {}",
                                path.display(),
                                line_idx + 1,
                                line.trim()
                            ));

                            if matches.len() >= max_results {
                                break;
                            }
                        }
                    }
                }
            }
            if matches.len() >= max_results {
                break;
            }
        }

        if matches.is_empty() {
            Ok("No matches found for the specified pattern.".to_string())
        } else {
            Ok(matches.join("\n"))
        }
    }
}
