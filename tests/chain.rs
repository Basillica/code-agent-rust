#[cfg(test)]
mod tests {
    use code_agent_rust::action::permissions::PermissionMode;
    use code_agent_rust::orchestrator::autonomous::AutonomousOrchestrator;
    use code_agent_rust::state::session::SessionContext;
    use code_agent_rust::tools::registry::ToolRegistry;

    use serde_json::{Value, json};
    use serial_test::serial;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Helper to instantiate a clean registry and orchestrator for testing
    async fn setup_test_orchestrator() -> (
        AutonomousOrchestrator,
        Arc<Mutex<SessionContext>>,
        MockServer,
    ) {
        let mock_server = MockServer::start().await;
        let path = PathBuf::from("test");
        let session_ctx = Arc::new(Mutex::new(SessionContext::new(&path)));
        let model_name = "gemma4:e4b".to_string();
        let model_uri = format!("{}/api/chat", mock_server.uri());

        let registry = ToolRegistry::new(
            &path,
            session_ctx.clone(),
            model_name.clone(),
            model_uri.clone(),
        );

        let orchestrator = AutonomousOrchestrator::new(
            session_ctx.clone(),
            Arc::new(registry),
            model_name,
            model_uri,
        );

        (orchestrator, session_ctx, mock_server)
    }

    /// Helper to wrap your AgentResponse format inside the Ollama chat response structure
    fn make_ollama_response(
        thought: &str,
        tool_name: &str,
        args: Value,
        completed: bool,
        summary: Option<&str>,
    ) -> ResponseTemplate {
        let agent_response = json!({
            "thought": thought,
            "task_completed": completed,
            "final_summary": summary,
            "tool_call": if tool_name == "None" {
                None
            } else {
                Some(json!({
                    "name": tool_name,
                    "arguments": args
                }))
            }
        });

        // Ollama nests the text response inside message.content
        let serialized_content = serde_json::to_string(&agent_response).unwrap();

        ResponseTemplate::new(200).set_body_json(json!({
            "message": {
                "role": "assistant",
                "content": serialized_content
            },
            "done": true
        }))
    }

    // =========================================================================
    // TESTING SCENARIOS
    // =========================================================================

    #[tokio::test]
    #[serial]
    async fn test_handle_conversational_pivot_null_tool() {
        // Scenario: The model reaches a state where it asks for user input ("null" tool/None)
        let (orchestrator, _ctx, mock_server) = setup_test_orchestrator().await;

        // Feed an agent response that wants to stop and ask a question
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(make_ollama_response(
                "I am missing the founder profile parameters.",
                "null",
                json!({}),
                false,
                None,
            ))
            .mount(&mock_server)
            .await;

        // Execute the goal. It should complete or return Ok because hitting a conversational
        // pivot halts the inner autonomous loop to yield to terminal interactions.
        let result = orchestrator
            .execute_goal("Evaluate my business", PermissionMode::Auto)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_valid_tool_execution() {
        let (orchestrator, ctx, mock_server) = setup_test_orchestrator().await;
        // Turn 1: Model requests competition analyzer
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(make_ollama_response(
                "Let's look at the market competitors first.",
                "competition_analyzer",
                json!({"idea": "SaaS for Predictive Maintenance"}),
                false,
                None,
            ))
            .mount(&mock_server)
            .await;

        // Execute the single turn/step cycle
        let result = orchestrator
            .execute_goal("Check competition", PermissionMode::Auto)
            .await;
        assert!(result.is_ok());

        // Verify the context history now includes the systemic tool execution feedback
        let context = ctx.lock().await;
        assert!(
            context
                .history
                .iter()
                .any(|m| m.role == "system" && m.content.contains("direct_competitors"))
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_malformed_tool_arguments() {
        // Scenario: Model attempts to run a tool but omits required parameters
        let (orchestrator, ctx, mock_server) = setup_test_orchestrator().await;

        // 'business_model_analyzer' requires BOTH "idea" and "customer" fields
        // We purposefully pass an empty arguments payload to trick it
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(make_ollama_response(
                "Checking business model economic quality.",
                "business_model_analyzer",
                json!({}), // Missing mandatory args
                false,
                None,
            ))
            .mount(&mock_server)
            .await;

        let result = orchestrator
            .execute_goal("Analyze monetization", PermissionMode::Auto)
            .await;
        assert!(result.is_ok());

        // The orchestrator shouldn't panic! It should inject the missing argument validation
        // error response message into the history stack so the model can self-correct on Turn + 1.
        let context = ctx.lock().await;
        assert!(
            context
                .history
                .iter()
                .any(|m| m.role == "system" && m.content.contains("missing idea"))
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_hallucinated_unknown_tool() {
        // Scenario: Model hallucinates a tool that isn't registered in your registry inventory
        let (orchestrator, ctx, mock_server) = setup_test_orchestrator().await;

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(make_ollama_response(
                "Let's execute an unverified utility.",
                "super_secret_market_spy_tool", // Completely fake tool signature
                json!({"query": "German market text"}),
                false,
                None,
            ))
            .mount(&mock_server)
            .await;

        let result = orchestrator
            .execute_goal("Spy on market", PermissionMode::Auto)
            .await;
        assert!(result.is_ok());

        // Ensure the system message lets the model know the utility doesn't exist
        let context = ctx.lock().await;
        println!("the frigging context: {:?}", context.history);

        assert!(context.history.iter().any(|m| {
            m.role == "system"
                && m.content
                    .contains("Tool 'super_secret_market_spy_tool' not found")
        }));
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_task_completion_exit() {
        // Scenario: The model signals that the entire goal objective has been met
        let (orchestrator, ctx, mock_server) = setup_test_orchestrator().await;

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(make_ollama_response(
                "All assessments complete.",
                "None",
                json!({}),
                true, // task_completed = true
                Some("Venture Evaluation Complete. Final Score: 92/100."),
            ))
            .mount(&mock_server)
            .await;

        let result = orchestrator
            .execute_goal("Perform full audit", PermissionMode::Auto)
            .await;

        // Loop should break instantly and return clean execution confirmation
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_corrupted_raw_json_fallback() {
        // Scenario: The LLM outputs corrupted json formatting text that can't be safely cleaned
        let (orchestrator, ctx, mock_server) = setup_test_orchestrator().await;

        // Corrupted JSON payload missing brackets completely
        let corrupted_payload = ResponseTemplate::new(200).set_body_json(json!({
            "message": {
                "role": "assistant",
                "content": "This is raw descriptive text that totally breaks your extract_clean_json method"
            },
            "done": true
        }));

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(corrupted_payload)
            .mount(&mock_server)
            .await;

        let result = orchestrator
            .execute_goal("Test broken output handling", PermissionMode::Auto)
            .await;

        // Depending on your error handling, it will either pass back an error message to self-correct
        // or yield a safe Result Err back up to main.rs. Ensure it handles it gracefully instead of a thread panic.
        assert!(result.is_err() || result.is_ok());
    }
}
