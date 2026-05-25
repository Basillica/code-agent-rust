use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;

pub struct GrepTool;
#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &'static str {
        "grep"
    }
    fn description(&self) -> &'static str {
        "Search for string patterns or regex matches across all files in the repository."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": { "query": { "type": "string" } },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let query = args["query"].as_str().ok_or("Missing parameter 'query'")?;

        // Escape quotes to safeguard execution bounds
        let escaped_query = query.replace('"', "\\\"");
        let output = std::process::Command::new("git")
            .args(["grep", "-n", &escaped_query])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if text.is_empty() {
                    Ok("No pattern matches located in codebase.".to_string())
                } else {
                    Ok(text)
                }
            }
            _ => Ok("No structural matches found for the specified text query.".to_string()),
        }
    }
}
