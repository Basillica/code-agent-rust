use crate::tools::SafetyRule;
use chrono::Local;
use regex::Regex;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::OnceLock;
use tokio::process::Command;

pub fn banned_command_rules() -> &'static Vec<SafetyRule> {
    static RULES: OnceLock<Vec<SafetyRule>> = OnceLock::new();
    RULES.get_or_init(|| {
        vec![
            SafetyRule {
                pattern: r"(?i)\brm\s+-[rf]*\s*([/*\s]|$)",
                reason: "CRITICAL VIOLATION: Global or recursive file deletions via 'rm -rf' are strictly forbidden on this local host environment.",
            },
            SafetyRule {
                pattern: r"(?i)\b(cat|grep|open|edit_file|read_file)\s+.*\.env\b",
                reason: "SECURITY VIOLATION: Environment configuration files (.env) are sensitive and cannot be read or exposed to the execution stream.",
            },
            SafetyRule {
                pattern: r"(?i)\b(mkfs|dd|fdisk|chmod\s+777|chown)\b",
                reason: "SYSTEM VIOLATION: Low-level system formatting or destructive permission elevation overrides are blocked.",
            },
            SafetyRule {
                pattern: r"(?i)\b(git\s+push\s+--force|git\s+reset\s+--hard)\b",
                reason: "WORKSPACE VIOLATION: Force pushing or hard resets destroy uncommitted local work and code baseline history.",
            },
        ]
    })
}

pub fn check_bash_command_safety(command: &str) -> Result<(), String> {
    for rule in banned_command_rules() {
        let re = Regex::new(rule.pattern).unwrap();
        if re.is_match(command) {
            return Err(rule.reason.to_string());
        }
    }
    Ok(())
}

pub fn append_to_codebase_memory_log(file_path: &str, action_type: &str) {
    let memory_file_path = Path::new("MEMORY.md");
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("\n- [{}] **{}**: `{}`", timestamp, action_type, file_path);

    if !memory_file_path.exists() {
        let _ = fs::write(
            memory_file_path,
            "# Project Workspace Memory Log\nThis file tracks the autonomous generation modifications applied by the development engine.\n",
        );
    }

    if let Ok(mut file) = OpenOptions::new().append(true).open(memory_file_path) {
        let _ = writeln!(file, "{}", log_entry);
    }
}

/// Dynamically verifies workspace health by checking standard compiler loops.
pub async fn verify_workspace_health() -> (bool, Option<String>) {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Polyfill project intelligence: Auto-detect ecosystem verification targets
    let (cmd, args) = if current_dir.join("Cargo.toml").exists() {
        ("cargo", vec!["check", "--color", "never"])
    } else if current_dir.join("package.json").exists() {
        ("npx", vec!["tsc", "--noEmit"])
    } else {
        return (true, None); // Fallback for raw text directories
    };

    let output = Command::new(cmd)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match output {
        Ok(out) => {
            if out.status.success() {
                (true, None)
            } else {
                let stdout_log = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr_log = String::from_utf8_lossy(&out.stderr).to_string();
                let combined_log = format!("{}{}", stdout_log, stderr_log);
                (false, Some(combined_log.trim().to_string()))
            }
        }
        Err(err) => (
            false,
            Some(format!("Failed to invoke build checker '{}': {}", cmd, err)),
        ),
    }
}
