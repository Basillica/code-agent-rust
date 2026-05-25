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
}

impl SessionContext {
    pub fn new<P: AsRef<Path>>(project_root: P) -> Self {
        let mut ctx = Self {
            history: Vec::new(),
            project_root: project_root.as_ref().to_path_buf(),
            project_instructions: String::new(),
            auto_memory: String::new(),
        };
        ctx.reload_workspace_context();
        ctx
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
