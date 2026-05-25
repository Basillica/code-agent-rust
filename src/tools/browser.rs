use crate::tools::Tool;
use async_trait::async_trait;
use mdka::html_to_markdown;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct WebFetchArgs {
    pub url: String,
    pub extract_links: Option<bool>,
}

pub struct WebBrowserTool {
    client: Client,
}

impl WebBrowserTool {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Autonomous-Engineering-Agent/2026)")
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

#[async_trait]
impl Tool for WebBrowserTool {
    fn name(&self) -> &'static str {
        "web_fetch"
    }

    fn description(&self) -> &'static str {
        "Fetch raw content from any public web address or technical documentation endpoint, converting it to readable Markdown."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The exact URL target to download" },
                "extract_links": { "type": "boolean", "description": "Set true to dump an index of target anchors found" }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, arguments: &serde_json::Value) -> Result<String, String> {
        let args: WebFetchArgs = serde_json::from_value(arguments.clone())
            .map_err(|e| format!("Invalid browser arguments: {}", e))?;

        println!(
            "🌐 [Web Subsystem]: Requesting content matrix from -> {}",
            args.url
        );

        let response = self
            .client
            .get(&args.url)
            .send()
            .await
            .map_err(|e| format!("Web request connection failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("Server returned failing status code: {}", status));
        }

        let raw_html = response
            .text()
            .await
            .map_err(|e| format!("Failed to extract body payload stream: {}", e))?;

        // Convert the structural raw HTML into scannable, lightweight Markdown text
        let markdown_body = html_to_markdown(&raw_html);

        // Limit context size defensively to prevent local model context context-window overflows
        let cropped_text: String = markdown_body.chars().take(8000).collect();

        Ok(format!(
            "### [WEB CONTEXT ACQUIRED FROM {}]\nStatus Code: {}\n\n{}",
            args.url, status, cropped_text
        ))
    }
}
