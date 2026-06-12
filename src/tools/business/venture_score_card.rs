use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct VentureScorecardTool;

#[async_trait]
impl Tool for VentureScorecardTool {
    fn name(&self) -> &'static str {
        "venture_scorecard"
    }

    fn description(&self) -> &'static str {
        "Produces a final venture-quality assessment from prior analyses."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type":"object",
            "properties":{
                "market_analysis":{"type":"string"},
                "competition_analysis":{"type":"string"},
                "technical_analysis":{"type":"string"},
                "distribution_analysis":{"type":"string"},
                "founder_fit_analysis":{"type":"string"},
                "business_model_analysis":{"type":"string"}
            },
            "required":[
                "market_analysis",
                "competition_analysis",
                "technical_analysis",
                "distribution_analysis",
                "founder_fit_analysis",
                "business_model_analysis"
            ]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        Ok(format!(
            r#"Act as an investment committee.

Inputs:

{}

Weight:

Market: 30%
Founder: 20%
Distribution: 20%
Business Model: 15%
Technical Moat: 10%
Competition: 5%

Return:

{{
  "market":0,
  "founder_fit":0,
  "distribution":0,
  "business_model":0,
  "technical_moat":0,
  "competition":0,
  "overall":0,
  "recommendation":"",
  "top_strengths":[],
  "top_risks":[]
}}"#,
            serde_json::to_string_pretty(args).unwrap()
        ))
    }
}
