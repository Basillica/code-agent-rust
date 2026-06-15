#[derive(Debug, Clone)]
pub struct ModelCostConfig {
    pub name: String,
    pub input_cost_per_million: f64,
    pub output_cost_per_million: f64,
    pub max_context_tokens: Option<usize>, // Crucial for preventing local model context overflows
}

impl ModelCostConfig {
    /// Returns a fresh builder instance to construct a configuration dynamically
    pub fn builder() -> ModelCostConfigBuilder {
        ModelCostConfigBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct ModelCostConfigBuilder {
    name: Option<String>,
    input_cost_per_million: Option<f64>,
    output_cost_per_million: Option<f64>,
    max_context_tokens: Option<usize>,
}

impl ModelCostConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the financial cost incurred per 1 million input tokens consumed
    pub fn input_cost_per_million(mut self, cost: f64) -> Self {
        self.input_cost_per_million = Some(cost);
        self
    }

    /// Sets the financial cost incurred per 1 million generated output tokens
    pub fn output_cost_per_million(mut self, cost: f64) -> Self {
        self.output_cost_per_million = Some(cost);
        self
    }

    /// Sets the context window boundary ceiling capacity limit (optional)
    pub fn max_context_tokens(mut self, token_limit: usize) -> Self {
        self.max_context_tokens = Some(token_limit);
        self
    }

    /// Finalizes and validates the builder configuration matrix
    pub fn build(self) -> Result<ModelCostConfig, String> {
        let name = self
            .name
            .ok_or_else(|| "Model identification name must be provided to builder".to_string())?;

        // Local models default to free ($0.0) if no pricing structure is explicitly provided
        let input_cost_per_million = self.input_cost_per_million.unwrap_or(0.0);
        let output_cost_per_million = self.output_cost_per_million.unwrap_or(0.0);

        Ok(ModelCostConfig {
            name,
            input_cost_per_million,
            output_cost_per_million,
            max_context_tokens: self.max_context_tokens,
        })
    }
}
