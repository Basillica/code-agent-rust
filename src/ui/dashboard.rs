use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct AgentDashboard {
    spinner: ProgressBar,
}

impl AgentDashboard {
    pub fn new() -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{prefix:.bold.dim} {spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_prefix("[Engine]");
        Self { spinner: pb }
    }

    pub fn start_task(&self, task_description: &str) {
        println!("\n🚀 [Agent Objective]: {}", task_description);
        self.spinner
            .set_message("Initializing local reasoning loop frames...");
        self.spinner.enable_steady_tick(Duration::from_millis(80));
    }

    pub fn update_status(&self, tool_name: &str, detail: &str) {
        let truncated_detail = if detail.len() > 60 {
            format!("{}...", &detail[0..57])
        } else {
            detail.to_string()
        };
        self.spinner.set_message(format!(
            "🔧 Executing [{}] -> {}",
            tool_name, truncated_detail
        ));
    }

    pub fn log_verification(&self, message: &str) {
        self.spinner.println(format!("✨ {}", message));
    }

    pub fn pause_for_input(&self) {
        self.spinner.finish_and_clear();
    }

    pub fn resume_after_input(&mut self) {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{prefix:.bold.dim} {spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_prefix("[Engine]");
        pb.enable_steady_tick(Duration::from_millis(80));
        self.spinner = pb;
    }

    pub fn fail_task(&self, reason: &str) {
        self.spinner
            .abandon_with_message(format!("❌ Agent encountered an issue: {}", reason));
    }

    pub fn complete_task(&self) {
        self.spinner.finish_with_message(
            "🏆 Objective achieved successfully! Codebase synchronization ready.",
        );
    }
}
