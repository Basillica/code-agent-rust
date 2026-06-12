use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct DistributionAnalyzerTool;

#[async_trait]
impl Tool for DistributionAnalyzerTool {
    fn name(&self) -> &'static str {
        "distribution_analyzer"
    }

    fn description(&self) -> &'static str {
        "Evaluates customer acquisition and go-to-market risk."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type":"object",
            "properties":{
                "idea":{"type":"string"},
                "founder_profile":{"type":"string"}
            },
            "required":[
                "idea",
                "founder_profile"
            ]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let idea = args["idea"].as_str().ok_or("missing idea")?;

        let profile = args["founder_profile"]
            .as_str()
            .ok_or("missing founder_profile")?;

        Ok(format!(
            r#"Analyze distribution.

Idea:
{}

Founder:
{}

Return:

{{
  "distribution_channels":[],
  "cac_risk":0,
  "organic_growth_potential":0,
  "distribution_advantage":0,
  "go_to_market_recommendation":""
}}"#,
            idea, profile
        ))
    }
}
