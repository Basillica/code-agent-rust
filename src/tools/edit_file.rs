use crate::tools::Tool;
use crate::tools::utils::{append_to_codebase_memory_log, verify_workspace_health};
use async_trait::async_trait;
use serde_json::Value;
use std::fs::{self};
use std::path::Path;

pub struct EditFileTool;
#[async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> &'static str {
        "edit_file"
    }
    fn description(&self) -> &'static str {
        "Surgically modify an existing file using a strict structural search-and-replace block match."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "old_string": { "type": "string" },
                "new_string": { "type": "string" }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, String> {
        let path_str = args["path"].as_str().ok_or("Missing parameter 'path'")?;
        let old_string = args["old_string"]
            .as_str()
            .ok_or("Missing parameter 'old_string'")?;
        let new_string = args["new_string"]
            .as_str()
            .ok_or("Missing parameter 'new_string'")?;

        if !Path::new(path_str).exists() {
            return Err("Error: Target file path does not exist. Use write_file to initialize new codebases.".to_string());
        }

        let original_content = fs::read_to_string(path_str).map_err(|e| e.to_string())?;
        let occurrences = original_content.split(old_string).count() - 1;

        if occurrences == 0 {
            return Ok("PATCH ERROR: The code string provided in 'old_string' could not be found inside the file.".to_string());
        }
        if occurrences > 1 {
            return Ok(format!(
                "PATCH ERROR: Ambiguous target definition! 'old_string' occurs {} times. Provide more unique lines.",
                occurrences
            ));
        }

        let updated_content = original_content.replace(old_string, new_string);
        fs::write(path_str, &updated_content).map_err(|e| e.to_string())?;

        println!(
            "🧪 [Self-Healing Verification]: Verifying build integrity for {}...",
            path_str
        );
        append_to_codebase_memory_log(path_str, "MODIFY");

        let (healthy, error_log) = verify_workspace_health().await;
        if !healthy {
            // Rollback transaction instantly to save the workspace from regression
            let _ = fs::write(path_str, original_content);
            println!(
                "⚠️ [Build Regressed]: Edit introduced a syntax error. Reverted patch cleanly."
            );
            return Ok(format!(
                "### [PATCH REJECTED BY SELF-HEALING ENGINE]\nYour edit broke the compilation build! The file was safely rolled back.\n\nTypeScript Compiler Error Output:\n{}\n\nPlease analyze your 'new_string' code layout, fix the error, and try again.",
                error_log.unwrap()
            ));
        }

        Ok(format!(
            "Successfully applied surgical patch modifications to {}.",
            path_str
        ))
    }
}
