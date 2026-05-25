use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;

pub struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &'static str {
        "get_current_weather"
    }

    fn description(&self) -> &'static str {
        "Fetches real-time weather details and current conditions for a specified geographical location or city."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city name or target location (e.g., 'Munich', 'Paris,FR')"
                }
            },
            "required": ["location"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        // 1. Extract and validate parameters matching your input_schema layout
        let location = args["location"]
            .as_str()
            .ok_or("Missing parameter 'location'")?;

        if location.trim().is_empty() {
            return Err("Error: Parameter 'location' cannot be empty.".to_string());
        }

        // 2. URL-encode the location to handle multi-word city names gracefully (e.g., "New York")
        let encoded_location = urlencoding::encode(location);
        let url = format!("https://wttr.in/{}?format=3", encoded_location);

        // 3. Dispatch the outward HTTP request using reqwest with standard safety timeouts
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

                    // Return the clean textual weather forecast format (e.g., "Munich: 🌤️ +14°C")
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
