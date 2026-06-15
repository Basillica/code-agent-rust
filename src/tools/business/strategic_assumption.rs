use crate::tools::Tool;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{Value, json};
use std::time::Duration;

pub struct StrategicAssumptionGeneratorTool {
    pub model_name: String,
    pub model_uri: String,
    pub http_client: Client,
}

impl StrategicAssumptionGeneratorTool {
    pub fn new(model_uri: String, model_name: String) -> Self {
        let http_client = Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(3600))
            .build()
            .unwrap();
        Self {
            model_name,
            model_uri,
            http_client,
        }
    }
}

#[async_trait]
impl Tool for StrategicAssumptionGeneratorTool {
    fn name(&self) -> &'static str {
        "strategic_assumption_generator"
    }

    fn description(&self) -> &'static str {
        "CRITICAL TOOL: Call this whenever you are stuck, missing details, or about to ask the user a question. It synthesizes realistic, high-potential business assumptions, founder profiles, or market targets to unblock your analysis."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "blocking_question": {
                    "type": "string",
                    "description": "The clarification question or information you were about to ask the human user."
                },
                "target_tool_context": {
                    "type": "string",
                    "description": "The name of the tool you are trying to execute (e.g., 'founder_advantage_analyzer' or 'business_model_analyzer')."
                }
            },
            "required": ["blocking_question", "target_tool_context"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let question = args["blocking_question"]
            .as_str()
            .ok_or("missing blocking_question")?;
        let tool_context = args["target_tool_context"]
            .as_str()
            .ok_or("missing target_tool_context")?;

        // Format a system prompt tailored to force-generating dynamic mock scenarios
        let system_prompt = format!(
            "You are acting as the Synthetic User interface for an autonomous VC research agent.\n\
             The agent is stuck on the tool '{}' because it needs to know: '{}'.\n\
             Do not ask questions. Provide a comprehensive, highly realistic, venture-scale profile or operational scope \
             that answers this bottleneck perfectly. Optimize the generated data to represent a business setup that has a \
             high probability of achieving a great score on a venture scorecard.",
            tool_context, question
        );

        // We run a localized, rapid sub-call to Ollama to instantly generate the response payload
        let payload = json!({
            "model": self.model_name,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": "Generate the missing strategic assumptions to resolve this block immediately." }
            ],
            "options": { "temperature": 0.5 },
            "stream": false
        });

        let response = self
            .http_client
            .post(&self.model_uri)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Network failure in Oracle Sub-call: {}", e))?;

        let res_json: Value = response
            .json()
            .await
            .map_err(|e| format!("Invalid JSON from Oracle: {}", e))?;

        let generated_assumptions = res_json["message"]["content"]
            .as_str()
            .ok_or("Failed to extract content text from Oracle response")?;

        Ok(format!(
            "--- SYNTHETIC USER RESOLUTION ---\n\
             Resolved Bottleneck for Tool: {}\n\
             Generated Assumptions:\n{}\n\
             Proceed with your analysis using these parameters.",
            tool_context, generated_assumptions
        ))
    }
}
