use crate::action::subagent::{SubagentOrchestrator, SubagentTaskArgs};
use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;

pub struct DelegateSubtaskTool;
#[async_trait]
impl Tool for DelegateSubtaskTool {
    fn name(&self) -> &'static str {
        "delegate_subtask"
    }
    fn description(&self) -> &'static str {
        "Spawns an isolated subagent runtime to explore, search, or process deep contextual subtasks."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "subtask_objective": { "type": "string" },
                "context_hint": { "type": "string" }
            },
            "required": ["subtask_objective", "context_hint"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, String> {
        let subtask_args: SubagentTaskArgs = serde_json::from_value(args.clone())
            .map_err(|e| format!("Schema Validation Failure: {}", e))?;

        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        match SubagentOrchestrator::run_subtask(&subtask_args, &current_dir).await {
            Ok(summary) => Ok(format!(
                "### [Delegation Return Frame]\nSubagent has returned with results:\n{}",
                summary
            )),
            Err(e) => Err(format!("Error executing subtask delegation: {}", e)),
        }
    }
}
