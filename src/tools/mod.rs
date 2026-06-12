use async_trait::async_trait;
use serde_json::Value;

pub mod bash;
pub mod bootstrap;
pub mod browser;
pub mod business;
pub mod calculator;
pub mod code_chain;
pub mod diagonistic;
pub mod edit;
pub mod edit_file;
pub mod glob;
pub mod grep;
pub mod read_file;
pub mod registry;
pub mod search;
pub mod shell;
pub mod task;
pub mod utils;
pub mod weather;
pub mod write_file;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, args: &Value) -> Result<String, String>;
}

pub struct SafetyRule {
    pub pattern: &'static str,
    pub reason: &'static str,
}
