use crate::action::permissions::PermissionMode;
use crate::orchestrator::strategy::{CoreEngineRunner, ExecutionStrategy};
use clap::{Parser, ValueEnum};
use std::env;
use std::path::PathBuf;

mod action;
mod core;
mod intelligence;
mod orchestrator;
mod state;
mod terminal;
mod tools;
mod ui;
mod verification;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
enum CliStrategy {
    /// Claude Code style tool-by-tool autonomous discovery loop
    Autonomous,
    /// Upfront pre-planned topological dependency graph execution
    Upfront,
}

// #[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
// enum CliPermission {
//     /// Gate risky modifications behind interactive console prompts
//     Gated,
//     /// Authorize full autonomous background execution
//     Auto,
// }

#[derive(Parser, Debug)]
#[command(name = "bolt-code")]
#[command(about = "Multi-Paradigm Autonomous AI Coding Engine", long_about = None)]
struct CliArgs {
    /// The task description or refactoring request for the AI engine
    #[arg(short, long)]
    prompt: String, // Keeps '-p' and '--prompt'

    /// Target project root directory workspace path
    #[arg(short, long, default_value = ".")]
    workspace: PathBuf, // Keeps '-w' and '--workspace'

    /// Execution engine strategy paradigm choice
    #[arg(short, long, value_enum, default_value_t = CliStrategy::Autonomous)]
    strategy: CliStrategy, // Keeps '-s' and '--strategy'

    /// Permission boundaries applied to tool executions
    #[arg(long, value_enum, default_value_t = PermissionMode::DontAsk)]
    permission: PermissionMode, // Changed to '--permission' only (no short flag)

    /// Compilation or testing command utilized for self-healing verification loops
    #[arg(short, long, default_value = "cargo check")]
    compile_cmd: String, // Keeps '-c' and '--compile-cmd'

    ///
    #[arg(
        long,
        help = "Optional test suite validation command (e.g., 'cargo test')"
    )]
    pub test_cmd: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parse operational terminal flags
    let args = CliArgs::parse();

    // 2. Safely capture API credentials from the platform environment
    let api_key = "".to_string();
    // match env::var("AI_ENGINE_API_KEY") {
    //     Ok(key) => key,
    //     Err(_) => {
    //         eprintln!("❌ Error: 'AI_ENGINE_API_KEY' environment variable is missing.");
    //         std::process::exit(1);
    //     }
    // };

    // 3. Normalize paths to locate target repository roots accurately
    let target_path = if args.workspace.is_relative() {
        env::current_dir()?.join(args.workspace)
    } else {
        args.workspace
    };
    let canonical_workspace = std::fs::canonicalize(target_path)?;
    println!("📂 Workspace locked on: {:?}", canonical_workspace);

    // 4. Map CLI configuration space directly to inner strategy variants
    let selected_strategy = match args.strategy {
        CliStrategy::Autonomous => ExecutionStrategy::AutonomousAgent,
        CliStrategy::Upfront => ExecutionStrategy::UpfrontGraphPlan,
    };

    // 5. Instantiate Core Engine Runner
    let runner = CoreEngineRunner::new(
        canonical_workspace,
        api_key,
        args.compile_cmd,
        args.test_cmd,
        "gemma4:e4b".to_string(),
        "http://127.0.0.1:11434/api/chat".to_string(),
    );

    println!("🚀 Launching operational strategy dispatcher...");

    // 6. Handoff context directly into your multi-paradigm orchestrator
    match runner
        .dispatch_request(&args.prompt, selected_strategy, args.permission)
        .await
    {
        Ok(_) => {
            println!("✨ System Orchestrator: Execution completed successfully.");
            Ok(())
        }
        Err(err) => {
            eprintln!("❌ Critical Runtime Error: {}", err);
            std::process::exit(1);
        }
    }
}
