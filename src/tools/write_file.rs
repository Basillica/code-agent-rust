use crate::tools::Tool;
use crate::tools::utils::append_to_codebase_memory_log;
use async_trait::async_trait;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct WriteFileTool;

// #[async_trait]
// impl Tool for WriteFileTool {
//     fn name(&self) -> &'static str {
//         "write_file"
//     }
//     fn description(&self) -> &'static str {
//         "Create a new file or completely overwrite an existing one."
//     }
//     fn input_schema(&self) -> Value {
//         serde_json::json!({
//             "type": "object",
//             "properties": {
//                 "path": { "type": "string" },
//                 "content": { "type": "string" }
//             },
//             "required": ["path", "content"]
//         })
//     }
//     async fn execute(&self, args: &Value) -> Result<String, String> {
//         let path_str = args["path"].as_str().ok_or("Missing parameter 'path'")?;
//         let content = args["content"]
//             .as_str()
//             .ok_or("Missing parameter 'content'")?;

//         // fs::write(path_str, content).map_err(|e| format!("Error writing file: {}", e))?;
//         match fs::write(path_str, content) {
//             Ok(_) => {
//                 append_to_codebase_memory_log(path_str, "CREATE");
//                 Ok(format!("File successfully created at {}.", path_str))
//             }
//             Err(e) => Ok(format!("Error writing file: {}", e)),
//         }
//         // append_to_codebase_memory_log(path_str, "CREATE");
//         // Ok(format!("File successfully created at {}.", path_str))
//     }
// }

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }

    fn description(&self) -> &'static str {
        "Writes complete content to a specified file path, automatically resolving directories."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let path_str = args["path"].as_str().ok_or("Missing parameter 'path'")?;
        let content = args["content"]
            .as_str()
            .ok_or("Missing parameter 'content'")?;

        let path = Path::new(path_str);

        // Ensure parent structural directories exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        // Write raw generated text payload to target path
        fs::write(path_str, content).map_err(|e| e.to_string())?;

        // 🚀 TRIGGER AUTO-CLEANUP HOOK HERE
        println!(
            "✨ [Sanitizing & Auto-Formatting]: Cleaning up {}...",
            path_str
        );
        if let Err(sanitize_err) = sanitize_and_format_file(path_str).await {
            println!("⚠️ [Sanitizer Failed]: {}", sanitize_err);
            // We don't fail the whole tool execution if formatting fails,
            // but we alert the logs so the agent receives feedback.
        }

        append_to_codebase_memory_log(path_str, "CREATE");
        Ok(format!(
            "Successfully wrote and structurally formatted file at {}",
            path_str
        ))
    }
}

/// Sanitizes LLM string artifacts and formats the code using system utilities
pub async fn sanitize_and_format_file(path_str: &str) -> Result<(), String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err(format!(
            "Sanitizer Error: File {} does not exist.",
            path_str
        ));
    }

    // 1. Phase 1: Raw Text Cleanup (Fix LLM escaping anomalies)
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut cleaned_lines = Vec::new();

    for line in content.lines() {
        let mut trimmed = line.trim().to_string();

        // Strip accidental outer single/double quotes added by naive LLM generations
        if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            || (trimmed.starts_with('"') && trimmed.ends_with('"'))
        {
            if trimmed.len() > 1 {
                trimmed.remove(0);
                trimmed.pop();
            }
        }

        // Unescape literal \" back to normal double quotes
        let cleaned_line = trimmed.replace("\\\"", "\"");
        cleaned_lines.push(cleaned_line);
    }

    let cleaned_content = cleaned_lines.join("\n");
    fs::write(path, cleaned_content).map_err(|e| e.to_string())?;

    // 2. Phase 2: Structural Ecosystem Formatting
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    match extension {
        // === RUST ===
        "rs" => {
            let status = Command::new("rustfmt")
                .arg(path_str)
                .status()
                .map_err(|e| format!("Failed to execute rustfmt: {}", e))?;

            if !status.success() {
                println!(
                    "⚠️ [Sanitizer Warning]: rustfmt found structural issues in {}.",
                    path_str
                );
            }
        }

        // === TOML CONFIGURATIONS ===
        "toml" => {
            // Taplo is the modern enterprise standard for formatting and linting TOML files
            let status = Command::new("taplo")
                .args(["fmt", path_str])
                .status()
                .map_err(|e| format!("Failed to execute taplo: {}", e))?;

            if !status.success() {
                println!(
                    "⚠️ [Sanitizer Warning]: taplo failed to format {}.",
                    path_str
                );
            }
        }

        // === PYTHON ===
        "py" | "pyi" => {
            // 'black' or 'ruff format' are the definitive choices for Python syntax parsing
            let status = Command::new("black")
                .arg(path_str)
                .status()
                .map_err(|e| format!("Failed to execute python black formatter: {}", e))?;

            if !status.success() {
                println!(
                    "⚠️ [Sanitizer Warning]: black formatter rejected python syntax in {}.",
                    path_str
                );
            }
        }

        // === GO ===
        "go" => {
            // gofmt requires the "-w" flag to write changes directly back to the file
            let status = Command::new("gofmt")
                .args(["-w", path_str])
                .status()
                .map_err(|e| format!("Failed to execute gofmt: {}", e))?;

            if !status.success() {
                println!("⚠️ [Sanitizer Warning]: gofmt failed on {}.", path_str);
            }
        }

        // === WEB ECOSYSTEM (JavaScript, TypeScript, JSON, HTML, CSS, Markdown, YAML) ===
        "js" | "jsx" | "ts" | "tsx" | "json" | "html" | "css" | "scss" | "md" | "yaml" | "yml" => {
            // Prettier handles almost the entire modern frontend, layout, and document ecosystem.
            // Requires '--write' to modify the file in-place instead of dumping to stdout.
            let status = Command::new("npx")
                .args(["prettier", "--write", path_str])
                .status()
                .map_err(|e| format!("Failed to execute prettier via npx: {}", e))?;

            if !status.success() {
                println!(
                    "⚠️ [Sanitizer Warning]: Prettier layout formatting failed for {}.",
                    path_str
                );
            }
        }

        // === C / C++ / JAVA / C# ===
        "c" | "cpp" | "h" | "hpp" | "cc" | "cxx" | "java" | "cs" => {
            // clang-format cleanly standardizes curly-brace languages.
            // Requires "-i" to perform an in-place edit.
            let status = Command::new("clang-format")
                .args(["-i", path_str])
                .status()
                .map_err(|e| format!("Failed to execute clang-format: {}", e))?;

            if !status.success() {
                println!(
                    "⚠️ [Sanitizer Warning]: clang-format failed on {}.",
                    path_str
                );
            }
        }

        // === SHELL SCRIPTS ===
        "sh" | "bash" | "zsh" => {
            // shfmt standardizes shell script indentation and control flow structures.
            // Requires "-w" to write back to the file.
            let status = Command::new("shfmt")
                .args(["-w", path_str])
                .status()
                .map_err(|e| format!("Failed to execute shfmt: {}", e))?;

            if !status.success() {
                println!(
                    "⚠️ [Sanitizer Warning]: shfmt failed to clean shell script {}.",
                    path_str
                );
            }
        }

        // === RUBY ===
        "rb" | "gemspec" | "rakefile" => {
            let status = Command::new("rufo")
                .arg(path_str)
                .status()
                .map_err(|e| format!("Failed to execute rufo (Ruby Formatter): {}", e))?;

            if !status.success() {
                println!(
                    "⚠️ [Sanitizer Warning]: rufo formatting failed for {}.",
                    path_str
                );
            }
        }

        // === PHP ===
        "php" => {
            let status = Command::new("php-cs-fixer")
                .args(["fix", path_str])
                .status()
                .map_err(|e| format!("Failed to execute php-cs-fixer: {}", e))?;

            if !status.success() {
                println!(
                    "⚠️ [Sanitizer Warning]: php-cs-fixer failed on {}.",
                    path_str
                );
            }
        }

        // === SQL / DATABASE MIGRATIONS ===
        "sql" => {
            // sql-formatter standardizes uppercase keywords and complex join layouts
            let status = Command::new("npx")
                .args(["sql-formatter", "--replace", path_str])
                .status()
                .map_err(|e| format!("Failed to execute sql-formatter: {}", e))?;

            if !status.success() {
                println!(
                    "⚠️ [Sanitizer Warning]: SQL dialect optimization failed for {}.",
                    path_str
                );
            }
        }

        // === DEFAULT FALLBACK ===
        _ => {
            // If the agent creates a Dockerfile, .env, .gitignore, or raw text file,
            // we step over formatting safely without crashing or warning the user.
        }
    }

    Ok(())
}
