// use crate::tools::Tool;
// use async_trait::async_trait;
// use serde_json::Value;

// pub struct LocationTool;

// #[async_trait]
// impl Tool for LocationTool {
//     fn name(&self) -> &'static str {
//         "get_current_location"
//     }

//     fn description(&self) -> &'static str {
//         "Resolves the host machine's current physical/geographical city and country details using its public IP address routing signatures."
//     }

//     fn input_schema(&self) -> Value {
//         // Expects no parameters
//         serde_json::json!({
//             "type": "object",
//             "properties": {}
//         })
//     }

//     async fn execute(&self, _args: &Value) -> Result<String, String> {
//         let client = reqwest::Client::builder()
//             .timeout(std::time::Duration::from_secs(4))
//             .build()
//             .map_err(|e| e.to_string())?;

//         // Query a reliable, free geolocation endpoint
//         match client.get("http://ip-api.com/json/").send().await {
//             Ok(res) => {
//                 if res.status().is_success() {
//                     let json_data: Value = res.json().await.map_err(|e| e.to_string())?;

//                     let city = json_data["city"].as_str().unwrap_or("Unknown City");
//                     let country = json_data["countryCode"].as_str().unwrap_or("US");

//                     Ok(format!("[Location Found]: {}, {}", city, country))
//                 } else {
//                     Err(format!(
//                         "Geographic provider returned status: {}",
//                         res.status()
//                     ))
//                 }
//             }
//             Err(e) => Err(format!("Failed to determine system position: {}", e)),
//         }
//     }
// }

use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;

pub struct LocationTool;

#[async_trait]
impl Tool for LocationTool {
    fn name(&self) -> &'static str {
        "get_location"
    }

    fn description(&self) -> &'static str {
        "Resolves geographical details (city, country, ISP) for a specific target IP address or domain name. If the target is omitted, it resolves the location of the host machine."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Optional IP address or domain name to look up (e.g., '8.8.8.8', 'github.com'). Leave blank or omit to look up the local system's environment location."
                }
            }
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        // 1. Extract the optional target parameter safely
        let target = args.get("target").and_then(|t| t.as_str()).unwrap_or("");

        // 2. Build the appropriate endpoint URL
        let url = if target.trim().is_empty() {
            "http://ip-api.com/json/".to_string()
        } else {
            // URL-encode the target to cleanly handle domains or unexpected spacing
            let encoded_target = urlencoding::encode(target.trim());
            format!("http://ip-api.com/json/{}", encoded_target)
        };

        // 3. Dispatch the HTTP client request
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to build network configuration: {}", e))?;

        match client.get(&url).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    let json_data: Value = res
                        .json()
                        .await
                        .map_err(|e| format!("Failed to parse upstream payload: {}", e))?;

                    // 4. Check ip-api internal response status (handles invalid lookups gracefully)
                    if json_data["status"].as_str() == Some("fail") {
                        let error_msg = json_data["message"]
                            .as_str()
                            .unwrap_or("Unknown query routing failure");
                        return Err(format!(
                            "Location lookup failed for target '{}': {}",
                            target, error_msg
                        ));
                    }

                    // 5. Extract useful metadata out of the successful response matrix
                    let resolved_ip = json_data["query"].as_str().unwrap_or("Unknown IP");
                    let city = json_data["city"].as_str().unwrap_or("Unknown City");
                    let region = json_data["regionName"].as_str().unwrap_or("Unknown Region");
                    let country = json_data["country"].as_str().unwrap_or("Unknown Country");
                    let isp = json_data["isp"].as_str().unwrap_or("Unknown ISP");

                    Ok(format!(
                        "[Location Target Resolved - {}]: {}, {} in {} (ISP: {})",
                        resolved_ip, city, region, country, isp
                    ))
                } else {
                    Err(format!(
                        "Geographic provider returned an HTTP error code: {}",
                        res.status()
                    ))
                }
            }
            Err(e) => Err(format!(
                "Network dispatch failure checking system position: {}",
                e
            )),
        }
    }
}
