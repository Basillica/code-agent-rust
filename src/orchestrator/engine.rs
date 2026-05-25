use crate::orchestrator::diagnostic::DiagnosticParser;
use crate::orchestrator::models::{EditStatus, RefactorPlan};
use crate::orchestrator::patch::PatchEngine;
use crate::orchestrator::transaction::WorkspaceTransaction;
use crate::orchestrator::ui::{TerminalUI, UIStage};
use crate::state::session::SessionContext;
use std::path::PathBuf;
use std::process::Command;

pub struct RefactorOrchestrator<'a> {
    session: &'a mut SessionContext,
}

impl<'a> RefactorOrchestrator<'a> {
    pub fn new(session: &'a mut SessionContext) -> Self {
        Self { session }
    }

    /// Drives the structured execution lifecycle for broad refactoring operations with real-time UI tracking
    pub async fn execute_chain(
        &mut self,
        plan: RefactorPlan,
        compilation_cmd: &str,
    ) -> Result<(), String> {
        let mut tx = WorkspaceTransaction::new();
        let mut plan_state = plan;

        // --- Phase 1: Topological Dependency Sorting ---
        TerminalUI::print_status(
            UIStage::GraphSorting,
            "Analyzing cross-module dependencies...",
        );

        let active_files: Vec<PathBuf> = plan_state
            .task_graph
            .values()
            .map(|t| t.target_file.clone())
            .collect();

        // Use our real WorkspaceGraph module
        let graph = crate::orchestrator::graph::WorkspaceGraph::build_from_workspace(
            &self.session.project_root,
            &active_files,
        );

        let safe_execution_order = match graph.resolve_execution_order() {
            Ok(ordered_paths) => {
                let mut sorted_ids = Vec::new();
                for path in ordered_paths {
                    if let Some((id, _)) = plan_state
                        .task_graph
                        .iter()
                        .find(|(_, t)| t.target_file == path)
                    {
                        sorted_ids.push(id.clone());
                    }
                }
                for id in &plan_state.execution_order {
                    if !sorted_ids.contains(id) {
                        sorted_ids.push(id.clone());
                    }
                }
                sorted_ids
            }
            Err(e) => {
                TerminalUI::print_status(UIStage::GraphSorting, &format!("Notice: {}", e));
                plan_state.execution_order.clone()
            }
        };

        std::thread::sleep(std::time::Duration::from_millis(200));

        // --- Phase 2: Open Transaction Safety Buffers ---
        TerminalUI::print_status(
            UIStage::BackupLock,
            "Stashing original file states to memory safety layers...",
        );

        for id in &safe_execution_order {
            if let Some(task) = plan_state.task_graph.get(id) {
                let full_path = self.session.project_root.join(&task.target_file);
                tx.stage_file(&full_path)?;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(200));

        // --- Phase 3: Surgical Line Patching ---
        TerminalUI::print_status(
            UIStage::SurgicalPatching,
            "Applying structured search-and-replace hunks...",
        );

        let total_patches = safe_execution_order.len();
        for (index, id) in safe_execution_order.iter().enumerate() {
            if let Some(mut current_task) = plan_state.task_graph.get(id).cloned() {
                let absolute_target_path =
                    self.session.project_root.join(&current_task.target_file);

                TerminalUI::draw_progress(
                    index + 1,
                    total_patches,
                    &current_task.target_file.to_string_lossy(),
                );

                // Ensure snapshot is verified/tracked inside the safety frame
                tx.stage_file(&absolute_target_path)?;

                let parsed_hunks = PatchEngine::parse_patches(&current_task.patch_instructions);

                if parsed_hunks.is_empty() {
                    tx.rollback()?;
                    return Err(format!(
                        "Failed to parse any valid <<<<<<< SEARCH blocks inside task instruction for [{}].",
                        id
                    ));
                }

                match PatchEngine::apply_patches_to_file(&absolute_target_path, &parsed_hunks) {
                    Ok(_) => {
                        current_task.status = EditStatus::Applied;
                        plan_state.task_graph.insert(id.clone(), current_task);
                    }
                    Err(err_msg) => {
                        tx.rollback()?;
                        return Err(format!(
                            "Aborting refactor chain at task [{}]: {}",
                            id, err_msg
                        ));
                    }
                }
            }
        }

        // --- Phase 4: Verification Testing & Diagnostics ---
        TerminalUI::print_status(
            UIStage::Verification,
            &format!("Invoking `{}` pipeline validation...", compilation_cmd),
        );

        if let Err(diagnostic_report) = self.verify_workspace_with_diagnostics(compilation_cmd) {
            TerminalUI::print_status(
                UIStage::Failure,
                "Workspace assertions broken. Executing rollbacks.",
            );
            tx.rollback()?;

            println!("\n{}", diagnostic_report);
            return Err(diagnostic_report);
        }

        // Phase 5: Success Commit
        tx.commit();
        TerminalUI::print_status(
            UIStage::Success,
            "All code generations completely compiled and verified.",
        );

        self.session.save_persistent_memory(
            &format!(
                "Executed multi-file refactoring chain containing steps [{}] successfully verified via '{}'.", 
                safe_execution_order.join(", "), 
                compilation_cmd
            )
        );

        Ok(())
    }

    async fn apply_patch(
        &self,
        task: &crate::orchestrator::models::FileEditTask,
    ) -> Result<(), String> {
        let absolute_target = self.session.project_root.join(&task.target_file);

        if !absolute_target.parent().unwrap().exists() {
            std::fs::create_dir_all(absolute_target.parent().unwrap())
                .map_err(|e| format!("Failed constructing directory layout: {}", e))?;
        }

        // 1. If target file doesn't exist yet, standard generation path applies
        if !absolute_target.exists() {
            std::fs::write(&absolute_target, &task.patch_instructions)
                .map_err(|e| format!("Failed initializing missing file framework: {}", e))?;
            return Ok(());
        }

        // 2. If file is present, parse structural hunks out of instructions context
        let hunks = PatchEngine::parse_patches(&task.patch_instructions);

        if hunks.is_empty() {
            // Fallback: If no search blocks are found, treat the prompt as a clean baseline write overwrite
            std::fs::write(&absolute_target, &task.patch_instructions)
                .map_err(|e| format!("Failed writing raw fallback output context: {}", e))?;
        } else {
            // Apply surgical diff hunks dynamically using our execution strategies
            PatchEngine::apply_patches_to_file(&absolute_target, &hunks)?;
        }

        Ok(())
    }

    pub fn verify_workspace_with_diagnostics(&self, base_command: &str) -> Result<(), String> {
        let shell_exec = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "sh"
        };
        let flag = if cfg!(target_os = "windows") {
            "/C"
        } else {
            "-c"
        };

        let mut final_command = base_command.to_string();
        if final_command.starts_with("cargo ") && !final_command.contains("--message-format") {
            final_command.push_str(" --message-format=json");
        }

        let output = Command::new(shell_exec)
            .current_dir(&self.session.project_root)
            .args([flag, &final_command])
            .output()
            .map_err(|e| format!("Failed executing validation command environment: {}", e))?;

        if output.status.success() {
            return Ok(());
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let stderr_str = String::from_utf8_lossy(&output.stderr);

        let mut errors = DiagnosticParser::parse_cargo_json(&stdout_str);
        if errors.is_empty() {
            errors = DiagnosticParser::parse_cargo_json(&stderr_str);
        }

        if !errors.is_empty() {
            Err(DiagnosticParser::format_errors_for_llm(&errors))
        } else {
            let combined_error = format!("{}\n{}", stdout_str, stderr_str);
            let truncated_err = combined_error
                .lines()
                .take(20)
                .collect::<Vec<&str>>()
                .join("\n");
            Err(format!(
                "### 🚨 Build Command Failed:\n```\n{}\n```",
                truncated_err
            ))
        }
    }
}
