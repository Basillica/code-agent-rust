use crate::orchestrator::models::Hunk;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct PatchHunk {
    pub search_block: String,
    pub replace_block: String,
}

pub struct PatchEngine;

impl PatchEngine {
    /// Parses an AI-generated patch block into discrete search-and-replace humks.
    /// It looks for:
    /// <<<<<<< SEARCH
    /// old code
    /// =======
    /// new code
    /// >>>>>>> REPLACE
    pub fn parse_patches(raw_diff: &str) -> Vec<PatchHunk> {
        let mut hunks = Vec::new();
        let mut lines = raw_diff.lines();

        let mut inside_search = false;
        let mut inside_replace = false;

        let mut search_lines = Vec::new();
        let mut replace_lines = Vec::new();

        while let Some(line) = lines.next() {
            let trimmed = line.trim();

            if trimmed.starts_with("<<<<<<< SEARCH") {
                inside_search = true;
                search_lines.clear();
                continue;
            }

            if trimmed.starts_with("=======") && inside_search {
                inside_search = false;
                inside_replace = true;
                replace_lines.clear();
                continue;
            }

            if trimmed.starts_with(">>>>>>> REPLACE") && inside_replace {
                inside_replace = false;
                hunks.push(PatchHunk {
                    search_block: search_lines.join("\n"),
                    replace_block: replace_lines.join("\n"),
                });
                continue;
            }

            if inside_search {
                search_lines.push(line.to_string());
            } else if inside_replace {
                replace_lines.push(line.to_string());
            }
        }

        hunks
    }

    /// Applies a sequence of parsed patch hunks to a physical file destination safely.
    pub fn apply_patches_to_file<P: AsRef<Path>>(
        file_path: P,
        hunks: &[PatchHunk],
    ) -> Result<(), String> {
        let path = file_path.as_ref();
        if !path.exists() {
            return Err(format!("Target patch path does not exist: {:?}", path));
        }

        let mut file_content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read target source file: {}", e))?;

        // Normalize line endings to avoid cross-platform platform carriage return discrepancies
        file_content = file_content.replace("\r\n", "\n");

        for (index, hunk) in hunks.iter().enumerate() {
            let target_search = hunk.search_block.replace("\r\n", "\n");
            let target_replace = hunk.replace_block.replace("\r\n", "\n");

            // Strategy 1: Exact Substring Matching Alignment
            if file_content.contains(&target_search) {
                file_content = file_content.replace(&target_search, &target_replace);
                continue;
            }

            // Strategy 2: Whitespace Insensitive Normalization Fallback
            // Trims leading spaces line-by-line to prevent indentation gaps from crashing the run
            let matched_index = Self::find_fuzzy_offset(&file_content, &target_search)
                .ok_or_else(|| format!(
                    "Hunk #{}: Code block matching segment could not be resolved inside your target file.\n\
                     === EXPECTED TARGET ===\n{}", index + 1, target_search
                ))?;

            file_content = Self::replace_fuzzy_slice(
                &file_content,
                matched_index,
                &target_search,
                &target_replace,
            );
        }

        fs::write(path, file_content).map_err(|e| {
            format!(
                "Failed storing updated patch matrix onto storage array: {}",
                e
            )
        })?;

        Ok(())
    }

    fn find_fuzzy_offset(source: &str, search: &str) -> Option<usize> {
        let clean_search = search
            .lines()
            .map(|l| l.trim())
            .collect::<Vec<&str>>()
            .join("\n");
        if clean_search.is_empty() {
            return None;
        }

        let source_lines: Vec<&str> = source.lines().collect();
        let search_lines: Vec<&str> = search.lines().map(|l| l.trim()).collect();

        if source_lines.len() < search_lines.len() {
            return None;
        }

        // Slide window down target codebase lines
        for i in 0..=(source_lines.len() - search_lines.len()) {
            let mut match_confirmed = true;
            for j in 0..search_lines.len() {
                if source_lines[i + j].trim() != search_lines[j] {
                    match_confirmed = false;
                    break;
                }
            }
            if match_confirmed {
                // Convert line position back into raw byte offset index positioning
                let mut byte_offset = 0;
                for line in source_lines.iter().take(i) {
                    byte_offset += line.len() + 1; // +1 for the newline
                }
                return Some(byte_offset);
            }
        }
        None
    }

    fn replace_fuzzy_slice(source: &str, offset: usize, search: &str, replace: &str) -> String {
        let prefix = &source[..offset];
        let remaining = &source[offset..];

        // Calculate where the search match ends in raw bytes
        let mut search_lines_count = search.lines().count();
        let mut source_lines = remaining.lines();
        let mut consumption_bytes = 0;

        while search_lines_count > 0 {
            if let Some(line) = source_lines.next() {
                consumption_bytes += line.len() + 1;
            }
            search_lines_count -= 1;
        }

        let suffix = if consumption_bytes <= remaining.len() {
            &remaining[consumption_bytes..]
        } else {
            ""
        };

        format!("{}{}{}", prefix, replace, suffix)
    }

    /// Applies a series of differential hunks to a file using sliding-window matching
    pub fn apply_surgical_patch(path: &PathBuf, hunks: &[Hunk]) -> Result<(), String> {
        let mut file_content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read target file for patching: {}", e))?;

        for hunk in hunks {
            // 1. Exact Match Try
            if file_content.contains(&hunk.search_block) {
                file_content = file_content.replace(&hunk.search_block, &hunk.replace_block);
                continue;
            }

            // 2. Fallback: Fuzzy Line-by-Line Sliding Window Match
            let file_lines: Vec<&str> = file_content.lines().collect();
            let search_lines: Vec<&str> = hunk.search_block.lines().collect();

            if search_lines.is_empty() {
                return Err("Empty search block provided in patching hunk.".to_string());
            }

            let mut match_index = None;
            let window_size = search_lines.len();

            if file_lines.len() >= window_size {
                for i in 0..=(file_lines.len() - window_size) {
                    let mut lines_match = true;
                    for j in 0..window_size {
                        // Trim whitespace to handle indentation updates gracefully
                        if file_lines[i + j].trim() != search_lines[j].trim() {
                            lines_match = false;
                            break;
                        }
                    }
                    if lines_match {
                        match_index = Some(i);
                        break;
                    }
                }
            }

            if let Some(start_idx) = match_index {
                // Reconstruct the file with the replaced section
                let mut new_lines = Vec::new();
                new_lines.extend_from_slice(&file_lines[0..start_idx]);
                new_lines.push(&hunk.replace_block);
                new_lines.extend_from_slice(&file_lines[start_idx + window_size..]);
                file_content = new_lines.join("\n");
            } else {
                return Err(format!(
                    "Patch Hunk Failed: Could not locate anchor block matching:\n```\n{}\n
```",
                    hunk.search_block
                ));
            }
        }

        fs::write(path, file_content)
            .map_err(|e| format!("Failed writing surgical patch updates to disk: {}", e))?;

        Ok(())
    }
}
