use crate::orchestrator::engine::RefactorOrchestrator;
use crate::orchestrator::models::RefactorPlan;
use crate::state::session::SessionContext;
use crate::tools::registry::ToolRegistry;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

const MODEL_NAME: &str = "gemma4:e4b";

pub struct AgentPromptController {
    session_ctx: Arc<Mutex<SessionContext>>,
    registry: Arc<ToolRegistry>,
    max_repair_attempts: usize,
    api_key: String,
    model_name: String,
}

impl AgentPromptController {
    pub fn new(
        session_ctx: Arc<Mutex<SessionContext>>,
        registry: Arc<ToolRegistry>,
        api_key: String,
    ) -> Self {
        Self {
            session_ctx,
            registry,
            max_repair_attempts: 3,
            api_key,
            model_name: MODEL_NAME.to_string(), // Easily switchable to any specialized frontier model
        }
    }

    /// Primary entrypoint to submit a loose modification request down through the self-healing loop
    pub async fn dispatch_user_goal(
        &self,
        user_prompt: &str,
        compilation_cmd: &str,
    ) -> Result<(), String> {
        // =========================================================================
        // 🚀 POLYGLOT PRE-FLIGHT INTERCEPTION PASS
        // =========================================================================
        // Scan for baseline project indicators across multiple runtime ecosystems
        let runtime_indicators = [
            "Cargo.toml",       // Rust
            "package.json",     // Node / TypeScript
            "go.mod",           // Go
            "pyproject.toml",   // Python Modern
            "requirements.txt", // Python Legacy
            "src",              // Generic codebase directory structure
        ];

        let is_empty_workspace = !runtime_indicators
            .iter()
            .any(|indicator| std::path::Path::new(indicator).exists());

        if is_empty_workspace {
            println!(
                "🌱 [Loop Controller] Blank execution workspace detected. Attempting automatic project initialization..."
            );

            // Access the bootstrap capability directly from our active tool definitions
            if let Some(bootstrap_tool) = self.registry.tools.get("bootstrap_project") {
                let lower_prompt = user_prompt.to_lowercase();

                // Intelligently classify ecosystem routing based on keyword extraction arrays
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
                    "rust" // Safe fallback standard default choice
                };

                println!(
                    "🚀 [Loop Controller] Spawning an idiomatic '{}' project skeleton framework...",
                    inferred_lang
                );

                let bootstrap_args = serde_json::json!({
                    "language": inferred_lang,
                    "project_name": "app_workspace"
                });

                // Execute the tool synchronously to populate the disk layout before generating code
                match bootstrap_tool.execute(&bootstrap_args).await {
                    Ok(msg) => println!("✅ [Loop Controller Bootstrap Success]: {}", msg),
                    Err(e) => println!(
                        "⚠️ [Loop Controller Bootstrap Warning]: Couldn't complete initialization step: {}",
                        e
                    ),
                }
            } else {
                println!(
                    "⚠️ [Loop Controller Warning]: 'bootstrap_project' tool is not registered in the ToolRegistry."
                );
            }
        }
        // =========================================================================

        // =========================================================================
        // 🗺️ STEP 2: GENERATE THE INITIAL GRAPH PLAN (FIRST USE PASS)
        // =========================================================================
        // We call this BEFORE entering the repair loop. This asks the LLM to analyze
        // the request and output our master multi-file RefactorPlan JSON payload.
        let mut active_plan = match self.generate_upfront_plan(user_prompt).await {
            Ok(initial_plan) => initial_plan,
            Err(e) => {
                return Err(format!(
                    "Initial architectural planning stage failed: {}",
                    e
                ));
            }
        };

        let mut attempt = 0;
        let mut active_feedback = user_prompt.to_string();

        while attempt < self.max_repair_attempts {
            println!(
                "\n🤖 [Loop Controller] Querying Upstream AI Planner (Pass {}/{} using {})...",
                attempt + 1,
                self.max_repair_attempts,
                self.model_name
            );

            // 1. Fetch live structured planning layout directly from remote AI provider
            let plan = match self.fetch_live_llm_plan(&active_feedback).await {
                Ok(p) => p,
                Err(e) => {
                    return Err(format!(
                        "LLM Client API parsing or transmission failure: {}",
                        e
                    ));
                }
            };

            // 2. Lock the workspace context and run the visual transaction engine
            let mut lock = self.session_ctx.lock().await;
            let mut orchestrator = RefactorOrchestrator::new(&mut *lock);

            match orchestrator.execute_chain(plan, compilation_cmd).await {
                Ok(_) => {
                    println!(
                        "\n✨ [Loop Controller] Task completely resolved, verified, and safely committed!"
                    );
                    return Ok(());
                }
                Err(diagnostic_error) => {
                    attempt += 1;
                    println!(
                        "\n⚠️ [Loop Controller] Verification failed. Injecting diagnostics back into system loop contexts."
                    );

                    // Format actionable repair payload ensuring the AI targets precise source boundaries
                    active_feedback = format!(
                        "Your previous modifications broke compilation bounds. Fix the following build errors:\n\n{}\n\nOriginal user instruction was: {}",
                        diagnostic_error, user_prompt
                    );

                    // If we have repair attempts left, use tactical generation to fetch a corrected plan
                    if attempt < self.max_repair_attempts {
                        active_plan = match self.fetch_live_llm_plan(&active_feedback).await {
                            Ok(repaired_plan) => repaired_plan,
                            Err(e) => {
                                return Err(format!("Self-healing plan generation crashed: {}", e));
                            }
                        };
                    }
                }
            }
        }

        Err(format!(
            "Autonomous repair loop exhausted after {} attempts without successful compilation.",
            self.max_repair_attempts
        ))
    }

    // Performs the remote API call to fetch a structured json refactoring plan matching our exact models
    async fn fetch_live_llm_plan(&self, context_prompt: &str) -> Result<RefactorPlan, String> {
        let client = reqwest::Client::builder().no_proxy().build().unwrap();

        // 1. Dynamically build helper text of our system context capabilities from the registry tools!
        let mut tools_desc = String::new();
        for (name, tool) in &self.registry.tools {
            tools_desc.push_str(&format!(
                "- {}: {}. Schema: {}\n",
                name,
                tool.description(),
                serde_json::to_string(&tool.input_schema()).unwrap_or_default()
            ));
        }

        // 2. Hard system prompt constraints explicitly formatted to match FileEditTask fields perfectly
        let system_instruction = format!(
            "You are an automated software engineering architect running inside a multi-file IDE platform.\n\
            Your task is to take modification requests and return a raw JSON structure matching this precise model schema:\n\n\
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
            CRITICAL CONSTRAINTS:\n\
            - patch_instructions MUST use the exact search/replace patch diff format shown above.\n\
            - target_file MUST be a valid relative path path string to the file.\n\
            - You have access to these lower-level system capabilities if needed to verify your plan: \n{}\n\
            - Return ONLY clean valid JSON. Do not write markdown blocks, explanations, or code fences.",
            tools_desc
        );

        let body = json!({
            "model": self.model_name,
            "messages": [
                { "role": "system", "content": system_instruction },
                { "role": "user", "content": context_prompt }
            ],
            "stream": false,
            "options": { "temperature": 0.7 }
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

        // 1. Build a polyglot system description of our structural editing tool capabilities
        let mut tools_desc = String::new();
        for (name, tool) in &self.registry.tools {
            tools_desc.push_str(&format!("- {}: {}.\n", name, tool.description()));
        }

        // 2. High-level planning prompt instructions optimized for initializing multi-file graphs
        let system_instruction = format!(
            "You are an expert software architect. The workspace environment has already been initialized.\n\
            Your job is to decompose the user's feature request into a logical series of file modifications.\n\
            You must output a single, flat JSON object matching this exact structural schema:\n\n\
            {{\n\
              \"task_graph\": {{\n\
                \"init_source\": {{\n\
                   \"id\": \"init_source\",\n\
                   \"target_file\": \"src/main.rs\",\n\
                   \"patch_instructions\": \"<<<<<<< SEARCH\\n\\n=======\\n// Write complete initialization code here\\n>>>>>>> REPLACE\",\n\
                   \"dependencies\": [],\n\
                   \"status\": \"Pending\"\n\
                }}\n\
              }},\n\
              \"execution_order\": [\"init_source\"]\n\
            }}\n\n\
            POLYGLOT DESIGN RULES:\n\
            - Target files MUST adapt to the project language (e.g., use 'main.go' for Go, 'src/main.rs' for Rust, 'src/index.ts' for TypeScript, or 'app/main.py' for Python).\n\
            - Since you are writing to a newly bootstrapped or existing file, your SEARCH block can be empty if you are adding entirely new content to a file.\n\
            - Ensure `execution_order` lists the task IDs in the exact sequence they should be executed.\n\
            - You have access to these structural capabilities: \n{}\n\
            - CRITICAL: Return ONLY valid, minified JSON. Never wrap your answer in markdown code blocks (```json), code fences, or add conversational explanations.",
            tools_desc
        );

        let body = json!({
            "model": self.model_name,
            "messages": [
                { "role": "system", "content": system_instruction },
                { "role": "user", "content": user_prompt }
            ],
            "stream": false,
            "options": {
                "temperature": 0.2 // Lower temperature for stricter structure adherence during planning
            }
        });

        // 3. Dispatch payload over local HTTP loop to your running Ollama engine
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

        // Extract the raw text out of the Ollama chat response structure
        let raw_content = res_json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        Ok(raw_content)
    }
}
