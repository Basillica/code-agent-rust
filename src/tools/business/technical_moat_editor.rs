use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct TechnicalMoatAuditorTool;

#[async_trait]
impl Tool for TechnicalMoatAuditorTool {
    fn name(&self) -> &'static str {
        "technical_moat_auditor"
    }

    fn description(&self) -> &'static str {
        "Determines whether technology creates a defensible advantage."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type":"object",
            "properties":{
                "idea":{"type":"string"},
                "architecture":{"type":"string"},
                "founder_stack":{"type":"string"}
            },
            "required":[
                "idea",
                "architecture",
                "founder_stack"
            ]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let idea = args["idea"].as_str().ok_or("missing idea")?;

        let architecture = args["architecture"]
            .as_str()
            .ok_or("missing architecture")?;

        let stack = args["founder_stack"]
            .as_str()
            .ok_or("missing founder_stack")?;

        Ok(format!(
            r#"Act as a skeptical principal engineer.

Idea:
{}

Architecture:
{}

Founder Stack:
{}

Analyze:

- Technical difficulty.
- OpenAI replacement risk.
- Anthropic replacement risk.
- Cursor replacement risk.
- GitHub replacement risk.
- AWS replacement risk.
- Infrastructure moat.
- Data moat.
- Switching costs.

Return JSON:

{{
  "technical_difficulty":0,
  "wrapper_risk":0,
  "platform_dependency":0,
  "infrastructure_moat":0,
  "data_moat":0,
  "switching_costs":0,
  "overall_defensibility":0,
  "critical_risks":[]
}}"#,
            idea, architecture, stack
        ))
    }
}
