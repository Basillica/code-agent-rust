use crate::action::permissions::{PermissionGate, PermissionMode};
use crate::orchestrator::autonomous::AutonomousOrchestrator;
use crate::orchestrator::controller::AgentPromptController;
use crate::orchestrator::ui::{TerminalUI, UIStage};
use crate::state::session::SessionContext;
use crate::tools::code_chain::CodeGenerationChainTool;
use crate::tools::registry::ToolRegistry;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub enum ExecutionStrategy {
    /// Pure autonomous discovery (Claude Code style tool-by-tool exploration)
    AutonomousAgent,
    /// Upfront structural topological dependency plan applied through the loop
    UpfrontGraphPlan,
}

pub struct CoreEngineRunner {
    pub project_root: PathBuf,
    pub api_key: String,
    pub compilation_cmd: String,
    pub test_cmd: Option<String>,
}

impl CoreEngineRunner {
    pub fn new(
        project_root: PathBuf,
        api_key: String,
        compilation_cmd: String,
        test_cmd: Option<String>,
    ) -> Self {
        Self {
            project_root,
            api_key,
            compilation_cmd,
            test_cmd,
        }
    }

    pub async fn dispatch_request(
        &self,
        user_prompt: &str,
        strategy: ExecutionStrategy,
        permission_mode: PermissionMode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Initialize thread-safe session metrics
        let mut session = SessionContext::new(self.project_root.clone());
        if let Ok(guidelines) = std::fs::read_to_string(self.project_root.join("AGENT.md")) {
            session.project_instructions = guidelines;
        }
        let shared_session = Arc::new(Mutex::new(session));
        let permission_gate = PermissionGate::new(permission_mode);

        // 2. Populate capabilities framework registry
        let mut registry = ToolRegistry::new(&self.project_root);
        registry.register(CodeGenerationChainTool {
            session_ctx: shared_session.clone(),
        });
        let shared_registry = Arc::new(registry);

        // 3. Match and route based on specified workflow tactics
        match strategy {
            ExecutionStrategy::AutonomousAgent => {
                TerminalUI::print_status(
                    UIStage::GraphSorting,
                    "Starting execution workspace in Autonomous Agent mode...",
                );

                // Instantiate our cleaner object architecture
                let orchestrator = AutonomousOrchestrator::new(
                    shared_session.clone(),
                    shared_registry.clone(),
                    "gemma4:e4b".to_string(),
                );

                // Transfer control over to the ReAct execution matrix
                orchestrator
                    .execute_goal(user_prompt, permission_mode)
                    .await?;
            }

            ExecutionStrategy::UpfrontGraphPlan => {
                TerminalUI::print_status(
                    UIStage::GraphSorting,
                    "Starting execution workspace in Upfront Graph Plan mode...",
                );

                let controller = AgentPromptController::new(
                    shared_session,
                    shared_registry,
                    self.api_key.clone(),
                );

                controller
                    .dispatch_user_goal(
                        user_prompt,
                        &self.compilation_cmd,
                        self.test_cmd.as_deref(),
                        permission_mode,
                    )
                    .await
                    .map_err(|err_msg| std::io::Error::new(std::io::ErrorKind::Other, err_msg))?;
            }
        }

        Ok(())
    }
}
