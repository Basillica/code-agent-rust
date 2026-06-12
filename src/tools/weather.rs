use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;

pub struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &'static str {
        "get_weather"
    }

    fn description(&self) -> &'static str {
        "Fetches real-time weather details and current conditions. If no location is provided, it automatically infers the current local area via the system's public IP address."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "Optional city name or target destination (e.g., 'Munich', 'Paris,FR'). Leave blank or omit to look up the local system weather."
                }
            }
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        // 1. Extract optional parameter safely without throwing an error if missing
        let location = args.get("location").and_then(|l| l.as_str()).unwrap_or("");

        // 2. Build target URL depending on whether a location was specified
        let url = if location.trim().is_empty() {
            "https://wttr.in/?format=3".to_string()
        } else {
            let encoded_location = urlencoding::encode(location);
            format!("https://wttr.in/{}?format=3", encoded_location)
        };

        // 3. Dispatch the outward HTTP request
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to build network client configuration: {}", e))?;

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let weather_report = response
                        .text()
                        .await
                        .map_err(|e| format!("Failed to read upstream data text stream: {}", e))?;

                    Ok(format!(
                        "[Weather Observation Result]: {}",
                        weather_report.trim()
                    ))
                } else {
                    Err(format!(
                        "Upstream weather API returned an error status code: {}",
                        response.status()
                    ))
                }
            }
            Err(e) => Err(format!(
                "Network dispatch execution failure contacting external weather service: {}",
                e
            )),
        }
    }
}
