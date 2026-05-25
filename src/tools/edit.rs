use crate::orchestrator::models::Hunk;
use crate::orchestrator::patch::PatchEngine;
use crate::orchestrator::transaction::WorkspaceTransaction;
use crate::tools::Tool;
use crate::tools::utils::{append_to_codebase_memory_log, verify_workspace_health};
use async_trait::async_trait;
use serde_json::Value;
use std::fs::{self};
use std::path::Path;
use std::path::PathBuf;

pub struct SurgicalEditTool;

#[async_trait]
impl Tool for SurgicalEditTool {
    fn name(&self) -> &'static str {
        "surgical_edit"
    }
    fn description(&self) -> &'static str {
        "Edits an existing file using an explicit search and replace string block."
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
        let path_str = args["path"].as_str().ok_or("Missing field 'path'")?;
        let target_path = PathBuf::from(path_str);

        let hunk = Hunk {
            search_block: args["search"].as_str().unwrap_or_default().to_string(),
            replace_block: args["replace"].as_str().unwrap_or_default().to_string(),
        };

        // --- 🟢 APPLICATION OF MODULE: transaction.rs ---
        // Snapshot the file to memory before writing bytes to protect against workspace corruption
        let mut tx = WorkspaceTransaction::new();
        tx.stage_file(&target_path)?;

        // --- 🟢 APPLICATION OF MODULE: patch.rs ---
        // Perform line-aware sliding-window matches instead of a basic flat string substitution
        match PatchEngine::apply_surgical_patch(&target_path, &[hunk]) {
            Ok(_) => {
                tx.commit(); // Keep modifications safe on disk
                Ok(format!("Surgical edit applied cleanly to `{}`", path_str))
            }
            Err(e) => {
                tx.rollback()?; // Instant zero-pollution restoration if string anchors mismatch!
                Err(format!("Patch execution rejected: {}", e))
            }
        }
    }
}

impl SurgicalEditTool {
    async fn executer(&self, args: &Value) -> Result<String, String> {
        let path_str = args["path"].as_str().ok_or("Missing parameter 'path'")?;
        let search = args["search"]
            .as_str()
            .ok_or("Missing parameter 'search'")?;
        let replace = args["replace"]
            .as_str()
            .ok_or("Missing parameter 'replace'")?;

        if !Path::new(path_str).exists() {
            return Err("Error: File not found.".to_string());
        }
        let old_content = fs::read_to_string(path_str).map_err(|e| e.to_string())?;

        let occurrences = old_content.split(search).count() - 1;
        if occurrences == 0 {
            return Err(
                "Error: SEARCH block target not found in file. Ensure exact match.".to_string(),
            );
        }
        if occurrences > 1 {
            return Err(format!(
                "Error: SEARCH block is ambiguous; matched {} places.",
                occurrences
            ));
        }

        let updated_content = old_content.replace(search, replace);
        fs::write(path_str, updated_content).map_err(|e| e.to_string())?;
        append_to_codebase_memory_log(path_str, "MODIFY");

        // Self-Healing compiler hook check
        let (healthy, error_log) = verify_workspace_health().await;
        if !healthy {
            // Rollback instantly to protect user workspace bounds from pollution
            let _ = fs::write(path_str, old_content);
            return Ok(format!(
                "### [PATCH REJECTED BY SELF-HEALING ENGINE]\nYour edit broke the compilation build! The file was safely rolled back to its original state.\n\nCompiler Error Output:\n{}\n\nPlease analyze your 'new_string' code layout, fix the syntax error, and try the edit again.",
                error_log.unwrap_or_default()
            ));
        }

        Ok("File successfully updated via surgical replacement.".to_string())
    }
}
