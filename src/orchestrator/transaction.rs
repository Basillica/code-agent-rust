use crate::orchestrator::models::FileSnapshot;
use std::fs;
use std::path::PathBuf;

pub struct WorkspaceTransaction {
    snapshots: Vec<FileSnapshot>,
    is_completed: bool,
}

impl WorkspaceTransaction {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            is_completed: false,
        }
    }

    /// Backs up a file state securely into vector buffers prior to execution mutations
    pub fn stage_file(&mut self, path: &PathBuf) -> Result<(), String> {
        if !path.exists() {
            // Check if we've already staged this new file to avoid duplicate entries
            if !self.snapshots.iter().any(|s| &s.original_path == path) {
                // Empty vector designates file did not exist before this transaction
                self.snapshots.push(FileSnapshot {
                    original_path: path.clone(),
                    content_backup: Vec::new(),
                });
            }
            return Ok(());
        }

        let content = fs::read(path)
            .map_err(|e| format!("Failed to read backup target {:?}: {}", path, e))?;

        // Standard idiomatic check: only push a snapshot if this path isn't already tracked
        if !self.snapshots.iter().any(|s| &s.original_path == path) {
            self.snapshots.push(FileSnapshot {
                original_path: path.clone(),
                content_backup: content,
            });
        }

        Ok(())
    }

    /// Commit locks the transaction, disabling rollback side-effects during cleanup
    pub fn commit(mut self) {
        self.is_completed = true;
        println!("[Transaction Engine] Refactoring chain successfully committed.");
    }

    /// Rollback restores files to their original baseline state if the transaction drops unexpectedly
    pub fn rollback(&mut self) -> Result<(), String> {
        if self.is_completed {
            return Ok(());
        }

        println!("[⚠️ Transaction Alert] Rolling back working directory changes cleanly...");
        for snapshot in &self.snapshots {
            if snapshot.content_backup.is_empty() {
                if snapshot.original_path.exists() {
                    let _ = fs::remove_file(&snapshot.original_path);
                }
            } else {
                fs::write(&snapshot.original_path, &snapshot.content_backup).map_err(|e| {
                    format!("Rollback failure on {:?}: {}", snapshot.original_path, e)
                })?;
            }
        }
        Ok(())
    }
}

// Extender utility trait to protect against array indexing lookups
trait SnapshotContains {
    fn raw_contains(&self, path: &PathBuf) -> Option<usize>;
}

impl SnapshotContains for Vec<FileSnapshot> {
    fn raw_contains(&self, path: &PathBuf) -> Option<usize> {
        self.iter().position(|s| s.original_path == *path)
    }
}

impl Drop for WorkspaceTransaction {
    fn drop(&mut self) {
        if !self.is_completed {
            let _ = self.rollback();
        }
    }
}
