use crate::terminal::shell::StatefulShell;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub struct SessionContext {
    pub history: Vec<Message>,
    pub project_root: PathBuf,
    pub project_instructions: String,
    pub auto_memory: String,
    pub active_shell: Option<StatefulShell>,
}

impl SessionContext {
    pub fn new<P: AsRef<Path>>(project_root: P) -> Self {
        let mut ctx = Self {
            history: Vec::new(),
            project_root: project_root.as_ref().to_path_buf(),
            project_instructions: String::new(),
            auto_memory: String::new(),
            active_shell: None, // Initialize as idle
        };
        ctx.reload_workspace_context();
        ctx
    }

    /// Lazy-initializes or returns the active stateful terminal pipeline
    pub fn get_or_init_shell(&mut self) -> Result<&mut StatefulShell, String> {
        if self.active_shell.is_none() {
            println!("[Context Engine] Initializing stateful background terminal context...");
            self.active_shell = Some(StatefulShell::new()?);
        }
        Ok(self.active_shell.as_mut().unwrap())
    }

    pub fn reload_workspace_context(&mut self) {
        // 1. Look for AGENT.md instructions in project root
        let claude_md_path = self.project_root.join("AGENT.md");
        if claude_md_path.exists() {
            if let Ok(content) = fs::read_to_string(&claude_md_path) {
                self.project_instructions = content;
                println!("[Context Engine] successfully loaded AGENT.md project constraints.");
            }
        } else {
            self.project_instructions = String::from(
                "No active AGENT.md guidelines file discovered in project workspace root.\n\
                 Guideline Defaults: Prefer modern clean Rust implementations. Write comprehensive tests.",
            );
        }

        // 2. Look for persistent Auto-Memory (MEMORY.md)
        let memory_md_path = self.project_root.join("MEMORY.md");
        if memory_md_path.exists() {
            if let Ok(raw_memory) = fs::read_to_string(&memory_md_path) {
                // Protect context limits by absorbing the first 200 lines
                self.auto_memory = raw_memory
                    .lines()
                    .take(200)
                    .collect::<Vec<&str>>()
                    .join("\n");
            }
        } else {
            self.auto_memory = String::from("No accumulated persistent memories saved yet.");
        }
    }

    pub fn append_message(&mut self, role: &str, content: &str) {
        self.history.push(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
    }

    /// Appends learning notes securely to the persistent file-system audit layer.
    pub fn save_persistent_memory(&mut self, new_learning: &str) {
        let memory_md_path = self.project_root.join("MEMORY.md");
        let timestamp = chrono::Local::now().format("%Y-%m-%d").to_string();
        let log_entry = format!("\n- [{}] Persistent Learning: {}", timestamp, new_learning);

        if !memory_md_path.exists() {
            let _ = fs::write(
                &memory_md_path,
                "# Project Workspace Memory Log\nThis file tracks the autonomous generation modifications applied by the development engine.\n",
            );
        }

        if let Ok(mut file) = fs::OpenOptions::new().append(true).open(&memory_md_path) {
            use std::io::Write;
            let _ = writeln!(file, "{}", log_entry);
        }

        self.reload_workspace_context();
    }

    pub fn clear_memory(&mut self, result: &str) {
        self.save_persistent_memory(result);
        self.history = vec![];
        self.project_instructions = String::new();
    }
}

pub struct ContextSqueezer;

impl ContextSqueezer {
    /// Compresses tool observations (like massive terminal outputs) by keeping
    /// the head, tail, and extracting explicit error blocks.
    pub fn squeeze_terminal_output(output: &str, max_lines: usize) -> String {
        let lines: Vec<&str> = output.lines().collect();

        if lines.len() <= max_lines {
            return output.to_string();
        }

        // Search for high-value error signals typical in compilation/testing
        let mut high_value_indices = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            let lower = line.to_lowercase();
            if lower.contains("error:")
                || lower.contains("failed")
                || lower.contains("panic")
                || lower.contains("compiler output:")
            {
                high_value_indices.push(idx);
            }
        }

        if high_value_indices.is_empty() {
            // No explicit errors found; perform standard head/tail truncation
            let half = max_lines / 2;
            let head = lines[..half].join("\n");
            let tail = lines[lines.len() - half..].join("\n");
            return format!(
                "{}\n\n[... TRUNCATED {} LINES OF OMITTED LOGS ...] \n\n{}",
                head,
                lines.len() - max_lines,
                tail
            );
        }

        // If errors are present, surgically isolate rows surrounding those errors
        let mut curated_lines = Vec::new();
        curated_lines.push(format!(
            "--- SURGICAL DIAGNOSTIC SNAPSHOT (Original size: {} lines) ---",
            lines.len()
        ));
        curated_lines.push(lines[..std::cmp::min(5, lines.len())].join("\n")); // Keep the first few context lines
        curated_lines.push("\n[... ISOLATING RUNTIME FAILURES ...]".to_string());

        let mut last_added_idx = 0;
        for err_idx in high_value_indices {
            // Capture a small window of 2 lines before and 3 lines after the failure indicator
            let start = err_idx.saturating_sub(2);
            let end = std::cmp::min(lines.len(), err_idx + 4);

            if start > last_added_idx && last_added_idx != 0 {
                curated_lines.push("...".to_string());
            }

            for i in std::cmp::max(start, last_added_idx)..end {
                curated_lines.push(lines[i].to_string());
            }
            last_added_idx = end;

            if curated_lines.len() > max_lines {
                curated_lines.push(
                    "\n⚠️ Additional errors omitted to respect context boundaries.".to_string(),
                );
                break;
            }
        }

        curated_lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_squeezing() {
        let massive_output = "unrelated line 1\nunrelated line 2\nerror: expected type String, found u32\nline item 4\nline item 5";
        let compressed = ContextSqueezer::squeeze_terminal_output(massive_output, 3);
        assert!(compressed.contains("error: expected type String"));
    }
}
