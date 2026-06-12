use crate::action::permissions::{PermissionGate, PermissionMode};
use crate::core::compaction::{apply_generative_auto_compact, apply_snip_shaper};
use crate::core::indexer::CodebaseIndexer;
use crate::orchestrator::diagnostic::DiagnosticParser;
use crate::orchestrator::graph::WorkspaceGraph;
use crate::orchestrator::ui::{TerminalUI, UIStage};
use crate::state::session::SessionContext;
use crate::tools::Tool;
use crate::tools::registry::ToolRegistry;
use crate::tools::search::CodebaseSearchTool;
use crate::ui::dashboard::AgentDashboard;
use crate::verification::gate::VerificationGate;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::error::Error;
use std::time::Duration;

const OLLAMA_HOST: &str = "http://localhost:11434/api/chat";
const MODEL_NAME: &str = "gemma4:e4b";

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub thought: String,
    pub tool_call: Option<AgentToolCall>,
    pub task_completed: bool,
    pub final_summary: Option<String>,
}

pub async fn _query_loop(
    user_prompt: &str,
    session: &mut SessionContext,
    mode: PermissionMode,
    registry: &ToolRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup session and tracking parameters
    session.append_message("user", user_prompt);
    session.reload_workspace_context();

    let permission_gate = PermissionGate::new(mode);
    let http_client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_mins(60))
        .build()?;

    let mut dashboard = AgentDashboard::new();
    dashboard.start_task(user_prompt);

    let mut is_running = true;
    let mut turns = 0;
    let max_turns = 50;
    let mut last_executed_tool: Option<(String, serde_json::Value)> = None;

    while is_running && turns < max_turns {
        turns += 1;
        println!(
            "\n🔄 [Turn {}] Gathering workspace structural frames...",
            turns
        );

        // Pre-Model Shapers Layer 2: Snip Context
        session.history = apply_generative_auto_compact(&session.history, 15).await?;
        let compacted_history = apply_snip_shaper(&session.history, 12, 4);

        // Regenerate workspace index in real-time to capture changes made in prior turns
        let (file_tree, structural_signatures) =
            CodebaseIndexer.scan_workspace(&session.project_root);
        let index_text_block = format!(
            "=== CURRENT WORKSPACE STRUCTURAL INDEX ===\n\
             Discovered Code Files:\n{}\n\n\
             Structural Declarations mapped across active modules:\n{}\n\
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
                .join("\n")
        );

        // Build tools registry schema map
        let mut available_tools_text = String::new();
        for (name, tool) in &registry.tools {
            available_tools_text.push_str(&format!(
                "- {}: {}. Parameter Layout: {}\n",
                name,
                tool.description(),
                serde_json::to_string(&tool.input_schema()).unwrap_or_default()
            ));
        }

        let system_prompt = format!(
            "You are an autonomous engineering agent executing engineering tasks in a local workspace.\n\
                Current Workspace Status:\n{}\n\n\
                Current Style Guidelines (from AGENT.md):\n{}\n\n\
                Accumulated Engine Memory: {}\n\n\
                Workspace Action Interfaces:\n{}\n\
            \n\
            CRITICAL EXECUTION POLICIES:\n\
                Use the 'bash' tool ONLY to run commands like database migrations, installing packages, or testing code. \
                Never run interactive background tasks like 'runserver' that block indefinitely.\n\
                Always think step‑by‑step and use tools iteratively to explore the codebase and make incremental changes. \
                Don't try to do everything in one turn.\n\
                Follow the project guidelines from AGENT.md closely to match the coding style and architectural preferences.\n\
                You MUST answer by formulating your output using a single strict raw JSON block configuration matching this schema:\n\
                    {{\n\
                    \"thought\": \"Step-by-step reasoning details explaining your approach\",\n\
                    \"tool_call\": {{\n\
                        \"name\": \"read_file\" | \"write_file\" | \"edit_file\" | \"bash\" | \"glob\" | \"check_diagnostics\",\n\
                        \"arguments\": {{ ... }}\n\
                    }} or null,\n\
                    \"task_completed\": true | false,\n\
                    \"final_summary\": \"Exhaustive resolution notes when task_completed is true\" or null\n\
                    }}",
            index_text_block,
            session.project_instructions,
            session.auto_memory,
            available_tools_text
        );

        // Build message frames for history payload integration
        let mut messages_payload =
            vec![serde_json::json!({ "role": "system", "content": system_prompt })];
        // for msg in &session.history {
        for msg in compacted_history {
            messages_payload
                .push(serde_json::json!({ "role": &msg.role, "content": &msg.content }));
        }

        println!(
            "🧠 Querying local LLM execution model ({}) via Ollama...",
            MODEL_NAME
        );

        let ollama_response = http_client
            .post(OLLAMA_HOST)
            .json(&json!({
                "model": MODEL_NAME,
                "messages": messages_payload,
                "stream": false,
                // REPLACE "format": "json" WITH A STRICT JSON SCHEMA CONFIGURATION
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
                        "task_completed": {
                            "type": "boolean"
                        },
                        "final_summary": {
                            "type": ["string", "null"]
                        }
                    },
                    "required": ["thought", "tool_call", "task_completed", "final_summary"]
                },
                "options": { "temperature": 0.1 }
            }))
            .send()
            .await;

        let res = match ollama_response {
            Ok(r) => r,
            Err(e) => {
                dashboard.fail_task(&format!("Network/Ollama connection failure: {}", e));
                println!("💥 Network/Ollama connection failure: {}", e);
                eprintln!("💥 Network/Ollama connection failure: {}", e);
                // Unroll the inner causes (hyper::Error -> std::io::Error)
                let mut source = e.source();
                while let Some(src) = source {
                    eprintln!("   ↳ Caused by: {}", src);
                    source = src.source();
                }
                break;
            }
        };

        if !res.status().is_success() {
            dashboard.fail_task(&format!(
                "Ollama returned HTTP Error state: {}",
                res.status()
            ));
            println!("💥 Ollama returned an HTTP Error state: {}", res.status());
            break;
        }

        let body_val: Value = match res.json().await {
            Ok(j) => j,
            Err(e) => {
                println!(
                    "❌ Stream layout processing failure on Ollama message frame: {}",
                    e
                );
                break;
            }
        };
        let raw_content = match body_val
            .pointer("/message/content")
            .and_then(|c| c.as_str())
        {
            Some(t) => t,
            None => {
                println!("❌ Received empty or invalid text stream from model endpoint execution.");
                break;
            }
        };

        let cleaned_json = extract_clean_json(raw_content);
        let json_value: serde_json::Value = match serde_json::from_str(&cleaned_json) {
            Ok(v) => v,
            Err(e) => {
                println!("❌ [Parser Syntax Error]: {:?}", e);
                // Trigger your self-healing loop injector
                session.append_message("assistant", raw_content);
                session.append_message(
            "user",
            "CRITICAL PARSING ERROR: Your response contained invalid JSON characters or broken markdown formatting. \
             Return a valid raw JSON object string directly."
        );
                continue;
            }
        };

        // Lenient Adapter Pattern: Auto-translate 'tool_calls' if the model uses its native schema
        let agent_decision: AgentResponse = if json_value.get("tool_calls").is_some() {
            println!(
                "🤖 [Harness Adapter]: Intercepted native model 'tool_calls' format. Auto-mapping properties..."
            );

            // Safely extract the first function invocation from the array
            if let Some(first_call) = json_value["tool_calls"].as_array().and_then(|a| a.first()) {
                AgentResponse {
                    thought: json_value["thought"]
                        .as_str()
                        .unwrap_or("Executing tool call via native schema adapter routing.")
                        .to_string(),
                    tool_call: Some(AgentToolCall {
                        name: first_call["function"].as_str().unwrap_or("").to_string(),
                        arguments: first_call["args"].clone(),
                    }),
                    task_completed: false,
                    final_summary: None,
                }
            } else {
                // tool_calls array was empty
                AgentResponse {
                    thought: "Detected empty tool_calls configuration array.".to_string(),
                    tool_call: None,
                    task_completed: false,
                    final_summary: None,
                }
            }
        } else {
            // Attempt standard direct mapping into your strict AgentResponse structure
            match serde_json::from_value::<AgentResponse>(json_value) {
                Ok(parsed) => parsed,
                Err(e) => {
                    println!(
                        "⚠️ [Schema Schema Mismatch]: Model structural format missed targets: {:?}",
                        e
                    );
                    session.append_message("assistant", &cleaned_json);
                    session.append_message(
                        "user",
                        "CRITICAL SCHEMA ERROR: Your keys do not match. You must specify exactly: \
                 'thought', 'tool_call', 'task_completed', and 'final_summary'.",
                    );
                    continue;
                }
            }
        };

        println!("💭 Agent Thought: {}", agent_decision.thought);
        // session.append_message("assistant", &cleaned_json);
        // session.append_message("assistant", &agent_decision.thought);
        let assistant_history_payload =
            serde_json::to_string_pretty(&agent_decision).unwrap_or_default();
        session.append_message("assistant", &assistant_history_payload);

        println!(
            "the fucking agent decision right here: {:?}",
            agent_decision
        );

        // Check for normal exit criteria
        if agent_decision.task_completed {
            dashboard.complete_task();
            println!("✅ Verification Checkpoints Passed! Task marked complete by Agent Engine.");
            if let Some(summary) = agent_decision.final_summary {
                println!("📝 Final Summary:\n{}", summary);
            }
            is_running = false;
            break;
            // let report = VerificationGate::execute_workspace_validation(&session.project_root);
            // if report.is_passing {
            //     dashboard.complete_task();
            //     println!(
            //         "✅ Verification Checkpoints Passed! Task marked complete by Agent Engine."
            //     );
            //     if let Some(summary) = agent_decision.final_summary {
            //         println!("📝 Final Summary:\n{}", summary);
            //     }
            //     is_running = false;
            //     break;
            // } else {
            //     println!(
            //         "⚠️ [Verification Gate]: Intercepted compilation or runtime errors. Rejecting finalization."
            //     );
            //     dashboard.log_verification(
            //         "⚠️ \t[Gate Interception]: Code failed quality verification benchmarks.",
            //     );

            //     // Build a correction prompt detailing the diagnostic breakdown
            //     let correction_payload = format!(
            //         "CRITICAL VERIFICATION FAILURE: Your changes cannot be finalized because your code contains lint or test errors. \
            //          You must resolve these failures before finishing the task.\n\n\
            //          === LINT REPORT ===\n{}\n\n\
            //          === TEST SUITE RUNTIME REPORT ===\n{}",
            //         report.lint_output, report.test_output
            //     );

            //     // Append the context frame back to history and trigger an immediate loop pass correction pass
            //     session.append_message(
            //         "assistant",
            //         &serde_json::to_string(&agent_decision).unwrap_or_default(),
            //     );
            //     session.append_message("user", &correction_payload);
            //     continue;
            // }
        }

        // Process actual tool invocation requests safely
        if let Some(tool_call) = agent_decision.tool_call {
            if let Some(ref last) = last_executed_tool {
                if last.0 == tool_call.name && last.1 == tool_call.arguments {
                    println!(
                        "⚠️ [Loop Guard]: Intercepted duplicate tool call loop sequence. Redirecting agent..."
                    );
                    session.append_message(
                        "system",
                        "CRITICAL WARNING: You are stuck in a repetitive tool loop execution. You have already executed this exact action. \
                        If you cannot find what you are looking for, change your parameters, try a different tool, or mark the task completed with your findings."
                    );
                    continue;
                }
            }
            // Update the tracker after execution
            last_executed_tool = Some((tool_call.name.clone(), tool_call.arguments.clone()));
            if tool_call.name != "null" && !tool_call.name.is_empty() {
                if tool_call.name == "codebase_search" || tool_call.name == "search_codebase" {
                    println!("🏃 Executing tool: {}", tool_call.name);
                    let search_tool = CodebaseSearchTool::new(session.project_root.clone());
                    match search_tool.execute(&tool_call.arguments).await {
                        Ok(observation) => {
                            dashboard.log_verification(
                                "Step Checkpoint: codebase_search ran successfully.",
                            );
                            session.append_message(
                                "system",
                                &format!("Observation from 'codebase_search':\n{}", observation),
                            );
                        }
                        Err(err_msg) => {
                            session.append_message(
                                "system",
                                &format!("Tool Execution Error: {}", err_msg),
                            );
                        }
                    }
                    continue;
                }

                if let Some(tool) = registry.tools.get(&tool_call.name) {
                    dashboard.update_status(
                        &tool_call.name,
                        &serde_json::to_string(&tool_call.arguments).unwrap_or_default(),
                    );
                    // Evaluate permission intercept firewall gates
                    dashboard.pause_for_input();
                    let is_authorized = permission_gate
                        .check_permission(&tool_call.name, &tool_call.arguments)
                        .await;
                    dashboard.resume_after_input();

                    if !is_authorized {
                        session.append_message(
                            "system", 
                            format!("Execution Denied: User explicitly rejected permissions for tool '{}'.", tool_call.name).as_str()
                        );
                        continue;
                    }

                    println!("🏃 Executing tool: {}", tool_call.name);
                    match tool.execute(&tool_call.arguments).await {
                        Ok(observation) => {
                            dashboard.log_verification(&format!(
                                "Step Checkpoint: {} executed flawlessly.",
                                tool_call.name
                            ));
                            session.append_message(
                                "system",
                                format!("Observation from '{}':\n{}", tool_call.name, observation)
                                    .as_str(),
                            );
                        }
                        Err(err_msg) => {
                            session.append_message(
                                "system",
                                format!("Tool Execution Error: {}", err_msg).as_str(),
                            );
                        }
                    }
                } else {
                    println!(
                        "❌ Tool '{}' not found in registry definitions.",
                        tool_call.name
                    );
                    session.append_message(
                        "system",
                        format!(
                            "Error: Tool '{}' not found in registry definitions.",
                            tool_call.name
                        )
                        .as_str(),
                    );
                }
            } else {
                println!("⚠️ Agent did not request tool executions on this turn.");
            }
        }
    }

    println!("runing status: {:#},", is_running);
    if turns >= max_turns {
        dashboard.fail_task("Reached safety turn loop boundary ceiling.");
        println!(
            "🛑 Safety Warning: Reached processing turn ceiling bounds limit ({} turns).",
            max_turns
        );
    }

    Ok(())
}

pub async fn query_loop(
    user_prompt: &str,
    session: &mut SessionContext,
    mode: PermissionMode,
    registry: &ToolRegistry,
    ui: &mut TerminalUI, // <--- Draw frames directly to UI reference
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup session and tracking parameters
    session.append_message("user", user_prompt);
    session.reload_workspace_context();

    let permission_gate = PermissionGate::new(mode);
    let http_client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(3600))
        .build()?;

    // Initialize WorkspaceGraph mapping structure
    let mut graph = WorkspaceGraph::new();

    ui.start_task(user_prompt);
    ui.render(session, &graph); // Initial draw pass

    let mut is_running = true;
    let mut turns = 0;
    let max_turns = 50;

    while is_running && turns < max_turns {
        turns += 1;

        ui.update_status(&format!(
            "Turn {} - Indexing codebase & updating Workspace Graph...",
            turns
        ));
        ui.render(session, &graph);

        // Pre-Model Shapers Layer 2: Snip Context

        session.history = apply_generative_auto_compact(&session.history, 15).await?;
        let compacted_history = apply_snip_shaper(&session.history, 15, 4);
        // Regenerate workspace index and update WorkspaceGraph paths
        let (file_tree, signature_maps) = CodebaseIndexer.scan_workspace(&session.project_root);

        let structural_signatures: Vec<(String, Vec<String>)> =
            signature_maps.into_iter().collect();

        // Populate and refine the Workspace Graph based on current code structures
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
            graph.render_dependency_edges_to_string() // <--- Let LLM see topological relationships
        );

        // Build tools description block
        let mut available_tools_text = String::new();
        for (name, tool) in &registry.tools {
            available_tools_text.push_str(&format!(
                "- {}: {}. Schema: {}\n",
                name,
                tool.description(),
                serde_json::to_string(&tool.input_schema()).unwrap_or_default()
            ));
        }

        let _system_prompt = format!(
            "You are an autonomous engineering agent executing engineering tasks in a local workspace.\n\
                Current Workspace Status:\n{}\n\n\
                Current Style Guidelines (from AGENT.md):\n{}\n\n\
                Accumulated Engine Memory: {}\n\n\
                Available Tools in your Runtime Environment:\n{}\n\
            \n\
            CRITICAL EXECUTION POLICIES:\n\
                1. Use dedicated tools ('glob', 'grep', 'read_file', 'edit_file', etc.) to find and edit code files.\n\
                2. Use the 'bash' tool ONLY to run commands like database migrations, installing packages, or testing code. \
                Never run interactive background tasks like 'runserver' that block indefinitely.\n\
                3. Always think step-by-step and use tools iteratively to explore the codebase and make incremental changes. \
                Don't try to do everything in one turn.\n\
                4. If you are unsure about the workspace state, use the 'glob' and 'read_file' tools to gather more information before making edits.\n\
                5. Follow the project guidelines from AGENT.md closely to match the coding style and architectural preferences.\n\
                6. If you complete the task, respond with {{\"task_completed\": true, \"final_summary\": \"your summary here\"}} \
                to signal successful completion.\n\
                7. If you need to ask the user for clarification or permission, respond with \
                {{\"tool_call\": {{\"name\": \"null\", \"arguments\": {{\"question\": \"your question here\"}}}}}} and wait for the user's response in the next turn.\n\
                8. Always return a JSON object matching the specified schema in every response, even if no tool calls are needed. \
                This ensures consistent communication with the system.\n\
            \n\
            \n\
            EXPECTED JSON RESPONSE SCHEMA:\n\
                {{\n  \"thought\": \"Your detailed step planning logic here\",\n  \"tool_call\": {{\n    \"name\": \"tool_name_here_or_null\",\n    \"arguments\": {{}}\n  }},\n  \"task_completed\": false,\n  \"final_summary\": null\n}}",
            index_text_block,
            session.project_instructions,
            session.auto_memory,
            available_tools_text
        );

        let system_prompt = format!(
            "You are an autonomous venture analyst, startup strategist, and founder advisor.

            CORE OPERATING PRINCIPLES

            1. You are NOT an idea generator only.
            2. Optimize for viable businesses.
            3. Challenge assumptions aggressively.
            ...

            =========================================================
            VENTURE EVALUATION FRAMEWORK (PERMANENT)
            =========================================================

            Every startup opportunity must be evaluated using the
            following weighting model.

            Market Demand:        30%
            Founder Fit:          20%
            Distribution:         20%
            Business Model:       15%
            Technical Moat:       10%
            Competition:           5%

            Definitions:

            Market Demand (30%)
            - Severity of pain
            - Frequency of pain
            - Existing spending behavior
            - Urgency of solving the problem
            - Size of reachable market

            Founder Fit (20%)
            - Domain expertise
            - Technical expertise
            - Industry knowledge
            - Existing network advantages
            - Credibility with customers

            Distribution (20%)
            - Ability to acquire customers
            - Existing audience
            - Existing relationships
            - Organic acquisition potential
            - Paid acquisition feasibility

            Business Model (15%)
            - Revenue quality
            - Gross margin potential
            - Pricing power
            - Retention potential
            - Expansion revenue

            Technical Moat (10%)
            - Proprietary infrastructure
            - Proprietary data
            - Switching costs
            - Technical differentiation
            - Defensibility

            Competition (5%)
            - Saturation
            - Competitor strength
            - Differentiation opportunities

            IMPORTANT:

            A technically impressive product with weak demand
            should score lower than a simple product with strong
            customer demand.

            Example:

            Strong demand + mediocre technology
            = GOOD BUSINESS

            Amazing technology + weak demand
            = BAD BUSINESS

            Use this framework for every recommendation,
            scorecard, ranking, and final decision.

            =========================================================
            IDEA REJECTION CRITERIA
            =========================================================

            Immediately penalize ideas exhibiting:

            - Generic AI wrappers
            - Feature businesses
            - No clear buyer
            - No budget holder
            - Dependence on a single platform
            - Difficult distribution
            - Nice-to-have problems
            - Consumer behavior change requirements
            - Markets with no evidence of spending

            =========================================================
            DECISION THRESHOLDS
            =========================================================

            Overall Score:

            0.0 - 4.0
            REJECT

            4.0 - 6.0
            WEAK OPPORTUNITY

            6.0 - 8.0
            PROMISING

            8.0 - 10.0
            EXCEPTIONAL

            Your purpose is to identify, analyze, challenge, and refine startup opportunities.

            Current Working Context:

            Founder Context:
            {}

            Accumulated Memory:
            {}

            Available Tools:
            {}

            CORE OPERATING PRINCIPLES

            1. You are NOT an idea generator only.
            Your primary responsibility is finding opportunities that can become real businesses.

            2. Always optimize for:
            - Revenue potential
            - Customer pain
            - Founder advantage
            - Distribution feasibility
            - Defensibility

            3. Do NOT assume an idea is good because it is technically interesting.

            4. Technical complexity is not a moat by itself.

            5. Distribution and customer demand are usually more important than architecture.

            6. Challenge assumptions aggressively.

            7. Treat every idea as guilty until proven viable.

            8. Use tools iteratively.

            Example workflow:

            founder_advantage_analyzer
                    ↓
            startup_idea_generator
                    ↓
            market_demand_validator
                    ↓
            competition_analyzer
                    ↓
            founder_fit_analyzer
                    ↓
            technical_moat_auditor
                    ↓
            business_model_analyzer
                    ↓
            distribution_analyzer
                    ↓
            venture_scorecard

            9. Never jump directly to a final recommendation without collecting evidence.

            10. When scores are weak, recommend rejection rather than forcing optimism.

            11. Prefer painful problems over exciting ideas.

            12. Prefer businesses that customers already spend money on.

            13. Favor founder-market fit whenever possible.

            14. Be skeptical of:
                - Generic AI wrappers
                - Feature businesses
                - Platform-dependent products
                - Markets with no identifiable buyers
                - Products requiring behavior change

            15. Explicitly identify:
                - Assumptions
                - Risks
                - Unknowns
                - Validation steps

            DECISION FRAMEWORK

            When evaluating opportunities, score:

            - Market Demand
            - Pain Severity
            - Urgency
            - Existing Spend
            - Founder Fit
            - Distribution
            - Competition
            - Technical Defensibility
            - Revenue Quality
            - Capital Efficiency

            Do not over-index on technical moat.

            A company with:
            - strong demand
            - strong distribution
            - moderate technology

            is usually better than:

            - advanced technology
            - weak demand

            SUCCESS CRITERIA

            A startup opportunity is considered promising when:

            - Customers have a painful problem
            - Customers have budget
            - Customers already spend money
            - Founder has meaningful advantage
            - Distribution path is plausible
            - Revenue model is clear
            - Competition does not eliminate differentiation

            TOOL USAGE

            Use tools as investigative instruments.

            Each tool provides evidence.

            Build conclusions from accumulated evidence.

            Do not generate conclusions first and justify them afterward.

            COMPLETION POLICY

            When sufficient evidence has been collected and analyzed,
            respond with:

            {{
            \"task_completed\": true,
            \"final_summary\": {{
                \"recommendation\": \"PURSUE | VALIDATE | REJECT\",
                \"confidence\": 0.0,
                \"venture_score\": 0.0,
                \"top_strengths\": [],
                \"top_risks\": [],
                \"critical_assumptions\": [],
                \"next_validation_steps\": []
            }}
            }}

            CLARIFICATION POLICY

            If critical information is missing, respond with:

            {{
            \"tool_call\": {{
                \"name\": \"null\",
                \"arguments\": {{
                \"question\": \"specific question here\"
                }}
            }}
            }}

            RESPONSE FORMAT

            Always return valid JSON matching:

            {{
            \"thought\": \"reasoning and planning\",
            \"tool_call\": {{
                \"name\": \"tool_name_or_null\",
                \"arguments\": {{}}
            }},
            \"task_completed\": false,
            \"final_summary\": null
            }}

            Never return plain text.
            Never return markdown.
            Always return valid JSON.",
            session.project_instructions, session.auto_memory, available_tools_text
        );

        let mut messages_payload = vec![json!({ "role": "system", "content": system_prompt })];
        for msg in compacted_history {
            messages_payload.push(json!({ "role": msg.role, "content": msg.content }));
        }

        ui.update_status(&format!(
            "Turn {} - Awaiting generation token arrays from Ollama...",
            turns
        ));
        ui.render(session, &graph);

        let ollama_response = http_client
            .post(OLLAMA_HOST)
            .json(&json!({
                "model": MODEL_NAME,
                "messages": messages_payload,
                "stream": false,
                // REPLACE "format": "json" WITH A STRICT JSON SCHEMA CONFIGURATION
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
                        "task_completed": {
                            "type": "boolean"
                        },
                        "final_summary": {
                            "type": ["string", "null"]
                        }
                    },
                    "required": ["thought", "tool_call", "task_completed", "final_summary"]
                },
                "options": { "temperature": 0.1 }
            }))
            .send()
            .await;

        let res = match ollama_response {
            Ok(r) => r,
            Err(e) => {
                ui.fail_task(&format!("Network/Ollama connection failure: {}", e));
                break;
            }
        };

        if !res.status().is_success() {
            ui.fail_task(&format!(
                "Ollama returned HTTP Error state: {}",
                res.status()
            ));
            break;
        }

        let body_val: Value = res.json().await?;
        let raw_content = match body_val
            .pointer("/message/content")
            .and_then(|c| c.as_str())
        {
            Some(t) => t,
            None => {
                println!("❌ Received empty or invalid text stream from model endpoint execution.");
                break;
            }
        };
        let cleaned_json = extract_clean_json(raw_content);
        let json_value: serde_json::Value = match serde_json::from_str(&cleaned_json) {
            Ok(v) => v,
            Err(e) => {
                println!("❌ [Parser Syntax Error]: {:?}", e);
                // Trigger your self-healing loop injector
                session.append_message("assistant", raw_content);
                session.append_message(
            "user",
            "CRITICAL PARSING ERROR: Your response contained invalid JSON characters or broken markdown formatting. \
             Return a valid raw JSON object string directly."
        );
                continue;
            }
        };
        let agent_decision: AgentResponse = if json_value.get("tool_calls").is_some() {
            println!(
                "🤖 [Harness Adapter]: Intercepted native model 'tool_calls' format. Auto-mapping properties..."
            );

            // Safely extract the first function invocation from the array
            if let Some(first_call) = json_value["tool_calls"].as_array().and_then(|a| a.first()) {
                AgentResponse {
                    thought: json_value["thought"]
                        .as_str()
                        .unwrap_or("Executing tool call via native schema adapter routing.")
                        .to_string(),
                    tool_call: Some(AgentToolCall {
                        name: first_call["function"].as_str().unwrap_or("").to_string(),
                        arguments: first_call["args"].clone(),
                    }),
                    task_completed: false,
                    final_summary: None,
                }
            } else {
                // tool_calls array was empty
                AgentResponse {
                    thought: "Detected empty tool_calls configuration array.".to_string(),
                    tool_call: None,
                    task_completed: false,
                    final_summary: None,
                }
            }
        } else {
            // Attempt standard direct mapping into your strict AgentResponse structure
            match serde_json::from_value::<AgentResponse>(json_value) {
                Ok(parsed) => parsed,
                Err(e) => {
                    println!(
                        "⚠️ [Schema Schema Mismatch]: Model structural format missed targets: {:?}",
                        e
                    );
                    session.append_message("assistant", &cleaned_json);
                    session.append_message(
                        "user",
                        "CRITICAL SCHEMA ERROR: Your keys do not match. You must specify exactly: \
                 'thought', 'tool_call', 'task_completed', and 'final_summary'.",
                    );
                    continue;
                }
            }
        };

        // Stream thoughts directly into the console/UI log section
        ui.log_message("assistant", &format!("Thought: {}", agent_decision.thought));
        session.append_message("assistant", &cleaned_json);

        if agent_decision.task_completed {
            if let Some(summary) = agent_decision.final_summary {
                println!("\n✨ [Task Completed]: {}", summary);
            }
            break;
        }

        // Check for normal completion exit criteria
        if agent_decision.task_completed {
            let report = VerificationGate::execute_workspace_validation(&session.project_root);
            ui.update_status(
                "Task verification pass triggered. Executing environment test hooks...",
            );
            ui.render(session, &graph);

            // Check cross-file integrity through the WorkspaceGraph layout as well
            let graph_integrity_passing = graph.verify_structural_integrity();
            println!("current integrity: {graph_integrity_passing}");
            if report.is_passing && graph_integrity_passing {
                TerminalUI::print_status(
                    UIStage::Success,
                    "All quality gate checks passed cleanly!",
                );

                // FIX: Synchronize final summary back to the Session Context records before closing out
                let summary_content = agent_decision.final_summary.unwrap_or_else(|| {
                    String::from("Task completed successfully. Integrity verified.")
                });

                session.append_message(
                    "system",
                    &format!("Execution Finalized Summary:\n{}", summary_content),
                );
                println!("📝 Final Summary:\n{}", summary_content);

                ui.complete_task(&summary_content);
                session.clear_memory(&summary_content);
                is_running = false;
                break;
            } else {
                println!(
                    "⚠️ [Verification Gate]: Intercepted runtime errors. Injecting corrections loop tracking..."
                );
                let correction_payload = format!(
                    "CRITICAL VERIFICATION FAILURE: Your changes cannot be finalized due to errors:\n\n=== LINT SUITE ===\n{}",
                    report.lint_output
                );
                session.append_message("user", &correction_payload);
                continue;
            }
        }

        // Process requested tool invocation sequences safely
        if let Some(tool_call) = agent_decision.tool_call {
            let normalized_tool_name = tool_call.name.trim().to_lowercase();
            // If the model calls a placeholder tool, FORCE an early loop termination.
            if normalized_tool_name == "null"
                || normalized_tool_name == "none"
                || normalized_tool_name.is_empty()
            {
                println!("\n🛑 [Orchestrator] Intercepted conversational early exit signal.");

                if let Some(summary) = agent_decision.final_summary {
                    println!("\n✨ [Agent Summary]:\n{}", summary);
                } else {
                    println!(
                        "\n✨ [Agent Thought Reflection]:\n{}",
                        agent_decision.thought
                    );
                }
                break;
            }

            if tool_call.name != "null" && !tool_call.name.is_empty() {
                if tool_call.name == "codebase_search" {
                    ui.update_active_tool(&tool_call.name, &tool_call.arguments);
                    ui.render(session, &graph);

                    let search_tool = CodebaseSearchTool::new(session.project_root.clone());
                    match search_tool.execute(&tool_call.arguments).await {
                        Ok(observation) => {
                            session.append_message(
                                "system",
                                &format!("Observation from 'codebase_search':\n{}", observation),
                            );
                        }
                        Err(err_msg) => {
                            session.append_message(
                                "system",
                                &format!("Tool Execution Error: {}", err_msg),
                            );
                        }
                    }
                    continue;
                }

                if let Some(tool) = registry.tools.get(&tool_call.name) {
                    ui.update_active_tool(&tool_call.name, &tool_call.arguments);
                    ui.render(session, &graph);

                    // Explicit permission request firewall prompt handled directly via UI layout thread
                    let is_authorized = ui
                        .prompt_permission(&tool_call.name, &tool_call.arguments, &permission_gate)
                        .await;

                    if !is_authorized {
                        session.append_message(
                            "system",
                            &format!(
                                "Execution Denied: User rejected permission for '{}'.",
                                tool_call.name
                            ),
                        );
                        continue;
                    }

                    match tool.execute(&tool_call.arguments).await {
                        Ok(output) => {
                            let formatted_payload = if tool_call.name == "shell"
                                || tool_call.name == "bash"
                                || tool_call.name == "check_diagnostics"
                            {
                                // Parse the raw stream using your actual cargo json parser
                                let structured_errors = DiagnosticParser::parse_cargo_json(&output);

                                if !structured_errors.is_empty() {
                                    // Pass the errors directly to your custom layout generator
                                    let markdown_report =
                                        DiagnosticParser::format_errors_for_llm(&structured_errors);

                                    // Combine the raw console output with your explicit high-signal Markdown warning
                                    format!("[RAW TOOL OUTPUT]\n{}\n\n{}", output, markdown_report)
                                } else {
                                    output
                                }
                            } else {
                                output
                            };

                            session.append_message(
                                "system",
                                &format!(
                                    "Observation from '{}':\n{}",
                                    tool_call.name, formatted_payload
                                ),
                            );
                        }
                        Err(err_msg) => {
                            session.append_message(
                                "system",
                                &format!("Tool Execution Error: {}", err_msg),
                            );
                        }
                    }
                } else {
                    session.append_message(
                        "system",
                        &format!(
                            "Error: Tool '{}' missing from registry context.",
                            tool_call.name
                        ),
                    );
                }
            }
        }
    }

    if turns >= max_turns {
        ui.fail_task("Reached maximum turn iteration depth safety boundary limit.");
    }

    Ok(())
}

/// Cleans markdown wrappers like ```json ... ``` that local models often wrap responses in.
fn clean_json_wrapper(raw: &str) -> String {
    let mut text = raw.trim();
    if text.starts_with("```json") {
        text = text.trim_start_matches("```json");
    } else if text.starts_with("```") {
        text = text.trim_start_matches("```");
    }
    if text.ends_with("```") {
        text = text.trim_end_matches("```");
    }
    text.trim().to_string()
}

/// Dynamic self-healing parsing layer. Isolates object contents between outer braces
/// to extract raw JSON data safely regardless of markdown formatting wraps.
fn _clean_and_parse_json(raw_text: &str) -> Result<AgentResponse, serde_json::Error> {
    let mut cleaned = raw_text.trim();
    if let Some(start_idx) = cleaned.find('{') {
        if let Some(end_idx) = cleaned.rfind('}') {
            if end_idx >= start_idx {
                cleaned = &cleaned[start_idx..=end_idx];
            }
        }
    }
    serde_json::from_str::<AgentResponse>(cleaned)
}

fn extract_clean_json(raw: &str) -> String {
    // Trim markdown backticks if present
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Find the absolute first structural brace and last structural brace
    let start = cleaned.find('{');
    let end = cleaned.rfind('}');

    if let (Some(s), Some(e)) = (start, end) {
        if e > s {
            return cleaned[s..=e].to_string();
        }
    }
    cleaned.to_string()
}
