use crate::state::session::ContextSqueezer;
use crate::{state::session::SessionContext, tools::Tool};
use serde_json::Value;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

pub struct ExecuteCommandTool {
    pub session_ctx: Arc<Mutex<SessionContext>>,
}

impl ExecuteCommandTool {
    pub fn new(project_root: PathBuf) -> Self {
        let context = Arc::new(Mutex::new(SessionContext::new(project_root)));
        Self {
            session_ctx: context,
        }
    }
}

#[async_trait::async_trait]
impl Tool for ExecuteCommandTool {
    fn name(&self) -> &'static str {
        "execute_command"
    }

    fn description(&self) -> &'static str {
        "Executes a shell command statefully within the workspace terminal environment. Keeps directory paths, configuration parameters, and environment modifications alive across calls. Use this to install dependencies, run custom build frameworks, or seed localized testing pools."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The raw system command sequence string to dispatch downstream (e.g. 'cargo build')."
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: &serde_json::Value) -> Result<String, String> {
        let command_str = args["command"]
            .as_str()
            .ok_or_else(|| "Missing target payload verification argument: 'command'".to_string())?;

        // 1. Lock the persistent session context frame
        let mut lock = self.session_ctx.lock().await;

        // 2. Fetch the stateful active background shell
        let shell = lock.get_or_init_shell()?;

        println!("🐚 [Terminal Execution Executed]: {}", command_str);

        // 3. Dispatch execution to persistent shell sequence
        match shell.run_command(command_str).await {
            Ok((logs, exit_code)) => {
                let managed_logs = ContextSqueezer::squeeze_terminal_output(&logs, 40);
                if exit_code == 0 {
                    Ok(format!(
                        "--- COMMAND SUCCESSFUL ---\n[OUTPUT LOGS]:\n{}",
                        managed_logs
                    ))
                } else {
                    Err(format!(
                        "--- COMMAND FAILED WITH EXIT CODE: {} ---\n[ERROR OUTPUT LOGS]:\n{}",
                        exit_code, managed_logs
                    ))
                }
            }
            Err(runtime_err) => Err(format!("System shell driver fatal panic: {}", runtime_err)),
        }
    }
}
