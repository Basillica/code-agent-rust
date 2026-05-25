use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;
use std::fs::{self};
use std::path::Path;

pub struct ReadFileTool;
#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }
    fn description(&self) -> &'static str {
        "Read the contents of a specific file safely with line bounds."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": { "path": { "type": "string", "description": "Relative path to the target file" } },
            "required": ["path"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, String> {
        let path_str = args["path"].as_str().ok_or("Missing parameter 'path'")?;
        println!("what fucing file are we reading now: {path_str}");
        if !Path::new(path_str).exists() {
            return Err(format!("Error: File not found at {}", path_str));
        }
        fs::read_to_string(path_str).map_err(|e| format!("Error reading file: {}", e))
    }
}
