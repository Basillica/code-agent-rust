use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct MarketDemandValidatorTool;

#[async_trait]
impl Tool for MarketDemandValidatorTool {
    fn name(&self) -> &'static str {
        "market_demand_validator"
    }

    fn description(&self) -> &'static str {
        "Determines whether a problem is painful enough that customers will pay."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type":"object",
            "properties":{
                "idea":{"type":"string"},
                "customer":{"type":"string"}
            },
            "required":["idea","customer"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let idea = args["idea"].as_str().ok_or("missing idea")?;

        let customer = args["customer"].as_str().ok_or("missing customer")?;

        Ok(format!(
            r#"Evaluate market demand.

Idea:
{}

Customer:
{}

Score 0-10:

- Pain Severity
- Frequency
- Urgency
- Existing Spend
- Budget Availability

Return:

{{
  "pain_score":0,
  "frequency_score":0,
  "urgency_score":0,
  "budget_score":0,
  "overall_market_score":0,
  "reasoning":""
}}"#,
            idea, customer
        ))
    }
}
