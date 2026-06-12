use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct StartupIdeaGeneratorTool;

#[async_trait]
impl Tool for StartupIdeaGeneratorTool {
    fn name(&self) -> &'static str {
        "startup_idea_generator"
    }

    fn description(&self) -> &'static str {
        "Generates startup opportunities aligned with founder strengths."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type":"object",
            "properties":{
                "founder_profile":{"type":"string"},
                "market_preferences":{"type":"string"},
                "idea_count":{"type":"integer"}
            },
            "required":[
                "founder_profile",
                "idea_count"
            ]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let profile = args["founder_profile"]
            .as_str()
            .ok_or("missing founder_profile")?;

        let preferences = args["market_preferences"].as_str().unwrap_or("");

        let count = args["idea_count"].as_i64().unwrap_or(10);

        Ok(format!(
            r#"Generate {} startup ideas.

Founder Profile:
{}

Market Preferences:
{}

Rules:

- Solve expensive problems.
- Target customers with budgets.
- Avoid trivial AI wrappers.
- Prefer B2B.
- Prefer recurring revenue.
- Leverage founder strengths.

Return JSON array:

[
  {{
    "idea_name":"",
    "customer":"",
    "problem":"",
    "solution":"",
    "why_now":"",
    "revenue_model":""
  }}
]"#,
            count, profile, preferences
        ))
    }
}
