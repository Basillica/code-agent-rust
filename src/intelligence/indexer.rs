use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SymbolType {
    Struct,
    Function,
    Interface,
    Trait,
    Module,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeSymbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub file_path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct CodebaseIndexer {
    /// Maps a global symbol name (e.g., "AgentPromptController") to its file information
    pub symbol_table: HashMap<String, Vec<CodeSymbol>>,
}

impl CodebaseIndexer {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
        }
    }

    /// Recursively scans the active workspace directory to index code structures
    pub fn index_workspace<P: AsRef<Path>>(&mut self, workspace_root: P) -> Result<(), String> {
        self.symbol_table.clear();

        for entry in walkdir::WalkDir::new(workspace_root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != "target" && name != "node_modules" && name != ".git"
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    match ext {
                        "rs" => self.parse_rust_symbols(path)?,
                        "go" => self.parse_go_symbols(path)?,
                        "ts" | "js" => self.parse_typescript_symbols(path)?,
                        "py" => self.parse_python_symbols(path)?,
                        _ => {} // Skip unknown extensions
                    }
                }
            }
        }
        Ok(())
    }

    /// Simplified internal block parser to extract language tokens
    fn parse_rust_symbols(&mut self, path: &Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file for indexing: {}", e))?;

        for (idx, line) in content.lines().enumerate() {
            let line_trimmed = line.trim();

            // Look for struct declarations
            if line_trimmed.starts_with("pub struct ") || line_trimmed.starts_with("struct ") {
                let parts: Vec<&str> = line_trimmed.split_whitespace().collect();
                if let Some(name_index) = parts.iter().position(|&r| r == "struct").map(|i| i + 1) {
                    if name_index < parts.len() {
                        let name = parts[name_index]
                            .split('{')
                            .next()
                            .unwrap_or("")
                            .to_string();
                        self.insert_symbol(name, SymbolType::Struct, path, idx + 1);
                    }
                }
            }
            // Look for function declarations
            else if line_trimmed.starts_with("pub fn ") || line_trimmed.starts_with("fn ") {
                let parts: Vec<&str> = line_trimmed.split_whitespace().collect();
                if let Some(name_index) = parts.iter().position(|&r| r == "fn").map(|i| i + 1) {
                    if name_index < parts.len() {
                        let name = parts[name_index]
                            .split('(')
                            .next()
                            .unwrap_or("")
                            .to_string();
                        self.insert_symbol(name, SymbolType::Function, path, idx + 1);
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_go_symbols(&mut self, path: &Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("type ") && trimmed.contains("struct") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() > 1 {
                    self.insert_symbol(parts[1].to_string(), SymbolType::Struct, path, idx + 1);
                }
            } else if trimmed.starts_with("func ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() > 1 {
                    let name = parts[1].split('(').next().unwrap_or("").to_string();
                    self.insert_symbol(name, SymbolType::Function, path, idx + 1);
                }
            }
        }
        Ok(())
    }

    fn parse_typescript_symbols(&mut self, path: &Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("interface ") || trimmed.contains("type ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(pos) = parts.iter().position(|&r| r == "interface" || r == "type") {
                    if pos + 1 < parts.len() {
                        self.insert_symbol(
                            parts[pos + 1].to_string(),
                            SymbolType::Interface,
                            path,
                            idx + 1,
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_python_symbols(&mut self, path: &Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("class ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() > 1 {
                    let name = parts[1]
                        .split(':')
                        .next()
                        .unwrap_or("")
                        .split('(')
                        .next()
                        .unwrap_or("")
                        .to_string();
                    self.insert_symbol(name, SymbolType::Struct, path, idx + 1);
                }
            } else if trimmed.starts_with("def ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() > 1 {
                    let name = parts[1].split('(').next().unwrap_or("").to_string();
                    self.insert_symbol(name, SymbolType::Function, path, idx + 1);
                }
            }
        }
        Ok(())
    }

    fn insert_symbol(&mut self, name: String, sym_type: SymbolType, path: &Path, line: usize) {
        if name.is_empty() {
            return;
        }
        let symbol = CodeSymbol {
            name: name.clone(),
            symbol_type: sym_type,
            file_path: path.to_path_buf(),
            start_line: line,
            end_line: line,
        };
        self.symbol_table
            .entry(name)
            .or_insert_with(Vec::new)
            .push(symbol);
    }

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

    /// Recursively indexes repository paths and high-level signature anchors to brief the context frame.
    pub fn scan_workspace(&self, base_path: &Path) -> (Vec<String>, HashMap<String, Vec<String>>) {
        let mut file_tree: Vec<String> = Vec::new();
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
}
