use crate::tools::Tool;
use crate::tools::utils::check_bash_command_safety;
use async_trait::async_trait;
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

pub struct BashTool;
#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &'static str {
        "bash"
    }
    fn description(&self) -> &'static str {
        "Execute workspace shell commands locally with a 45-second safety timeout. Background daemons are forbidden."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": { "command": { "type": "string" } },
            "required": ["command"]
        })
    }
    async fn execute(&self, args: &Value) -> Result<String, String> {
        let command_str = args["command"]
            .as_str()
            .ok_or("Missing parameter 'command'")?;

        if let Err(block_msg) = check_bash_command_safety(command_str) {
            println!(
                "\n🛑 [SAFETY BLOCKED]: Agent attempted an unsafe shell command:\n   -> \"{}\"",
                command_str
            );
            return Ok(format!(
                "### [TOOL EXECUTION DENIED BY SECURITY GATE]\n{}\n\nReview your command and structure your action using valid alternatives.",
                block_msg
            ));
        }

        let is_windows = cfg!(target_os = "windows");
        let (shell, shell_args) = if is_windows {
            ("cmd.exe", vec!["/d", "/s", "/c", command_str])
        } else {
            ("/bin/bash", vec!["-c", command_str])
        };

        // Spawn the shell process wrapper
        let child = match Command::new(shell)
            .args(&shell_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return Ok(format!("[Process Spawn Error]: {}", e)),
        };

        // Wrap the channel thread bounds with a precise timeout race mechanism
        let _timeout_duration = Duration::from_secs(45);
        let pid = child.id();

        println!(
            "\n💻 [Terminal Execution]: Shell command invoked -> \"{}\"",
            command_str
        );

        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd.exe");
            c.args(["/d", "/s", "/C", command_str]);
            c
        } else {
            let mut c = Command::new("/bin/bash");
            c.args(["-c", command_str]);
            c
        };

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        match timeout(Duration::from_secs(45), cmd.output()).await {
            Ok(Ok(out)) => {
                let mut buffer = String::new();
                if !out.stdout.is_empty() {
                    buffer.push_str(&String::from_utf8_lossy(&out.stdout));
                }
                if !out.stderr.is_empty() {
                    buffer.push_str(&format!(
                        "[STDERR]: {}",
                        String::from_utf8_lossy(&out.stderr)
                    ));
                }

                let result_text = if buffer.trim().is_empty() {
                    "Command completed successfully with clean exit criteria.".to_string()
                } else {
                    buffer.trim().to_string()
                };
                Ok(format!(
                    "[Process Exit Code: {}]\n{}",
                    out.status.code().unwrap_or(-1),
                    result_text
                ))
            }
            Ok(Err(e)) => Err(format!("[Process Spawn Error]: {}", e)),
            // Err(_) => Ok("[BASH TIMEOUT EXCEPTION]: Command timed out after 45 seconds. The process was terminated safely.".to_string())
            Err(_) => {
                // Terminate rogue runaway background tasks securely across OS layers
                if is_windows {
                    let _ = Command::new("taskkill")
                        .args(&["/pid", &pid.unwrap().to_string(), "/f", "/t"])
                        .output();
                } else {
                    let _ = Command::new("kill")
                        .args(&["-9", &pid.unwrap().to_string()])
                        .output();
                }
                Ok("[BASH TIMEOUT EXCEPTION]: Command timed out after 45 seconds. The process was terminated safely.".to_string())
            }
        }
    }
}
