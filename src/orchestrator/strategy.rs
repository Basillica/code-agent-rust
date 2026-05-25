use crate::action::permissions::PermissionMode;
use crate::core::main_loop::query_loop;
use crate::orchestrator::controller::AgentPromptController;
use crate::orchestrator::dispatch::ExecutionStrategy;
use crate::orchestrator::ui::{TerminalUI, UIStage};
use crate::state::session::SessionContext;
use crate::tools::registry::ToolRegistry;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CoreEngineRunner {
    pub project_root: PathBuf,
    pub api_key: String,
    pub compilation_cmd: String,
}

impl CoreEngineRunner {
    pub fn new(project_root: PathBuf, api_key: String, compilation_cmd: String) -> Self {
        Self {
            project_root,
            api_key,
            compilation_cmd,
        }
    }

    /// Primary execution dispatcher routing prompts to their corresponding runtime strategies
    pub async fn dispatch_request(
        &self,
        user_prompt: &str,
        strategy: ExecutionStrategy,
        permission_mode: PermissionMode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Initialize the shared structural Session Context state
        let mut session = SessionContext::new(self.project_root.clone());

        // Populate standard system project guidelines (e.g., loading from AGENT.md)
        if let Ok(guidelines) = std::fs::read_to_string(self.project_root.join("AGENT.md")) {
            session.project_instructions = guidelines;
        }

        match strategy {
            ExecutionStrategy::AutonomousAgent => {
                let registry = ToolRegistry::new(&self.project_root);
                TerminalUI::print_status(
                    UIStage::GraphSorting,
                    "Starting execution workspace in Autonomous Agent mode...",
                );

                // Fire up your iterative tool-use supervisor loop!
                // It will independently explore, gate permissions, edit files, and self-heal.
                query_loop(
                    user_prompt,
                    &mut session,
                    permission_mode,
                    &registry,
                    &mut TerminalUI::new(),
                )
                .await?;
            }

            ExecutionStrategy::UpfrontGraphPlan => {
                let registry = ToolRegistry::new(&self.project_root);
                let registry_arc = Arc::new(registry);
                TerminalUI::print_status(
                    UIStage::GraphSorting,
                    "Starting execution workspace in Upfront Graph Plan mode...",
                );

                // Wrap the context in a Mutex as expected by your pre-planned AgentPromptController
                let shared_session = Arc::new(Mutex::new(session));
                let controller =
                    AgentPromptController::new(shared_session, registry_arc, self.api_key.clone());

                // Triggers the upfront LLM structured planning phase, executing
                // the topological dependency graph inside its own transactional wrapper
                controller
                    .dispatch_user_goal(user_prompt, &self.compilation_cmd)
                    .await
                    .map_err(|err_msg| std::io::Error::new(std::io::ErrorKind::Other, err_msg))?;
            }
        }

        Ok(())
    }
}
