use crate::{
    action::permissions::{GateResult, PermissionGate},
    orchestrator::graph::WorkspaceGraph,
    state::session::SessionContext,
};
use serde_json::Value;
use std::io::{self, Write};

pub enum UIStage {
    GraphSorting,
    BackupLock,
    SurgicalPatching,
    Verification,
    Success,
    Failure,
}

pub struct TerminalUI {
    pub current_task: String,
    pub current_status: String,
    pub logs: Vec<(String, String)>, // (Source: e.g. "assistant", "system", Message)
    pub active_tool_name: String,
    pub active_tool_args: String,
}

impl TerminalUI {
    pub fn new() -> Self {
        Self {
            current_task: String::from("Idle"),
            current_status: String::from("Awaiting payload..."),
            logs: Vec::new(),
            active_tool_name: String::from("None"),
            active_tool_args: String::from("{}"),
        }
    }

    pub fn start_task(&mut self, objective: &str) {
        self.current_task = objective.to_string();
        self.log_message(
            "system",
            &format!("🚀 Starting execution task: {}", objective),
        );
    }

    pub fn update_status(&mut self, status: &str) {
        self.current_status = status.to_string();
    }

    pub fn log_message(&mut self, sender: &str, text: &str) {
        self.logs.push((sender.to_string(), text.to_string()));
        // Print it immediately or push it onto your alternate screen raw buffer layout
        println!("[{}] {}", sender.to_uppercase(), text);
    }

    pub fn update_active_tool(&mut self, name: &str, args: &Value) {
        self.active_tool_name = name.to_string();
        self.active_tool_args = serde_json::to_string(args).unwrap_or_default();
        self.log_message(
            "system",
            &format!(
                "🛠️ Preparing Tool Call: {} with args: {}",
                name, self.active_tool_args
            ),
        );
    }

    /// Renders a structured layout terminal view frame of the current tracking environment
    pub fn render(&self, _session: &SessionContext, graph: &WorkspaceGraph) {
        println!("\n=================== TERMINAL DASHBOARD VIEW ===================");
        println!("🎯 ACTIVE TASK  : {}", self.current_task);
        println!("📊 CURRENT STATE: {}", self.current_status);
        println!(
            "🛠️ ACTIVE TOOL : {} ({})",
            self.active_tool_name, self.active_tool_args
        );
        println!(
            "🔗 STRUCTURAL GRAPH EDGES CAPTURED: {} nodes",
            graph.nodes.len()
        );
        if !graph.verify_structural_integrity() {
            println!(
                "⚠️ WARNING: {} Dangling/Broken Graph References Detected!",
                graph.broken_edges.len()
            );
        }
        println!("===============================================================\n");
    }

    /// Suspends processing execution threads to collect security approvals directly from stdin
    pub async fn prompt_permission(
        &mut self,
        tool_name: &str,
        args: &Value,
        gate: &PermissionGate,
    ) -> bool {
        let result = match gate.evaluate_tool_policy(tool_name) {
            GateResult::Allowed => true,
            GateResult::Blocked => {
                self.log_message("system", &format!("❌ [Gate Block]: execution of '{}' rejected by project environment constraints.", tool_name));
                return false;
            }
            GateResult::RequiresUserApproval => {
                // 1. Log a clean, formatted intervention notice down your existing UI console stream
                self.log_message("system", "⚠️  [GATE INTERVENTION REQUIRED]");
                self.log_message("system", &format!("👉 Tool Request: \"{}\"", tool_name));
                self.log_message(
                    "system",
                    &format!(
                        "👉 Parameters:   {}",
                        serde_json::to_string(args).unwrap_or_default()
                    ),
                );
                self.log_message("system", "Allow execution? (y/N): ");

                // Print the input question directly to standard output
                print!("Allow execution? (y/N): ");
                let _ = io::stdout().flush();

                // 2. Read standard input synchronously
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() {
                    let approved = input.trim().to_lowercase() == "y";
                    if approved {
                        self.log_message("system", "✅ Action authorized by user.");
                        return true;
                    } else {
                        self.log_message("system", "❌ Action denied by user. Aborting turn.");
                        return false;
                    }
                } else {
                    // If reading stdin fails, default safe to false
                    return false;
                }
            }
        };
        if !result {
            return true;
        }

        // if !gate.requires_approval(tool_name) {
        //     return true;
        // }

        println!("\n🛡️  [SECURITY OVERWRITE PERMISSION REQUEST]");
        println!(
            "   The tool '{}' requires authorized runtime rights to run.",
            tool_name
        );
        print!("   Allow execution sequence? (y/N): ");

        use std::io::{self, Write};
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let answer = input.trim().to_lowercase();
            if answer == "y" || answer == "yes" {
                self.log_message("system", "✓ User approved tool authorization sequence.");
                return true;
            }
        }

        self.log_message(
            "system",
            "❌ User rejected tool execution permission prompt.",
        );
        false
    }

    pub fn complete_task(&mut self, summary: &str) {
        self.current_status = "Successfully Finished!".to_string();
        println!("\n🎉 [TASK COMPLETED SUCCESSFULLY]");
        println!("Summary: {}\n", summary);
    }

    pub fn fail_task(&mut self, error_reason: &str) {
        self.current_status = "Failed!".to_string();
        println!("\n🛑 [CRITICAL EXCEPTION RUNTIME FAILURE]");
        println!("Reason: {}\n", error_reason);
    }

    /// Clears the current line and prints a stylized, color-coded status message
    pub fn print_status(stage: UIStage, message: &str) {
        let mut stdout = io::stdout();

        // ANSI escape codes: \r returns cursor to start of line, \x1b[K clears the line
        print!("\r\x1b[K");

        match stage {
            UIStage::GraphSorting => {
                print!("\x1b[36m[1/4] ┠─⚙️ Graph Sorting:\x1b[0m {}", message);
            }
            UIStage::BackupLock => {
                print!("\x1b[35m[2/4] ┠─🔒 Transaction Lock:\x1b[0m {}", message);
            }
            UIStage::SurgicalPatching => {
                print!("\x1b[33m[3/4] ┠─✂️ Surgical Patching:\x1b[0m {}", message);
            }
            UIStage::Verification => {
                print!(
                    "\x1b[34m[4/4] ┠─🧪 Workspace Verification:\x1b[0m {}",
                    message
                );
            }
            UIStage::Success => {
                println!("\x1b[32m\n🚀 Refactor Complete: {}\x1b[0m", message);
                return;
            }
            UIStage::Failure => {
                println!("\x1b[31m\n❌ Refactor Rollback: {}\x1b[0m", message);
                return;
            }
        }

        let _ = stdout.flush();
    }

    /// Draws a quick inline ASCII progress bar for tracking large multi-file chains
    pub fn draw_progress(current: usize, total: usize, file_name: &str) {
        let mut stdout = io::stdout();
        let width = 20;
        let progress = if total > 0 {
            (current * width) / total
        } else {
            0
        };

        let mut bar = String::new();
        for i in 0..width {
            if i < progress {
                bar.push('█');
            } else {
                bar.push('░');
            }
        }

        print!(
            "\r\x1b[K\x1b[33m ┗━ [{}] {}/{} \x1b[0m modifying `{}`",
            bar, current, total, file_name
        );
        let _ = stdout.flush();
    }
}
