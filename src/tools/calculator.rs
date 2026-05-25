use crate::tools::Tool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CalculatorArgs {
    pub expression: String,
}

pub struct CalculatorTool;

impl CalculatorTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &'static str {
        "calculator"
    }

    fn description(&self) -> &'static str {
        "Evaluate mathematical and trigonometric expressions safely (e.g., '2 + 2', 'sin(pi / 2)', 'cos(1.57)')."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "The mathematical expression to evaluate, including basic operators and trig functions like sin, cos, tan."
                }
            },
            "required": ["expression"]
        })
    }

    async fn execute(&self, arguments: &serde_json::Value) -> Result<String, String> {
        let args: CalculatorArgs = serde_json::from_value(arguments.clone())
            .map_err(|e| format!("Invalid calculator arguments: {}", e))?;

        // Lowercase the input to make string matching uniform
        let normalized = args.expression.to_lowercase();

        // 1. First, enforce strict character validation (letters, numbers, basic operators, parens)
        if normalized.chars().any(|c| {
            !c.is_digit(10)
                && !c.is_alphabetic()
                && !['+', '-', '*', '/', '(', ')', '.', ' '].contains(&c)
        }) {
            return Err(
                "Security Violation: Unsafe characters detected in expression.".to_string(),
            );
        }

        // 2. Extract words (sequences of alphabetic characters) and ensure they match ONLY safe math operators/constants
        let words: Vec<&str> = normalized
            .split(|c: char| !c.is_alphabetic())
            .filter(|s| !s.is_empty())
            .collect();

        let allowed_tokens = ["sin", "cos", "tan", "sqrt", "pi", "e", "ln", "log", "abs"];
        for word in words {
            if !allowed_tokens.contains(&word) {
                return Err(format!(
                    "Security Violation: Unauthorized token or function call '{}' detected.",
                    word
                ));
            }
        }

        println!(
            "🔢 [Math Subsystem]: Evaluating expression -> {}",
            normalized
        );

        // meval inherently parses functions like sin, cos, tan, and the constant pi
        match meval::eval_str(&normalized) {
            Ok(result) => Ok(format!("Calculation Result: {}", result)),
            Err(err) => Err(format!(
                "Failed to evaluate mathematical expression: {}",
                err
            )),
        }
    }
}
