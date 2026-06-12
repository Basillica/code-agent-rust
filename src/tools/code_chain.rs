use crate::orchestrator::models::RefactorPlan;
use crate::tools::Tool;
use crate::{orchestrator::engine::RefactorOrchestrator, state::session::SessionContext};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CodeGenerationChainTool {
    // Shared ref thread state matching typical engine layout setups
    session_ctx: Arc<Mutex<SessionContext>>,
}

impl CodeGenerationChainTool {
    pub fn new() -> Self {
        Self {
            session_ctx: Arc::new(Mutex::new(SessionContext::new("project_root"))),
        }
    }
}

#[async_trait]
impl Tool for CodeGenerationChainTool {
    fn name(&self) -> &'static str {
        "apply_multi_file_refactor"
    }

    fn description(&self) -> &'static str {
        "Executes a multi-file code generation and editing sequence. Automatically manages atomic rollbacks if verification checks fail."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "plan": {
                    "type": "object",
                    "description": "The complete execution plan mapping tasks and their application order sequence structural dependencies."
                },
                "verification_command": {
                    "type": "string",
                    "description": "The terminal check instruction used to validate changes (e.g., 'cargo check' or 'cargo test')."
                }
            },
            "required": ["plan", "verification_command"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let plan_val = args
            .get("plan")
            .ok_or("Missing plan configuration object.")?;
        let plan: RefactorPlan = serde_json::from_value(plan_val.clone())
            .map_err(|e| format!("Invalid plan schema payload format: {}", e))?;

        let verify_cmd = args
            .get("verification_command")
            .and_then(|v| v.as_str())
            .unwrap_or("cargo check");

        let mut lock = self.session_ctx.lock().await;
        let mut orchestrator = RefactorOrchestrator::new(&mut *lock);

        match orchestrator.execute_chain(plan, verify_cmd).await {
            Ok(_) => Ok(json!({ "status": "success", "message": "All refactoring operations completed cleanly." }).to_string()),
            Err(e) => Err(format!("Refactoring sequence failed: {}", e)),
        }
    }
}
