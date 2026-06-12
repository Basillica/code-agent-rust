use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct CompetitionAnalyzerTool;

#[async_trait]
impl Tool for CompetitionAnalyzerTool {
    fn name(&self) -> &'static str {
        "competition_analyzer"
    }

    fn description(&self) -> &'static str {
        "Evaluates competitive landscape and differentiation opportunities."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type":"object",
            "properties":{
                "idea":{"type":"string"}
            },
            "required":["idea"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let idea = args["idea"].as_str().ok_or("missing idea")?;

        Ok(format!(
            r#"Analyze competition.

Idea:
{}

Identify:

- Direct competitors
- Indirect competitors
- Substitute solutions
- Saturation level
- Differentiation opportunities

Return JSON:
{{
  "direct_competitors":[],
  "indirect_competitors":[],
  "substitutes":[],
  "market_saturation":0,
  "differentiation_options":[]
}}"#,
            idea
        ))
    }
}
