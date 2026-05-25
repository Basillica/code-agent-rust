use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct CodebaseIndexer;

impl CodebaseIndexer {
    fn is_ignored(name: &str) -> bool {
        static IGNORED: &[&str] = &[
            "node_modules",
            ".git",
            "dist",
            "target",
            "venv",
            ".venv",
            "__pycache__",
            "staticfiles",
            ".pytest_cache",
        ];
        IGNORED.iter().any(|&dir| dir == name)
    }

    fn has_target_extension(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext,
                "ts" | "js" | "py" | "go" | "json" | "html" | "css" | "rs"
            )
        } else {
            false
        }
    }

    /// Recursively indexes repository paths and high-level signature anchors to brief the context frame.
    pub fn scan_workspace(&self, base_path: &Path) -> (Vec<String>, HashMap<String, Vec<String>>) {
        let mut file_tree = Vec::new();
        let mut structural_signatures = HashMap::new();

        fn walk(
            dir: &Path,
            base: &Path,
            files: &mut Vec<String>,
            sigs: &mut HashMap<String, Vec<String>>,
        ) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    if CodebaseIndexer::is_ignored(&file_name.as_ref()) {
                        continue;
                    }

                    if path.is_dir() {
                        walk(&path, base, files, sigs);
                    } else if path.is_file() {
                        if let Some(ext) = path.extension() {
                            let ext_str = ext.to_string_lossy();
                            if matches!(
                                ext_str.as_ref(),
                                "ts" | "js" | "py" | "go" | "json" | "rs" | "toml"
                            ) {
                                if let Ok(rel_path) = path.strip_prefix(base) {
                                    let rel_str = rel_path.to_string_lossy().to_string();
                                    files.push(rel_str.clone());

                                    // Extract lightweight core structures
                                    if matches!(ext_str.as_ref(), "rs" | "ts" | "py" | "go") {
                                        if let Ok(content) = fs::read_to_string(&path) {
                                            let mut file_sigs = Vec::new();
                                            for line in content.lines() {
                                                let trimmed = line.trim();
                                                if trimmed.starts_with("pub fn ")
                                                    || trimmed.starts_with("fn ")
                                                    || trimmed.starts_with("pub struct ")
                                                    || trimmed.starts_with("struct ")
                                                    || trimmed.starts_with("pub enum ")
                                                    || trimmed.starts_with("enum ")
                                                    || trimmed.starts_with("export function ")
                                                    || trimmed.starts_with("export class ")
                                                    || trimmed.starts_with("def ")
                                                {
                                                    file_sigs.push(trimmed.to_string());
                                                }
                                            }
                                            if !file_sigs.is_empty() {
                                                sigs.insert(rel_str, file_sigs);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        walk(
            base_path,
            base_path,
            &mut file_tree,
            &mut structural_signatures,
        );
        (file_tree, structural_signatures)
    }

    /// Regex anchor extractor mapping high-level code layout declarations to safeguard context boundaries.
    fn extract_signatures(content: &str, extension: &str) -> Vec<String> {
        let mut declarations = Vec::new();

        let pattern = match extension {
            "rs" => r"^\s*(pub\s+)?(fn|struct|enum|trait|mod|impl)\s+([a-zA-Z0-9_<>]+)",
            "py" => r"^\s*(def|class)\s+([a-zA-Z0-9_]+)",
            _ => r"^\s*(export\s+)?(function|interface|class|type)\s+([a-zA-Z0-9_]+)",
        };

        if let Ok(re) = Regex::new(pattern) {
            for line in content.lines() {
                if re.is_match(line) {
                    declarations.push(line.trim().to_string());
                }
            }
        }
        declarations
    }
}
