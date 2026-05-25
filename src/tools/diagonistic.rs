use crate::tools::{Tool, utils::verify_workspace_health};
use async_trait::async_trait;
use serde_json::Value;

pub struct CheckDiagnosticsTool;
#[async_trait]
impl Tool for CheckDiagnosticsTool {
    fn name(&self) -> &'static str {
        "check_diagnostics"
    }
    fn description(&self) -> &'static str {
        "Run full compilation type-checks across the entire repository to locate any errors."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }
    async fn execute(&self, _args: &Value) -> Result<String, String> {
        let (healthy, error_log) = verify_workspace_health().await;
        if healthy {
            Ok("Clean Bill of Health: The workspace builds completely with zero syntax or type errors. ✓".to_string())
        } else {
            Ok(format!(
                "### [Current Workspace Compilation Failures]\n{}",
                error_log.unwrap()
            ))
        }
    }
}
