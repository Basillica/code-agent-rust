use serde::{Deserialize, Serialize};

use crate::orchestrator::graph::WorkspaceGraph;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSpan {
    pub file_name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerError {
    pub message: String,
    pub code: Option<String>,
    pub level: String, // "error" or "warning"
    pub spans: Vec<DiagnosticSpan>,
}

pub struct DiagnosticParser;

impl DiagnosticParser {
    /// Parses raw terminal output bytes, stripping away compiler noise to extract clean structural errors
    pub fn parse_cargo_json(raw_output: &str) -> Vec<CompilerError> {
        let mut errors = Vec::new();

        for line in raw_output.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with('{') {
                continue; // Skip standard non-JSON build text
            }

            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                // We only care about compiler-message blocks
                if val.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                    if let Some(msg) = val.get("message") {
                        let level = msg.get("level").and_then(|l| l.as_str()).unwrap_or("info");

                        // Filter for actionable build breaks
                        if level == "error" {
                            let message = msg
                                .get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("Unknown error")
                                .to_string();
                            let code = msg
                                .get("code")
                                .and_then(|c| c.get("code"))
                                .and_then(|c| c.as_str())
                                .map(|s| s.to_string());

                            let mut extracted_spans = Vec::new();
                            if let Some(spans) = msg.get("spans").and_then(|s| s.as_array()) {
                                for span in spans {
                                    extracted_spans.push(DiagnosticSpan {
                                        file_name: span
                                            .get("file_name")
                                            .and_then(|f| f.as_str())
                                            .unwrap_or("")
                                            .to_string(),
                                        line_start: span
                                            .get("line_start")
                                            .and_then(|l| l.as_u64())
                                            .unwrap_or(0)
                                            as usize,
                                        line_end: span
                                            .get("line_end")
                                            .and_then(|l| l.as_u64())
                                            .unwrap_or(0)
                                            as usize,
                                        column_start: span
                                            .get("column_start")
                                            .and_then(|c| c.as_u64())
                                            .unwrap_or(0)
                                            as usize,
                                        column_end: span
                                            .get("column_end")
                                            .and_then(|c| c.as_u64())
                                            .unwrap_or(0)
                                            as usize,
                                        is_primary: span
                                            .get("is_primary")
                                            .and_then(|p| p.as_bool())
                                            .unwrap_or(false),
                                    });
                                }
                            }

                            errors.push(CompilerError {
                                message,
                                code,
                                level: level.to_string(),
                                spans: extracted_spans,
                            });
                        }
                    }
                }
            }
        }

        errors
    }

    /// Formats structured errors into an incredibly clear, tight Markdown instruction template for the LLM
    pub fn format_errors_for_llm(errors: &[CompilerError]) -> String {
        if errors.is_empty() {
            return "No compilation errors encountered.".to_string();
        }

        let mut output = String::from(
            "### 🚨 Compilation Verification Failed\n\nPlease apply surgical patches to fix the following build regressions:\n\n",
        );
        for (i, err) in errors.iter().enumerate() {
            let code_str = err
                .code
                .as_ref()
                .map(|c| format!(" [{}]", c))
                .unwrap_or_default();
            output.push_str(&format!(
                "{}. **Error:** {}{}\n",
                i + 1,
                err.message,
                code_str
            ));

            for span in &err.spans {
                if span.is_primary {
                    output.push_str(&format!(
                        "   - **Target:** `{}` (Line {}, Col {})\n",
                        span.file_name, span.line_start, span.column_start
                    ));
                }
            }
        }
        output
    }

    pub fn parse_cargo_errors(compiler_output: &str) -> Vec<(String, String)> {
        let mut errors = Vec::new();
        let mut current_file = String::new();

        for line in compiler_output.lines() {
            if line.starts_with("error[") || line.starts_with("error:") {
                // Next lines usually contain file path bounds
            } else if line.trim().starts_with("--> ") {
                if let Some(path_part) = line.split("--> ").nth(1) {
                    if let Some(file_path) = path_part.split(':').next() {
                        current_file = file_path.trim().to_string();
                    }
                }
            } else if line.contains("cannot find") || line.contains("unresolved import") {
                if !current_file.is_empty() {
                    errors.push((current_file.clone(), line.trim().to_string()));
                }
            }
        }
        errors
    }
}

pub async fn verify_and_parse_workspace() -> (bool, String) {
    use std::process::Stdio;
    use tokio::process::Command;

    // Run cargo check utilizing JSON output streaming
    let output = Command::new("cargo")
        .args(&["check", "--message-format=json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match output {
        Ok(out) => {
            let stdout_str = String::from_utf8_lossy(&out.stdout);

            // 1. Structural extraction using your JSON parser
            let structured_errors = DiagnosticParser::parse_cargo_json(&stdout_str);

            if structured_errors.is_empty() && out.status.success() {
                (true, "Workspace builds cleanly! ✓".to_string())
            } else {
                // 2. Format the errors into a tight Markdown summary block for the LLM context
                let formatted_report = DiagnosticParser::format_errors_for_llm(&structured_errors);
                (false, formatted_report)
            }
        }
        Err(e) => (
            false,
            format!("Failed to invoke cargo check execution block: {}", e),
        ),
    }
}

pub fn handle_compilation_failure(graph: &mut WorkspaceGraph, compiler_stderr: &str) {
    let extracted_errors = DiagnosticParser::parse_cargo_errors(compiler_stderr);

    // Inject extracted issues right into the graph's break tracking array
    for (file_with_error, error_description) in extracted_errors {
        graph
            .broken_edges
            .push((file_with_error, error_description));
    }
}
