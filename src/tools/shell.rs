use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

pub struct ShellTool;
#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &'static str {
        "shell"
    }
    fn description(&self) -> &'static str {
        "Execute shell commands locally with a strict 30 second timeout boundary."
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

        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd.exe");
            c.args(["/C", command_str]);
            c
        } else {
            let mut c = Command::new("/bin/sh");
            c.args(["-c", command_str]);
            c
        };

        match timeout(Duration::from_secs(30), cmd.output()).await {
            Ok(Ok(out)) => {
                let mut result = String::new();
                if !out.stdout.is_empty() { result.push_str(&format!("[STDOUT]\n{}\n", String::from_utf8_lossy(&out.stdout))); }
                if !out.stderr.is_empty() { result.push_str(&format!("[STDERR]\n{}\n", String::from_utf8_lossy(&out.stderr))); }
                Ok(if result.is_empty() { "Command completed with empty terminal metrics.".to_string() } else { result.trim().to_string() })
            }
            Ok(Err(e)) => Err(format!("[EXECUTION ERROR]: {}", e)),
            Err(_) => Err("[SHELL TIMEOUT EXCEPTION]: Process execution exceeded the 30-second boundary ceiling.".to_string())
        }
    }
}
