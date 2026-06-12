use crate::tools::Tool;
use async_trait::async_trait;
use chrono::Local;
use serde_json::{Value, json};

pub struct GetCurrentDateTool;

#[async_trait]
impl Tool for GetCurrentDateTool {
    fn name(&self) -> &'static str {
        "get_current_date"
    }

    fn description(&self) -> &'static str {
        "Returns the current system date, time, and timezone metadata context. Use this whenever the user asks for the current date or time."
    }

    fn input_schema(&self) -> Value {
        // json!({
        //     "type": "object",
        //     "properties": {}
        // })
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "date": { "type": "string" },
                "time": { "type": "string" }
            },
            "required": ["path", "date", "time"]
        })
    }

    async fn execute(&self, _args: &Value) -> Result<String, String> {
        let now = Local::now();
        Ok(format!(
            "Current Timestamp: {}\nDate: {}\nTime: {}\nTimezone: {}",
            now.to_rfc3339(),
            now.format("%Y-%m-%d"),
            now.format("%H:%M:%S"),
            now.format("%Z")
        ))
    }
}
