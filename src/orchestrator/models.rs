use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hunk {
    pub search_block: String,
    pub replace_block: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EditStatus {
    Pending,
    Applied,
    Failed(String),
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct RefactorTask {
//     pub id: String,
//     pub target_file: PathBuf,
//     pub explanation: String,
//     pub status: EditStatus,
//     pub hunks: Vec<Hunk>,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditTask {
    pub id: String,
    pub target_file: PathBuf,
    pub patch_instructions: String,
    pub dependencies: Vec<String>, // IDs of other tasks that must complete first
    pub status: EditStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorPlan {
    pub task_graph: HashMap<String, FileEditTask>,
    pub execution_order: Vec<String>,
}

pub struct FileSnapshot {
    pub original_path: PathBuf,
    pub content_backup: Vec<u8>,
}
