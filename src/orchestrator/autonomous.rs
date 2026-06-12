use crate::action::permissions::PermissionMode;
use crate::core::compaction::{apply_generative_auto_compact, apply_snip_shaper};
use crate::intelligence::indexer::CodebaseIndexer;
use crate::orchestrator::budget::TokenBudgetGuardrail;
use crate::orchestrator::config::ModelCostConfig;
use crate::orchestrator::diagnostic::DiagnosticParser;
use crate::orchestrator::graph::WorkspaceGraph;
use crate::orchestrator::ui::TerminalUI;
use crate::state::session::{ContextSqueezer, SessionContext};
use crate::tools::registry::ToolRegistry;
use crate::verification::gate::VerificationGate;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

const OLLAMA_HOST: &str = "http://127.0.0.1:11434/api/chat";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentToolCall {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentResponse {
    pub thought: String,
    pub tool_call: Option<AgentToolCall>,
    pub task_completed: bool,
    pub final_summary: Option<String>,
}

pub struct AutonomousOrchestrator {
    session_ctx: Arc<Mutex<SessionContext>>,
    registry: Arc<ToolRegistry>,
    model_name: String,
    max_steps: usize,
}

impl AutonomousOrchestrator {
    pub fn new(
        session_ctx: Arc<Mutex<SessionContext>>,
        registry: Arc<ToolRegistry>,
        model_name: String,
    ) -> Self {
        Self {
            session_ctx,
            registry,
            model_name,
            max_steps: 50, // Reconciled match with your max_turns safety cap
        }
    }

    /// Primary entry point running the dynamic structural ReAct tool loop to conclusion
    pub async fn execute_goal(
        &self,
        user_prompt: &str,
        permission_mode: PermissionMode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Core setup and initial user message injection
        let mut session_initial = self.session_ctx.lock().await;
        session_initial.append_message("user", user_prompt);
        session_initial.reload_workspace_context();
        let project_root = session_initial.project_root.clone();
        drop(session_initial); // Release lock during active processing operations

        let http_client = Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(3600))
            .build()?;

        let mut graph = WorkspaceGraph::new();
        let mut ui = TerminalUI::new();
        ui.start_task(user_prompt);

        let custom_config = ModelCostConfig::builder()
            .name(&self.model_name)
            .input_cost_per_million(3.00)
            .output_cost_per_million(15.00)
            .max_context_tokens(200_000)
            .build()
            .expect("Failed to construct model configuration topology");

        // Initialize our global token guardrail monitor
        let mut budget = TokenBudgetGuardrail::new(
            custom_config,
            2.00,    // Spend cap boundary ceiling safety
            150_000, // Processing volume threshold limit
        );

        let mut step = 0;

        while step < self.max_steps {
            // Check budget constraints prior to making remote calls
            if let Err(breach_msg) = budget.check_budget_safety() {
                println!("{}", breach_msg);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    breach_msg,
                )));
            }

            step += 1;

            // Update UI telemetry data tracking
            ui.update_status(&format!(
                "Turn {}/{} - Indexing codebase & updating Workspace Graph...",
                step, self.max_steps
            ));
            {
                let lock = self.session_ctx.lock().await;
                ui.render(&*lock, &graph);
            }

            // 2. Pre-Model Shaper Layer: Compact session history space dynamically
            {
                let mut lock = self.session_ctx.lock().await;
                lock.history = apply_generative_auto_compact(&lock.history, 15).await?;
            }

            // 3. Scan workspace structure dynamically to construct structural code topologies
            let indexer = CodebaseIndexer::new();
            let (file_tree, signature_maps) = indexer.scan_workspace(&project_root);
            let structural_signatures: Vec<(String, Vec<String>)> =
                signature_maps.into_iter().collect();
            graph.update_from_workspace(&file_tree, &structural_signatures);

            let index_text_block = format!(
                "=== CURRENT WORKSPACE STRUCTURAL INDEX ===\n\
                 Discovered Code Files:\n{}\n\n\
                 Structural Declarations mapped across active modules:\n{}\n\
                 Workspace Cross-File Structural Dependencies:\n{}\n\
                 ==========================================",
                file_tree
                    .iter()
                    .map(|f| format!("- {}", f))
                    .collect::<Vec<_>>()
                    .join("\n"),
                structural_signatures
                    .iter()
                    .map(|(file, sigs)| format!("[{}]:\n  {}", file, sigs.join("\n  ")))
                    .collect::<Vec<_>>()
                    .join("\n"),
                graph.render_dependency_edges_to_string()
            );

            // 4. Construct unified System Matrix Context string
            let mut available_tools_text = String::new();
            for (name, tool) in &self.registry.tools {
                available_tools_text.push_str(&format!(
                    "- {}: {}. Schema: {}\n",
                    name,
                    tool.description(),
                    serde_json::to_string(&tool.input_schema()).unwrap_or_default()
                ));
            }

            let lock = self.session_ctx.lock().await;
            let current_project_instructions = lock.project_instructions.clone();
            let current_auto_memory = lock.auto_memory.clone();
            let compacted_history = apply_snip_shaper(&lock.history, 15, 4);
            drop(lock);

            let system_prompt = format!(
                "You are an autonomous engineering agent executing engineering tasks in a local workspace.\n\
                 Current Workspace Status:\n{}\n\n\
                 Current Style Guidelines (from AGENT.md):\n{}\n\n\
                 Accumulated Engine Memory: {}\n\n\
                 Available Tools in your Runtime Environment:\n{}\n\n\
                 CRITICAL EXECUTION POLICIES:\n\
                 1. Use dedicated tools ('glob', 'grep', 'read_file', 'edit_file', etc.) to find and edit code files.\n\
                 2. Use the 'bash' tool ONLY to run commands like database migrations, installing packages, or testing code.\n\
                 3. Always return a JSON object matching the specified schema in every response.\n\n\
                \n\
                \n\
                 EXPECTED JSON RESPONSE SCHEMA:\n\
                 {{\n  \"thought\": \"Your detailed step planning logic here\",\n  \"tool_call\": {{\n    \"name\": \"tool_name_here_or_null\",\n    \"arguments\": {{}}\n  }},\n  \"task_completed\": false,\n  \"final_summary\": null\n}}",
                index_text_block,
                current_project_instructions,
                current_auto_memory,
                available_tools_text
            );

            // 5. Query Ollama Endpoint via Chat Completion Payload Matrix
            ui.update_status(&format!(
                "Turn {} - Awaiting generation token arrays from Ollama...",
                step
            ));

            let mut messages_payload = vec![json!({ "role": "system", "content": system_prompt })];
            for msg in compacted_history {
                messages_payload.push(json!({ "role": msg.role, "content": msg.content }));
            }

            let res = http_client
                .post(OLLAMA_HOST)
                .json(&json!({
                    "model": self.model_name,
                    "messages": messages_payload,
                    "stream": false,
                    "format": {
                        "type": "object",
                        "properties": {
                            "thought": {
                                "type": "string",
                                "description": "Step-by-step reasoning details explaining your approach"
                            },
                            "tool_call": {
                                "type": ["object", "null"],
                                "properties": {
                                    "name": { "type": "string" },
                                    "arguments": { "type": "object" }
                                },
                                "required": ["name", "arguments"]
                            },
                            "task_completed": { "type": "boolean" },
                            "final_summary": { "type": ["string", "null"] }
                        },
                        "required": ["thought", "tool_call", "task_completed", "final_summary"]
                    },
                    "options": { "temperature": 0.1 }
                }))
                .send()
                .await?;

            // Track standard simulated metric values across fallback vectors
            let input_tokens = 4500; // res.usage.input_tokens;
            let output_tokens = 200; // res.usage.output_tokens;
            budget.record_usage(input_tokens, output_tokens);
            budget.print_telemetry_report();

            println!("---------------------- {:?}", res);

            if !res.status().is_success() {
                ui.fail_task(&format!(
                    "Ollama returned HTTP Error state: {}",
                    res.status()
                ));
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "Ollama connection lost",
                )));
            }

            let body_val: Value = res.json().await?;
            let raw_content = match body_val
                .pointer("/message/content")
                .and_then(|c| c.as_str())
            {
                Some(t) => t,
                None => {
                    println!("❌ Received empty text stream from model endpoint.");
                    break;
                }
            };

            // 6. Extract and validate structural JSON response payload boundaries
            let cleaned_json = Self::extract_clean_json(raw_content);
            let json_value: Value = match serde_json::from_str(&cleaned_json) {
                Ok(v) => v,
                Err(e) => {
                    println!("error clearning json. error: {:?}", e);
                    let mut lock = self.session_ctx.lock().await;
                    lock.append_message("assistant", raw_content);
                    lock.append_message("user", "CRITICAL PARSING ERROR: Response contained invalid JSON syntax characters.");
                    continue;
                }
            };

            // Process native adapter mapping vectors
            println!(",,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,, {:?}", json_value);
            let agent_decision: AgentResponse = if json_value.get("tool_calls").is_some() {
                // FORMAT A: Standard OpenAI/Ollama tool_calls pattern
                if let Some(first_call) =
                    json_value["tool_calls"].as_array().and_then(|a| a.first())
                {
                    AgentResponse {
                        thought: json_value["thought"]
                            .as_str()
                            .unwrap_or("Routing call via native schema adapter.")
                            .to_string(),
                        tool_call: Some(AgentToolCall {
                            name: first_call["function"].as_str().unwrap_or("").to_string(),
                            arguments: first_call["args"].clone(),
                        }),
                        task_completed: false,
                        final_summary: None,
                    }
                } else {
                    AgentResponse {
                        thought: "Empty tool configuration matrix array intercepted.".to_string(),
                        tool_call: None,
                        task_completed: false,
                        final_summary: None,
                    }
                }
            } else if json_value.get("tool_name").is_some() {
                // FORMAT B: Direct Ollama flat schema pattern (Fixes Turn 9-15 structural breakdown)
                AgentResponse {
                    thought: json_value["thought"].as_str().unwrap_or("").to_string(),
                    tool_call: Some(AgentToolCall {
                        name: json_value["tool_name"].as_str().unwrap_or("").to_string(),
                        arguments: json_value
                            .get("parameters")
                            .or_else(|| json_value.get("tool_input"))
                            .cloned()
                            .unwrap_or(serde_json::Value::Null),
                    }),
                    task_completed: false,
                    final_summary: None,
                }
            } else {
                // FORMAT C: Standard message response tracking fallback
                match serde_json::from_value::<AgentResponse>(json_value) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        println!(
                            "⚠️ [Schema Mismatch]: Model structural format missed targets: {:?}",
                            e
                        );
                        let mut lock = self.session_ctx.lock().await;
                        lock.append_message("assistant", &cleaned_json);
                        lock.append_message("user", "CRITICAL SCHEMA ERROR: Provided keys mismatch strict expected schema targets.");
                        continue;
                    }
                }
            };

            ui.log_message("assistant", &format!("Thought: {}", agent_decision.thought));
            {
                let mut lock = self.session_ctx.lock().await;
                lock.append_message("assistant", &cleaned_json);
            }

            // 7. Handle Termination Paths (Quality Gate Verification Rules)
            if agent_decision.task_completed {
                ui.update_status("Task verification pass triggered. Executing environment validation check hooks...");
                let report = VerificationGate::execute_workspace_validation(&project_root);
                let graph_integrity_passing = graph.verify_structural_integrity();

                if report.is_passing && graph_integrity_passing {
                    let summary_content = agent_decision.final_summary.unwrap_or_else(|| {
                        String::from("Task completed successfully. Structural compilation integrity fully verified.")
                    });

                    let mut lock = self.session_ctx.lock().await;
                    lock.append_message(
                        "system",
                        &format!("Execution Finalized Summary:\n{}", summary_content),
                    );
                    lock.clear_memory(&summary_content);

                    ui.complete_task(&summary_content);
                    return Ok(());
                } else {
                    let correction_payload = format!(
                        "CRITICAL VERIFICATION FAILURE: Your alterations cannot be finalized due to errors:\n\n=== LINT SUITE ===\n{}",
                        report.lint_output
                    );
                    let mut lock = self.session_ctx.lock().await;
                    lock.append_message("user", &correction_payload);
                    continue;
                }
            }

            // 8. Process Requested Tool Execution Blocks
            if let Some(tool_call) = agent_decision.tool_call {
                let normalized_tool_name = tool_call.name.trim().to_lowercase();
                let conversational_aliases = [
                    "none", "null", "system", "print", "respond", "speak", "message",
                ];

                // if normalized_tool_name == "null"
                //     || normalized_tool_name == "none"
                //     || normalized_tool_name.is_empty()
                // {
                if normalized_tool_name.is_empty()
                    || conversational_aliases.contains(&normalized_tool_name.as_str())
                {
                    println!(
                        "\n🛑 [Orchestrator] Intercepted conversational early exit execution sequence."
                    );
                    let conversation_content = if let Some(msg) =
                        tool_call.arguments.get("message").and_then(|m| m.as_str())
                    {
                        msg.to_string()
                    } else if let Some(txt) =
                        tool_call.arguments.get("text").and_then(|t| t.as_str())
                    {
                        txt.to_string()
                    } else if !agent_decision.thought.is_empty() {
                        agent_decision.thought.clone()
                    } else if let Some(summary) = agent_decision.final_summary.clone() {
                        summary
                    } else {
                        // Fallback fallback if the object is totally empty
                        format!(
                            "Task processing finished via direct response matrix: {}",
                            tool_call.arguments
                        )
                    };

                    // Append the assistant's final response to the memory
                    let mut lock = self.session_ctx.lock().await;
                    // lock.append_message(
                    //     "assistant",
                    //     &format!("Final Response: {}", conversation_content),
                    // );
                    lock.append_message("system", &conversation_content);

                    // Render the text answer clearly to your UI layer
                    ui.complete_task(&conversation_content);
                    return Ok(()); // Gracefully finish up the execution step
                }

                let tool = match self.registry.tools.get(&tool_call.name) {
                    Some(t) => t,
                    None => {
                        // Extract all currently registered tool keys directly from the live registry state
                        let registered_keys: Vec<String> =
                            self.registry.tools.keys().cloned().collect();

                        println!(
                            "🛑 [Tool Interceptor] Model hallucinated unregistered tool: '{}'",
                            tool_call.name
                        );

                        let correction_payload = format!(
                            "SYSTEM ERROR: The tool '{}' is missing from the registry context. \
                            You are strictly prohibited from calling virtual or unmapped functions.\n\
                            VALID REGISTERED SYSTEM TOOLS MATRIX: {:?}",
                            tool_call.name, registered_keys
                        );

                        let mut lock = self.session_ctx.lock().await;
                        // Log the sequence into historical conversation context so it doesn't loop blindly
                        // lock.append_message(
                        //     "assistant",
                        //     &format!("Executing tool call: {}", tool_call.name),
                        // );
                        lock.append_message("user", &correction_payload);
                        continue; // Recycle prompt back to LLM immediately with the corrective constraint
                    }
                };

                // if let Some(tool) = self.registry.tools.get(&tool_call.name) {
                ui.update_active_tool(&tool_call.name, &tool_call.arguments);
                {
                    let lock = self.session_ctx.lock().await;
                    ui.render(&*lock, &graph);
                }

                // Enforce Human-In-The-Loop explicit permission guardrails
                if permission_mode == PermissionMode::DefaultMode {
                    let is_authorized =
                        self.prompt_tool_authorization(&tool_call.name, &tool_call.arguments);
                    if !is_authorized {
                        let mut lock = self.session_ctx.lock().await;
                        lock.append_message(
                            "system",
                            &format!(
                                "Execution Denied: User rejected permission for '{}'.",
                                tool_call.name
                            ),
                        );
                        continue;
                    }
                }

                // Execute tool safely and extract observation responses
                match tool.execute(&tool_call.arguments).await {
                    Ok(output) => {
                        let mut formatted_payload = if ["shell", "bash", "check_diagnostics"]
                            .contains(&tool_call.name.as_str())
                        {
                            let structured_errors = DiagnosticParser::parse_cargo_json(&output);
                            if !structured_errors.is_empty() {
                                let markdown_report =
                                    DiagnosticParser::format_errors_for_llm(&structured_errors);
                                format!("[RAW TOOL OUTPUT]\n{}\n\n{}", output, markdown_report)
                            } else {
                                output
                            }
                        } else {
                            output
                        };

                        // --- APPLY THE CONTEXT SQUEEZER ---
                        // Sanitize massive logs or text blocks to protect agent context depth limits
                        formatted_payload =
                            ContextSqueezer::squeeze_terminal_output(&formatted_payload, 40);

                        let mut lock = self.session_ctx.lock().await;
                        lock.append_message(
                            "system",
                            &format!(
                                "Observation from '{}':\n{}",
                                tool_call.name, formatted_payload
                            ),
                        );
                    }
                    Err(err_msg) => {
                        let squeezed_err = ContextSqueezer::squeeze_terminal_output(&err_msg, 40);
                        let mut lock = self.session_ctx.lock().await;
                        lock.append_message(
                            "system",
                            &format!("Tool Execution Error: {}", squeezed_err),
                        );
                    }
                }
                // } else {
                //     let mut lock = self.session_ctx.lock().await;
                //     lock.append_message(
                //         "system",
                //         &format!(
                //             "Error: Tool '{}' missing from registry context.",
                //             tool_call.name
                //         ),
                //     );
                // }
            }
        }

        if step >= self.max_steps {
            ui.fail_task("Reached maximum turn iteration depth safety boundary limit.");
        }

        Ok(())
    }

    /// Extractor method isolating markdown JSON code block boundaries safely
    fn extract_clean_json(raw_text: &str) -> String {
        if let Some(start) = raw_text.find('{') {
            if let Some(end) = raw_text.rfind('}') {
                return raw_text[start..=end].to_string();
            }
        }
        raw_text.to_string()
    }

    /// Interactive terminal fallback gating mechanism for HITL Mode authorization queries
    fn prompt_tool_authorization(&self, tool_name: &str, args: &Value) -> bool {
        println!("\n🛡️  [AUTONOMOUS TOOL GUARDRAIL] Agent is invoking an external utility.");
        println!("🔧 Target Tool: \x1b[1;35m{}\x1b[0m", tool_name);
        println!(
            "Box Payload:\n{}",
            serde_json::to_string_pretty(args).unwrap_or_default()
        );
        println!("------------------------------------------------------------");

        loop {
            println!("\x1b[1mAllow tool invocation? [Y]es / [N]o: \x1b[0m");
            let _ = std::io::stdout().flush();

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                return false;
            }

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => return true,
                "n" | "no" => return false,
                _ => println!("⚠️ Invalid entry. Please reply with 'Y' or 'N'."),
            }
        }
    }
}
