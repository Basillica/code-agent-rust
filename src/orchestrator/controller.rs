use crate::action::permissions::PermissionMode;
use crate::intelligence::indexer::CodebaseIndexer;
use crate::orchestrator::engine::RefactorOrchestrator;
use crate::orchestrator::models::RefactorPlan;
use crate::state::session::SessionContext;
use crate::tools::registry::ToolRegistry;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AgentPromptController {
    session_ctx: Arc<Mutex<SessionContext>>,
    registry: Arc<ToolRegistry>,
    max_repair_attempts: usize,
    api_key: String,
    model_name: String,
    pub indexer: Arc<Mutex<CodebaseIndexer>>,
}

impl AgentPromptController {
    pub fn new(
        session_ctx: Arc<Mutex<SessionContext>>,
        registry: Arc<ToolRegistry>,
        api_key: String,
        model_name: String,
    ) -> Self {
        Self {
            session_ctx,
            registry,
            max_repair_attempts: 3,
            api_key,
            model_name: model_name, // Easily switchable to any specialized frontier model
            indexer: Arc::new(Mutex::new(CodebaseIndexer::new())),
        }
    }

    /// Performs an active index pass across codebases and maps structural capabilities into the context system
    pub async fn synchronize_codebase_intelligence(&self) -> Result<(), String> {
        println!("🔍 [Intelligence Engine] Scanning repository structure and index vectors...");
        let mut indexer_lock = self.indexer.lock().await;

        // Scan the active repository workspace sector
        indexer_lock.index_workspace(".")?;

        println!(
            "📊 [Intelligence Engine] Scan completed successfully. Registered {} unique code symbols.",
            indexer_lock.symbol_table.len()
        );
        Ok(())
    }

    /// Helper method to format indexed code structures into clear LLM context instructions
    async fn format_codebase_intelligence_ctx(&self) -> String {
        let indexer_lock = self.indexer.lock().await;
        if indexer_lock.symbol_table.is_empty() {
            return "No symbols indexed yet (empty or un-synchronized workspace).\n".to_string();
        }

        let mut output = String::from("ACTIVE WORKSPACE ARCHITECTURAL SYMBOLS:\n");
        for (name, symbols) in &indexer_lock.symbol_table {
            for sym in symbols {
                output.push_str(&format!(
                    "- Unique Symbol '{}' ({:?}) found at path: {}\n",
                    name,
                    sym.symbol_type,
                    sym.file_path.display()
                ));
            }
        }
        output
    }

    /// Primary entrypoint to submit a modification request down through the behavioral self-healing loop
    pub async fn dispatch_user_goal(
        &self,
        user_prompt: &str,
        compilation_cmd: &str,
        test_cmd: Option<&str>,
        permission_mode: PermissionMode,
    ) -> Result<(), String> {
        // =========================================================================
        // 🚀 STEP 1: POLYGLOT PRE-FLIGHT INTERCEPTION PASS (FIXED PATH SECTORS)
        // =========================================================================
        let runtime_indicators = [
            "Cargo.toml",
            "package.json",
            "go.mod",
            "pyproject.toml",
            "requirements.txt",
            "src",
        ];

        let is_empty_workspace = {
            !runtime_indicators
                .iter()
                .any(|indicator| std::path::Path::new(indicator).exists())
        };

        if is_empty_workspace {
            println!(
                "🌱 [Loop Controller] Blank execution workspace detected. Attempting automatic project initialization..."
            );

            if let Some(bootstrap_tool) = self.registry.tools.get("bootstrap_project") {
                let lower_prompt = user_prompt.to_lowercase();
                let inferred_lang = if lower_prompt.contains("rust")
                    || lower_prompt.contains("axum")
                    || lower_prompt.contains("tokio")
                    || lower_prompt.contains("cargo")
                {
                    "rust"
                } else if lower_prompt.contains("go")
                    || lower_prompt.contains("gin")
                    || lower_prompt.contains("fiber")
                {
                    "go"
                } else if lower_prompt.contains("typescript")
                    || lower_prompt.contains("ts")
                    || lower_prompt.contains("node")
                    || lower_prompt.contains("next")
                {
                    "typescript"
                } else if lower_prompt.contains("python")
                    || lower_prompt.contains("py")
                    || lower_prompt.contains("fastapi")
                    || lower_prompt.contains("flask")
                    || lower_prompt.contains("django")
                {
                    "python"
                } else {
                    "rust"
                };

                println!(
                    "🚀 [Loop Controller] Spawning an idiomatic '{}' project skeleton framework...",
                    inferred_lang
                );

                let bootstrap_args = serde_json::json!({
                    "language": inferred_lang,
                    "project_name": "app_workspace"
                });

                match bootstrap_tool.execute(&bootstrap_args).await {
                    Ok(msg) => {
                        println!("✅ [Loop Controller Bootstrap Success]: {}", msg);
                        // Force structural resynchronization immediately after bootstrapping
                        let _ = self.synchronize_codebase_intelligence().await;
                    }
                    Err(e) => println!(
                        "⚠️ [Loop Controller Bootstrap Warning]: Couldn't complete initialization: {}",
                        e
                    ),
                }
            } else {
                println!(
                    "⚠️ [Loop Controller Warning]: 'bootstrap_project' tool is not registered in the ToolRegistry."
                );
            }
        } else {
            // Repositories with existing files should be mapped right before generating plans
            let _ = self.synchronize_codebase_intelligence().await;
        }

        // =========================================================================
        // 🗺️ STEP 2: GENERATE THE INITIAL GRAPH PLAN (WITH INDEXER DATA)
        // =========================================================================
        let mut active_plan = match self.generate_upfront_plan(user_prompt).await {
            Ok(initial_plan) => initial_plan,
            Err(e) => {
                return Err(format!(
                    "Initial architectural planning stage failed: {}",
                    e
                ));
            }
        };

        // =========================================================================
        // 🔄 STEP 3: THE SELF-HEALING REPAIR & BEHAVIORAL TESTING LOOP
        // =========================================================================
        let mut attempt = 0;
        let mut error_history = String::new();

        while attempt < self.max_repair_attempts {
            println!(
                "\n🤖 [Loop Controller] Processing Plan Execution (Pass {}/{} using {})...",
                attempt + 1,
                self.max_repair_attempts,
                self.model_name
            );

            if attempt > 0 {
                println!("🔧 [Loop Controller] Querying Tactical Repair Plan from AI Provider...");

                let repair_context_prompt = format!(
                    "Your previous changes broke runtime constraints or functional verifications.\n\
                     [HISTORICAL REFLATION REGRESSION DIAGNOSTICS]:\n{}\n\
                     Please analyze the code failures above and return a clean, corrected JSON patch execution graph.\n\
                     Original objective: {}",
                    error_history, user_prompt
                );

                active_plan = match self.fetch_live_llm_plan(&repair_context_prompt).await {
                    Ok(repaired_plan) => repaired_plan,
                    Err(e) => {
                        return Err(format!(
                            "LLM Client API repair plan generation failed: {}",
                            e
                        ));
                    }
                };
            }

            // Lock workspace context, scope tightly, and execute the current plan
            let execution_result = {
                let mut lock = self.session_ctx.lock().await;
                let mut orchestrator = RefactorOrchestrator::new(&mut *lock);
                orchestrator
                    .execute_chain(active_plan.clone(), compilation_cmd, permission_mode)
                    .await
            };

            match execution_result {
                Ok(_) => {
                    // Stage 2 Validation: Compilation passed. Now execute behavioral test suite if provided.
                    if let Some(cmd) = test_cmd {
                        println!(
                            "🧪 [Loop Controller] Compilation successful. Evaluating runtime verification assertions: '{}'...",
                            cmd
                        );

                        let mut lock = self.session_ctx.lock().await;
                        // Dereference MutexGuard to extract our Week 2 Stateful background shell
                        let shell = (&mut *lock).get_or_init_shell()?;

                        match shell.run_command(cmd).await {
                            Ok((test_logs, exit_code)) => {
                                if exit_code == 0 {
                                    println!(
                                        "\n✨ [Loop Controller] Code passed both compilation checks and behavioral test specs!"
                                    );
                                    return Ok(());
                                } else {
                                    attempt += 1;
                                    println!(
                                        "⚠️ [Loop Controller] Logic error caught. Runtime validation tests rejected verification."
                                    );

                                    error_history.push_str(&format!(
                                        "--- Logic/Assertion Verification Failure Pass {} ---\nExecuted Command: {}\nLogs:\n{}\n\n",
                                        attempt, cmd, test_logs
                                    ));
                                }
                            }
                            Err(e) => {
                                return Err(format!(
                                    "Fatal runtime exception inside terminal verification: {}",
                                    e
                                ));
                            }
                        }
                    } else {
                        // No logic test suite passed; static compilation success is sufficient
                        println!(
                            "\n✨ [Loop Controller] Task completely resolved and verified via stable compilation!"
                        );
                        return Ok(());
                    }
                }
                Err(diagnostic_error) => {
                    attempt += 1;
                    println!(
                        "\n⚠️ [Loop Controller] Static verification failed. Appending compiler output to diagnostics."
                    );

                    error_history.push_str(&format!(
                        "--- Compilation Failure Pass {} ---\nCompiler Output:\n{}\n\n",
                        attempt, diagnostic_error
                    ));
                }
            }
        }

        Err(format!(
            "Autonomous repair loop exhausted after {} attempts without resolving compilation or passing behavioral suites.",
            self.max_repair_attempts
        ))
    }

    /// Performs the remote API call to fetch a structured tactical repair plan
    async fn fetch_live_llm_plan(&self, context_prompt: &str) -> Result<RefactorPlan, String> {
        let client = reqwest::Client::builder().no_proxy().build().unwrap();

        let mut tools_desc = String::new();
        for (name, tool) in &self.registry.tools {
            tools_desc.push_str(&format!(
                "- {}: {}. Schema: {}\n",
                name,
                tool.description(),
                serde_json::to_string(&tool.input_schema()).unwrap_or_default()
            ));
        }

        // Put the codebase indexer context to use for the tactical repair prompt
        let codebase_intelligence = self.format_codebase_intelligence_ctx().await;

        let system_instruction = format!(
            "You are an automated software engineering architect running inside an IDE platform.\n\
             Your task is to analyze code modifications and return a raw JSON structure matching this precise model schema:\n\n\
             {{\n\
               \"task_graph\": {{\n\
                 \"task_id_1\": {{\n\
                    \"id\": \"task_id_1\",\n\
                    \"target_file\": \"src/models.rs\",\n\
                    \"patch_instructions\": \"<<<<<<< SEARCH\\nold code line\\n=======\\nnew code line\\n>>>>>>> REPLACE\",\n\
                    \"dependencies\": [],\n\
                    \"status\": \"Pending\"\n\
                 }}\n\
               }},\n\
               \"execution_order\": [\"task_id_1\"]\n\
             }}\n\n\
             {}\n\n\
             CRITICAL CONSTRAINTS:\n\
             - patch_instructions MUST use the exact search/replace patch diff format shown above.\n\
             - target_file MUST be a valid relative path string.\n\
             - You have access to these structural editing capabilities: \n{}\n\
             - Return ONLY clean valid JSON. Do not write markdown blocks, explanations, or code fences.",
            codebase_intelligence, tools_desc
        );

        let body = json!({
            "model": self.model_name,
            "messages": [
                { "role": "system", "content": system_instruction },
                { "role": "user", "content": context_prompt }
            ],
            "stream": false,
            "options": { "temperature": 0.5 }
        });

        let response = client
            .post("http://localhost:11434/api/chat")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network transport connection drop: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Remote API rejected request (Status {}): {}",
                status, err_body
            ));
        }

        let res_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed parsing upstream inference packet structure: {}", e))?;

        let raw_content = res_json["message"]["content"].as_str().unwrap_or("").trim();

        let plan: RefactorPlan = serde_json::from_str(raw_content)
            .map_err(|e| format!(
                "JSON Schema layout mismatch against internal RefactorPlan definition: {}\nRaw output received: {}", 
                e, raw_content
            ))?;

        Ok(plan)
    }

    pub async fn generate_upfront_plan(&self, user_prompt: &str) -> Result<RefactorPlan, String> {
        println!("🤖 [Controller] Formulating upfront graph dependency plan via LLM...");
        let raw_json_response = self.query_llm_for_plan(user_prompt).await?;
        let plan: RefactorPlan = serde_json::from_str(&raw_json_response)
            .map_err(|e| format!(
                "JSON Schema layout mismatch during upfront planning generation: {}\nRaw output received: {}", 
                e, raw_json_response
            ))?;

        Ok(plan)
    }

    /// Performs the network transport layer execution to fetch the initial structural graph layout
    async fn query_llm_for_plan(&self, user_prompt: &str) -> Result<String, String> {
        let client = reqwest::Client::builder().no_proxy().build().unwrap();

        let mut tools_desc = String::new();
        for (name, tool) in &self.registry.tools {
            tools_desc.push_str(&format!("- {}: {}.\n", name, tool.description()));
        }

        // Put the codebase indexer context to use for the upfront planning prompt
        let codebase_intelligence = self.format_codebase_intelligence_ctx().await;

        let system_instruction = format!(
            "You are an expert software architect. The workspace environment has already been initialized.\n\
             Your job is to decompose the user's feature request into a logical series of file modifications.\n\
             You must output a single, flat JSON object matching this exact structural schema:\n\n\
             {{\n\
               \"task_graph\": {{\n\
                 \"init_source\": {{\n\
                    \"id\": \"init_source\",\n\
                    \"target_file\": \"src/main.rs\",\n\
                    \"patch_instructions\": \"<<<<<<< SEARCH\\n\\n=======\\n// Write code structures\\n>>>>>>> REPLACE\",\n\
                    \"dependencies\": [],\n\
                    \"status\": \"Pending\"\n\
                 }}\n\
               }},\n\
               \"execution_order\": [\"init_source\"]\n\
             }}\n\n\
             {}\n\n\
             POLYGLOT DESIGN RULES:\n\
             - Target files MUST adapt to the project language (e.g., use 'main.go' for Go, 'src/main.rs' for Rust).\n\
             - Ensure `execution_order` lists the task IDs in the exact sequence they should be executed.\n\
             - You have access to these capabilities: \n{}\n\
             - CRITICAL: Return ONLY valid, minified JSON. Never wrap your answer in markdown code blocks or code fences.",
            codebase_intelligence, tools_desc
        );

        let body = json!({
            "model": self.model_name,
            "messages": [
                { "role": "system", "content": system_instruction },
                { "role": "user", "content": user_prompt }
            ],
            "stream": false,
            "options": {
                "temperature": 0.2 // Low temperature for high structural rigidity during initialization passes
            }
        });

        let response = client
            .post("http://localhost:11434/api/chat")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                format!(
                    "Network connection drop during upfront plan generation: {}",
                    e
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Upstream Planning API rejected request (Status {}): {}",
                status, err_body
            ));
        }

        let res_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed parsing upstream planning response packet: {}", e))?;

        let raw_content = res_json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        Ok(raw_content)
    }
}
