use crate::orchestrator::config::ModelCostConfig;

pub struct TokenBudgetGuardrail {
    config: ModelCostConfig,
    max_allowed_cost_usd: f64,
    max_allowed_tokens: usize,

    // Tracked usage statistics
    cumulative_input_tokens: usize,
    cumulative_output_tokens: usize,
}

impl TokenBudgetGuardrail {
    /// Creates a new budget guardrail with predefined hard limits
    pub fn new(
        config: ModelCostConfig,
        max_allowed_cost_usd: f64,
        max_allowed_tokens: usize,
    ) -> Self {
        Self {
            config,
            max_allowed_cost_usd,
            max_allowed_tokens,
            cumulative_input_tokens: 0,
            cumulative_output_tokens: 0,
        }
    }

    /// Records a transaction pass and calculates real-time consumption values
    pub fn record_usage(&mut self, input_tokens: usize, output_tokens: usize) {
        self.cumulative_input_tokens += input_tokens;
        self.cumulative_output_tokens += output_tokens;
    }

    /// Computes the current financial cost accumulated across operations
    pub fn current_cost_usd(&self) -> f64 {
        let input_cost = (self.cumulative_input_tokens as f64 / 1_000_000.0)
            * self.config.input_cost_per_million;
        let output_cost = (self.cumulative_output_tokens as f64 / 1_000_000.0)
            * self.config.output_cost_per_million;
        input_cost + output_cost
    }

    /// Returns total tokens processed so far
    pub fn total_tokens_used(&self) -> usize {
        self.cumulative_input_tokens + self.cumulative_output_tokens
    }

    /// Evaluates if execution remains within acceptable safety boundaries.
    /// Returns `Ok(())` if safe, or an error string specifying the safety breach.
    pub fn check_budget_safety(&self) -> Result<(), String> {
        let active_cost = self.current_cost_usd();
        let active_tokens = self.total_tokens_used();

        if active_cost >= self.max_allowed_cost_usd {
            return Err(format!(
                "🚫 [BUDGET BREACH] Operational cost limit reached! Spent: ${:.4} / Max allowed: ${:.2}",
                active_cost, self.max_allowed_cost_usd
            ));
        }

        if active_tokens >= self.max_allowed_tokens {
            return Err(format!(
                "🚫 [TOKEN BREACH] Cumulative volume safety cap exceeded! Used: {} tokens / Max limit: {}",
                active_tokens, self.max_allowed_tokens
            ));
        }

        Ok(())
    }

    pub fn reset_token(&mut self) {
        self.cumulative_input_tokens = 0;
        self.cumulative_output_tokens = 0;
    }

    /// Prints a summary status report to the terminal interface
    pub fn print_telemetry_report(&self) {
        println!(
            "📊 [Telemetry] Tokens: In={} Out={} | Total Spend: ${:.4} / ${:.2}",
            self.cumulative_input_tokens,
            self.cumulative_output_tokens,
            self.current_cost_usd(),
            self.max_allowed_cost_usd
        );
    }
}
