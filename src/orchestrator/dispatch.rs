use crate::action::permissions::PermissionMode;
use crate::core::main_loop::query_loop;
use crate::orchestrator::controller::AgentPromptController;
use crate::orchestrator::ui::TerminalUI;
use crate::state::session::SessionContext;
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

pub struct EngineDispatcher {
    pub project_root: PathBuf,
    pub api_key: String,
}

impl EngineDispatcher {
    pub fn new(project_root: PathBuf, api_key: String) -> Self {
        Self {
            project_root,
            api_key,
        }
    }

    /// Primary orchestration hub ensuring 100% feature alignment across all strategies
    pub async fn execute_request(
        &self,
        user_prompt: &str,
        strategy: ExecutionStrategy,
        permission_mode: PermissionMode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Initialize your structural Session Context state
        let mut session = SessionContext::new(self.project_root.clone());

        // Read local repository guidelines if present (e.g., AGENT.md verification scripts)
        if let Ok(guidelines) = std::fs::read_to_string(self.project_root.join("AGENT.md")) {
            session.project_instructions = guidelines;
        }

        // 2. Spin up your physical tool registry containing CodebaseSearchTool, EditFileTool, etc.
        let registry = ToolRegistry::new(&self.project_root);

        // 3. Formulate the system prompt or initial loop query based on the strategy
        let final_query = match strategy {
            ExecutionStrategy::AutonomousAgent => {
                // Organic strategy pass-through
                user_prompt.to_string()
            }
            ExecutionStrategy::UpfrontGraphPlan => {
                println!("🧠 [Engine] Generating Upfront Dependency Plan from Controller...");
                let registry = ToolRegistry::new(&self.project_root);
                let registry_arc = Arc::new(registry);

                // Wrap context securely to query your AgentPromptController layout
                let shared_session =
                    Arc::new(Mutex::new(SessionContext::new(self.project_root.clone())));
                let controller =
                    AgentPromptController::new(shared_session, registry_arc, self.api_key.clone());

                // Build the topological roadmap plan upfront
                match controller.generate_upfront_plan(user_prompt).await {
                    Ok(plan) => {
                        // Inject the structured plan directly as an explicit instruction block
                        // instructing the loop to immediately call its multi-file refactoring engine tool.
                        format!(
                            "You are in PLANNED REFACTOR mode. We have pre-calculated an optimized, topological dependency graph plan for this task.\n\n\
                            USER INTENT: {}\n\n\
                            STRICT EXECUTION PLAN GRAPH:\n{}\n\n\
                            INSTRUCTION: Call your tool `apply_multi_file_refactor` immediately passing this exact JSON plan block to execute it through our gated transaction engine. Validate project health upon completion.",
                            user_prompt,
                            serde_json::to_string_pretty(&plan).unwrap_or_default()
                        )
                    }
                    Err(e) => {
                        println!(
                            "⚠️ [Plan Error] Failed generating upfront plan: {}. Falling back to organic agent loop.",
                            e
                        );
                        user_prompt.to_string()
                    }
                }
            }
        };

        // 4. Fire the single execution loop runner.
        // Whether it's organic discovery or applying a pre-parsed structured plan,
        // it goes through this EXACT loop. This guarantees your token truncation budgets,
        // message logging, codebase search tools, and self-healing systems are ALWAYS active.
        query_loop(
            &final_query,
            &mut session,
            permission_mode,
            &registry,
            &mut TerminalUI::new(),
        )
        .await?;

        Ok(())
    }
}
