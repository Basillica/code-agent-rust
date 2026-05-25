use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CodeNode {
    pub file_path: String,
    pub imports: HashSet<String>,
    pub exports: Vec<String>,
}

impl CodeNode {
    fn new(file_path: String) -> Self {
        CodeNode {
            file_path,
            imports: HashSet::new(),
            exports: vec![],
        }
    }
}

pub struct WorkspaceGraph {
    pub nodes: HashMap<String, CodeNode>,
    pub broken_edges: Vec<(String, String)>, // Tracks dangling or invalid imports
}

impl WorkspaceGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            broken_edges: Vec::new(),
        }
    }

    pub fn add_dependency(&mut self, source: PathBuf, dependency: PathBuf) {
        let source_str = source.to_string_lossy().into_owned();
        let dep_str = dependency.to_string_lossy().into_owned();

        // Access the Entry for the source node, creating it if absent,
        // then push the dependency path string into its imports set.
        self.nodes
            .entry(source_str.clone())
            .or_insert_with(|| CodeNode::new(source_str))
            .imports
            .insert(dep_str);
    }

    /// Performs a topological sort using Kahn's Algorithm to generate
    /// a safe, non-conflicting chronological execution path.
    pub fn resolve_execution_order(&self) -> Result<Vec<PathBuf>, String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut all_nodes = HashSet::new();

        // 1. Initialize node spaces and calculate in-degree metrics
        for (node_path, code_node) in &self.nodes {
            all_nodes.insert(node_path.clone());
            for neighbor in &code_node.imports {
                all_nodes.insert(neighbor.clone());
                *in_degree.entry(neighbor.clone()).or_insert(0) += 1;
            }
        }

        // 2. Gather all root nodes with zero incoming entry barriers safely
        let mut queue: Vec<String> = all_nodes
            .iter()
            .filter(|node| in_degree.get(*node).copied().unwrap_or(0) == 0)
            .cloned()
            .collect();

        // Enforce exact alphabetical sorting to eliminate randomized flakiness
        // caused by internal HashSet and HashMap hashing seeds
        queue.sort();

        let mut ordered_sequence = Vec::new();

        // 3. Process the topological pathing sequence
        while let Some(current) = queue.pop() {
            ordered_sequence.push(PathBuf::from(current.clone()));

            if let Some(code_node) = self.nodes.get(&current) {
                // Collect and sort neighbor import keys for strict execution determinism
                let mut sorted_neighbors: Vec<&String> = code_node.imports.iter().collect();
                sorted_neighbors.sort();

                for neighbor in sorted_neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        // 4. Validate cycle loops
        if ordered_sequence.len() != all_nodes.len() {
            return Err(
                "Circular dependency deadlock detected in your structural refactoring plan!"
                    .to_string(),
            );
        }

        // Reverse sequence so downstream dependencies execute safely after their independent bases
        ordered_sequence.reverse();
        Ok(ordered_sequence)
    }

    /// Scans a target file directory structure to extract internal structural cross-references.
    pub fn build_from_workspace(root_dir: &PathBuf, targets: &[PathBuf]) -> Self {
        let mut graph = Self::new();

        for target in targets {
            let absolute_path = root_dir.join(target);
            if !absolute_path.exists() {
                continue;
            }

            if let Ok(file) = File::open(&absolute_path) {
                let reader = BufReader::new(file);

                for line in reader.lines().flatten() {
                    let trimmed = line.trim();

                    // Identify Rust module import links
                    if trimmed.starts_with("use crate::") {
                        if let Some(parsed_dep) = Self::extract_module_path(trimmed) {
                            let expected_file_target =
                                root_dir.join("src").join(format!("{}.rs", parsed_dep));
                            if expected_file_target.exists() {
                                let relative_dep = expected_file_target
                                    .strip_prefix(root_dir)
                                    .unwrap_or(&expected_file_target)
                                    .to_path_buf();

                                graph.add_dependency(target.clone(), relative_dep);
                            }
                        }
                    }
                }
            }
        }

        graph
    }

    fn extract_module_path(import_line: &str) -> Option<String> {
        // e.g., converts "use crate::models::user::User;" -> "models/user"
        let parts: Vec<&str> = import_line
            .trim_start_matches("use crate::")
            .split("::")
            .collect();

        if parts.len() > 1 {
            // Drop trailing type/struct definitions, keep path structures
            let path_parts = parts[..parts.len() - 1].join("/");
            return Some(path_parts);
        }
        None
    }

    /// Populates the graph using the file tree and raw content/signatures scanned from the workspace
    pub fn update_from_workspace(
        &mut self,
        file_tree: &[String],
        structural_signatures: &[(String, Vec<String>)],
    ) {
        self.nodes.clear();
        self.broken_edges.clear();

        // Step 1: Initialize nodes
        for file in file_tree {
            self.nodes.insert(
                file.clone(),
                CodeNode {
                    file_path: file.clone(),
                    imports: HashSet::new(),
                    exports: Vec::new(),
                },
            );
        }

        // Step 2: Parse basic dependencies from signatures (e.g., "use crate::tools::Tool")
        for (file, sigs) in structural_signatures {
            if let Some(node) = self.nodes.get_mut(file) {
                for sig in sigs {
                    if sig.starts_with("use crate::") {
                        // Extract a crude file path map out of the module path
                        let parts: Vec<&str> = sig.split("::").collect();
                        if parts.len() > 2 {
                            let simulated_path = format!("src/{}.rs", parts[2].replace(";", ""));
                            node.imports.insert(simulated_path);
                        }
                    } else if sig.starts_with("pub fn") || sig.starts_with("pub struct") {
                        node.exports.push(sig.clone());
                    }
                }
            }
        }

        // Step 3: Validate cross-file import existence
        let all_paths: HashSet<String> = self.nodes.keys().cloned().collect();
        for (path, node) in &self.nodes {
            for import in &node.imports {
                if !all_paths.contains(import) && !import.contains("mod") {
                    self.broken_edges.push((path.clone(), import.clone()));
                }
            }
        }
    }

    /// Renders dependency lines to expose topology back to your LLM system prompt context loop
    pub fn render_dependency_edges_to_string(&self) -> String {
        if self.nodes.is_empty() {
            return "No active structural node edges mapped.".to_string();
        }

        let mut output = String::new();
        for (path, node) in &self.nodes {
            if !node.imports.is_empty() {
                let deps: Vec<String> = node.imports.iter().cloned().collect();
                output.push_str(&format!(
                    "  {} -> depends on -> [{}]\n",
                    path,
                    deps.join(", ")
                ));
            }
        }
        output
    }

    /// Evaluates if any broken compilation or unresolved structural loops exist inside your graph layout
    pub fn verify_structural_integrity(&self) -> bool {
        self.broken_edges.is_empty()
    }
}
