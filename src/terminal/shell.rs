// src/terminal/shell.rs

use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout};

pub struct StatefulShell {
    _child: Child,
    stdin: ChildStdin,
    stdout_lines: Lines<BufReader<ChildStdout>>,
    stderr_lines: Lines<BufReader<ChildStderr>>,
    sentinel_token: String,
}

impl StatefulShell {
    /// Spawns a persistent background shell session
    pub fn new() -> Result<Self, String> {
        let shell_binary = if cfg!(windows) { "cmd.exe" } else { "bash" };
        let mut cmd = tokio::process::Command::new(shell_binary);

        // Pipe all three interfaces explicitly
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to initialize OS background shell process: {}", e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or("Failed to lock terminal stdin pipeline")?;
        let stdout = child
            .stdout
            .take()
            .ok_or("Failed to lock terminal stdout pipeline")?;
        let stderr = child
            .stderr
            .take()
            .ok_or("Failed to lock terminal stderr pipeline")?;

        // Transform streaming handles into persistent stateful line readers
        let stdout_lines = BufReader::new(stdout).lines();
        let stderr_lines = BufReader::new(stderr).lines();

        Ok(Self {
            _child: child,
            stdin,
            stdout_lines,
            stderr_lines,
            sentinel_token: "__AGENT_TERMINAL_BOUNDARY_MARKER__".to_string(),
        })
    }

    /// Executes a system command statefully, pulling stdout and stderr concurrently
    pub async fn run_command(&mut self, raw_command: &str) -> Result<(String, i32), String> {
        if raw_command.contains("ssh")
            || raw_command.contains("nano")
            || raw_command.contains("vim")
        {
            return Err(
                "Interactive terminal interfaces are blocked inside headless agent sectors."
                    .to_string(),
            );
        }

        // Format terminal commands with localized return-code trackers
        let formatted_payload = if cfg!(windows) {
            format!(
                "{}\necho.\necho {}:%errorlevel%\n",
                raw_command.trim(),
                self.sentinel_token
            )
        } else {
            format!(
                "{}; echo -e \"\\n{}:$?\"\n",
                raw_command.trim(),
                self.sentinel_token
            )
        };

        self.stdin
            .write_all(formatted_payload.as_bytes())
            .await
            .map_err(|e| format!("Terminal stdin pipeline write dropped: {}", e))?;

        self.stdin.flush().await.map_err(|e| e.to_string())?;

        let mut accumulated_logs = Vec::new();
        let mut exit_code = 0;

        // Concurrently read both stdout and stderr streams until the sentinel is found
        loop {
            tokio::select! {
                // Read next available line from standard output channel
                stdout_res = self.stdout_lines.next_line() => {
                    match stdout_res {
                        Ok(Some(line)) => {
                            if line.contains(&self.sentinel_token) {
                                if let Some(status_str) = line.split(':').last() {
                                    exit_code = status_str.trim().parse::<i32>().unwrap_or(0);
                                }
                                break;
                            }
                            accumulated_logs.push(format!("{}\n", line));
                        }
                        Ok(None) => return Err("Background terminal stdout closed unexpectedly.".to_string()),
                        Err(e) => return Err(format!("Stdout stream error: {}", e)),
                    }
                }
                // Read next available line from standard error channel
                stderr_res = self.stderr_lines.next_line() => {
                    match stderr_res {
                        Ok(Some(line)) => {
                            accumulated_logs.push(format!("{}\n", line));
                        }
                        Ok(None) => {}, // Stderr stream can idle safely
                        Err(e) => return Err(format!("Stderr stream error: {}", e)),
                    }
                }
            }
        }

        Ok((accumulated_logs.join(""), exit_code))
    }
}
