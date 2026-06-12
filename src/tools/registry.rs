use crate::tools::bash::BashTool;
use crate::tools::bootstrap::BootstrapProjectTool;
use crate::tools::browser::WebBrowserTool;
use crate::tools::business;
use crate::tools::calculator::CalculatorTool;
use crate::tools::diagonistic::CheckDiagnosticsTool;
use crate::tools::edit::SurgicalEditTool;
use crate::tools::edit_file::EditFileTool;
use crate::tools::glob::GlobTool;
use crate::tools::grep::GrepTool;
use crate::tools::read_file::ReadFileTool;
use crate::tools::search::CodebaseSearchTool;
use crate::tools::shell::ShellTool;
use crate::tools::task::DelegateSubtaskTool;
use crate::tools::weather::WeatherTool;
use crate::tools::write_file::WriteFileTool;
use crate::tools::{Tool, code_chain};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ToolRegistry {
    pub tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new(project_root: &PathBuf) -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        registry.register(ReadFileTool);
        registry.register(WriteFileTool);
        registry.register(SurgicalEditTool);
        registry.register(DelegateSubtaskTool);
        registry.register(ShellTool);
        registry.register(BashTool);
        registry.register(GlobTool);
        registry.register(GrepTool);
        registry.register(EditFileTool);
        registry.register(CheckDiagnosticsTool);
        registry.register(WebBrowserTool::new());
        registry.register(CodebaseSearchTool::new(project_root.to_path_buf()));
        registry.register(CalculatorTool::new());
        registry.register(WeatherTool);
        registry.register(code_chain::CodeGenerationChainTool::new());
        registry.register(BootstrapProjectTool::new(project_root.to_path_buf()));
        registry.register(business::business_model_analyzer::BusinessModelAnalyzerTool);
        registry.register(business::competition_analyzer::CompetitionAnalyzerTool);
        registry.register(business::distribution_analyzer::DistributionAnalyzerTool);
        registry.register(business::founder_analyzer::FounderAdvantageAnalyzerTool);
        registry.register(business::market_demand_validator::MarketDemandValidatorTool);
        registry.register(business::startup_idea_generator::StartupIdeaGeneratorTool);
        registry.register(business::technical_moat_editor::TechnicalMoatAuditorTool);
        registry.register(business::venture_score_card::VentureScorecardTool);

        registry
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }
}
