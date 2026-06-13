use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct FounderAdvantageAnalyzerTool;

#[async_trait]
impl Tool for FounderAdvantageAnalyzerTool {
    fn name(&self) -> &'static str {
        "founder_advantage_analyzer"
    }

    fn description(&self) -> &'static str {
        "Identifies founder unfair advantages, weaknesses, and markets where they are most likely to win."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "skills": {
                    "type": "string"
                },
                "experience": {
                    "type": "string"
                },
                "interests": {
                    "type": "string"
                },
                "network": {
                    "type": "string"
                }
            },
            "required": [
                "skills",
                "experience",
                "interests",
                "network"
            ]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let skills = args["skills"].as_str().ok_or("missing skills")?;
        let experience = args["experience"].as_str().ok_or("missing experience")?;
        let interests = args["interests"].as_str().ok_or("missing interests")?;
        let network = args["network"].as_str().ok_or("missing network")?;

        Ok(format!(
            r#"You are a venture partner evaluating founder-market fit.

            Founder Profile:

            Skills:
            {}

            Experience:
            {}

            Interests:
            {}

            Network:
            {}

            Analyze:

            1. Unfair advantages.
            2. Weaknesses.
            3. Markets where this founder has leverage.
            4. Markets where this founder should avoid competing.
            5. Areas where technical expertise creates a moat.
            6. Areas where domain expertise is missing.

            Return JSON:

            {{
            "advantage_areas": [],
            "weakness_areas": [],
            "recommended_markets": [],
            "anti_markets": [],
            "founder_fit_summary": ""
            }}"#,
            skills, experience, interests, network
        ))
    }
}
