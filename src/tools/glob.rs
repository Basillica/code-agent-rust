use crate::tools::Tool;
use async_trait::async_trait;
use glob::glob;
use serde_json::Value;

pub struct GlobTool;
#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &'static str {
        "glob"
    }
    fn description(&self) -> &'static str {
        "Find files in the project directory using pattern matching (e.g., 'src/**/*.ts')."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": { "pattern": { "type": "string" } },
            "required": ["pattern"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, String> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or("Missing parameter 'pattern'")?;
        let mut matches = Vec::new();

        match glob(pattern) {
            Ok(paths) => {
                for entry in paths.filter_map(Result::ok) {
                    if entry.is_file() {
                        matches.push(entry.to_string_lossy().into_owned());
                    }
                }
                if matches.is_empty() {
                    Ok("No matching files discovered.".to_string())
                } else {
                    Ok(matches.join("\n"))
                }
            }
            Err(e) => Err(format!("Glob search failure: {}", e)),
        }
    }
}
