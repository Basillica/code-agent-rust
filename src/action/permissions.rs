use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;
use std::io::Write;

#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    Plan,
    DefaultMode,
    AcceptEdits,
    Auto,
    DontAsk,
    BypassPermissions,
    Bubble,
}

pub enum GateResult {
    Allowed,
    Blocked,
    RequiresUserApproval,
}

pub struct PermissionGate {
    mode: PermissionMode,
}

impl PermissionGate {
    pub fn new(mode: PermissionMode) -> Self {
        Self { mode }
    }

    pub async fn check_permission(&self, tool_name: &str, args: &serde_json::Value) -> bool {
        let is_write_operation =
            tool_name == "write_file" || tool_name == "surgical_edit" || tool_name == "edit_file";
        let is_shell_operation = matches!(tool_name, "bash" | "execute_command");

        if self.mode == PermissionMode::Plan {
            println!("❌ [Gate Block] Write/Shell actions are fully blocked under 'plan' mode.");
            return false;
        }

        if self.mode == PermissionMode::DontAsk || self.mode == PermissionMode::BypassPermissions {
            return true;
        }

        if self.mode == PermissionMode::AcceptEdits && is_write_operation {
            println!(
                "🔓 [Permission Gate]: Code modification auto-authorized under AcceptEdits mode."
            );
            return true;
        }

        if is_shell_operation || is_write_operation {
            return self.prompt_user_approval(tool_name, args);
        }

        // match self.mode {
        //     PermissionMode::DefaultMode | PermissionMode::AcceptEdits => {
        //         self.prompt_user_approval(tool_name, args).await
        //     }
        //     PermissionMode::Plan => {
        //         println!(
        //             "❌ [Gate Block] Write/Shell actions are fully blocked under 'plan' mode."
        //         );
        //         false
        //     }
        //     _ => {
        //         eprintln!("[Gate Critical] Unsupported or unknown fallback safety net triggered.");
        //         false
        //     }
        // }
        false
    }

    pub fn evaluate_tool_policy(&self, tool_name: &str) -> GateResult {
        let is_write_operation =
            tool_name == "write_file" || tool_name == "surgical_edit" || tool_name == "edit_file";
        let is_shell_operation =
            tool_name == "bash" || tool_name == "shell" || tool_name == "execute_command";

        // 1. Under 'Plan' mode, structural operations are hard blocked
        if self.mode == PermissionMode::Plan && (is_write_operation || is_shell_operation) {
            return GateResult::Blocked;
        }

        // 2. Open bypass modes auto-approve everything
        if self.mode == PermissionMode::DontAsk
            || self.mode == PermissionMode::BypassPermissions
            || self.mode == PermissionMode::Auto
        {
            return GateResult::Allowed;
        }

        // 3. AcceptEdits mode auto-approves file writes, but still gates terminal shells
        if self.mode == PermissionMode::AcceptEdits && is_write_operation {
            return GateResult::Allowed;
        }

        // 4. If it's a structural command under generic mode, flag it for UI prompting
        if is_shell_operation || is_write_operation {
            return GateResult::RequiresUserApproval;
        }

        // Read-only tools (like codebase_search, read_file, glob) bypass checks
        GateResult::Allowed
    }

    fn prompt_user_approval(&self, tool_name: &str, args: &Value) -> bool {
        println!("\n⚠️  [GATE INTERVENTION REQUIRED]");
        println!("👉 Tool Request: \"{}\"", tool_name);
        println!(
            "👉 Parameters:   {}",
            serde_json::to_string_pretty(args).unwrap_or_default()
        );
        print!("Allow execution? (y/N): ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let approved = input.trim().to_lowercase() == "y";
            if approved {
                println!("✅ Action authorized by user.");
                true
            } else {
                println!("❌ Action denied by user. Aborting tool call execution.");
                false
            }
        } else {
            false
        }
    }
}
