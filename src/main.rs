// mod action;
// mod core;
// mod orchestrator;
// mod state;
// mod tools;
// mod ui;
// mod verification;

// use crate::core::main_loop::query_loop;
// use crate::orchestrator::ui::TerminalUI;
// use action::permissions::PermissionMode;
// use clap::Parser;
// use state::session::SessionContext;
// use std::io;
// use std::io::Write;
// use std::path::PathBuf;
// use tools::registry::ToolRegistry;

// #[derive(Parser, Debug)]
// #[command(
//     name = "agent-code",
//     // description = "Autonomous local developer agent harness in Rust"
// )]
// struct Args {
//     #[arg(short, long, default_value = "dontAsk")]
//     mode: String,

//     #[arg(short, long)]
//     prompt: Option<String>,
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     second().await
// }

// #[derive(Parser, Debug)]
// #[command(name = "agent-code")]
// #[command(
//     about = "Autonomous local developer agent harness powered by local LLMs via Ollama",
//     version = "1.0.0"
// )]
// struct CliArgs {
//     /// Graduated security permissions mode: default, acceptEdits, dontAsk
//     #[arg(short, long, default_value = "dontAsk")]
//     mode: String,

//     /// Direct coding directive to process immediately without opening interactive shell
//     #[arg(short, long)]
//     prompt: Option<String>,
// }

// async fn interactive_prompt_loop(
//     session: &mut SessionContext,
//     mode: PermissionMode,
//     registry: &ToolRegistry,
// ) {
//     loop {
//         print!("\n🤖 [Agent CLI] Enter coding goal or task (or type \"exit\"): ");
//         let _ = io::stdout().flush();

//         let mut buffer = String::new();
//         if io::stdin().read_line(&mut buffer).is_err() {
//             println!("Error capturing line input stream entries.");
//             break;
//         }

//         let target = buffer.trim();
//         if target.to_lowercase() == "exit" {
//             println!("👋 Operational runtime detached cleanly. Session saved.");
//             break;
//         }

//         if !target.is_empty() {
//             let _ = query_loop(target, session, mode, &registry, &mut TerminalUI::new()).await;
//         }
//     }
// }

// async fn second() -> Result<(), Box<dyn std::error::Error>> {
//     println!("=====================================================================");
//     println!("⚡ AGENTIC RUNTIME ENGINE ACTIVATED — LOCAL WORKSPACE HARNESS ONLINE ⚡");
//     let args = CliArgs::parse();
//     println!(
//         "🔒 Security Trust Level Profile Configured As: [{}]",
//         args.mode.to_uppercase()
//     );
//     println!("=====================================================================");

//     let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
//     let mut session = SessionContext::new(current_dir);
//     let mode = PermissionMode::from_str(&args.mode);
//     let registry = ToolRegistry::new(&session.project_root);

//     if let Some(immediate_prompt) = args.prompt {
//         println!(
//             "🎯 Immediate instruction payload intercepted: \"{}\"",
//             immediate_prompt
//         );
//         let _ = query_loop(
//             &immediate_prompt,
//             &mut session,
//             mode,
//             &registry,
//             &mut TerminalUI::new(),
//         )
//         .await;
//     } else {
//         interactive_prompt_loop(&mut session, mode, &registry).await;
//     }
//     Ok(())
// }

// async fn _finally() -> Result<(), Box<dyn std::error::Error>> {
//     let args = Args::parse();

//     println!("=====================================================================");
//     println!("⚡ AGENTIC RUNTIME ENGINE ACTIVATED — LOCAL RUST HARNESS ONLINE ⚡");
//     println!("=====================================================================");

//     let current_dir = std::env::current_dir()?;
//     let mut session = SessionContext::new(&current_dir);
//     let registry = ToolRegistry::new(&current_dir);
//     let selected_mode = PermissionMode::from_str(&args.mode);

//     if let Some(immediate_prompt) = args.prompt {
//         println!(
//             "🎯 Immediate instruction detected: \"{}\"",
//             immediate_prompt
//         );
//         session.append_message("user", &immediate_prompt);
//         query_loop(
//             &immediate_prompt,
//             &mut session,
//             selected_mode,
//             &registry,
//             &mut TerminalUI::new(),
//         )
//         .await?;
//     } else {
//         println!(
//             "[INFO] No direct prompt passed. Initializing interactive prompt line interface..."
//         );
//         let mut input = String::new();
//         println!("\n🤖 [Agent CLI] Enter coding goal or task: ");
//         if std::io::stdin().read_line(&mut input).is_ok() {
//             let task = input.trim();
//             if !task.is_empty() {
//                 session.append_message("user", task);
//                 query_loop(
//                     task,
//                     &mut session,
//                     selected_mode,
//                     &registry,
//                     &mut TerminalUI::new(),
//                 )
//                 .await?;
//             }
//         }
//     }

//     Ok(())
// }

// src/main.rs

use clap::{Parser, ValueEnum};
use std::env;
use std::path::PathBuf;

use crate::action::permissions::PermissionMode;
use crate::orchestrator::dispatch::ExecutionStrategy;
use crate::orchestrator::strategy::CoreEngineRunner;

mod action;
mod core;
mod orchestrator;
mod state;
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

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
enum CliPermission {
    /// Gate risky modifications behind interactive console prompts
    Gated,
    /// Authorize full autonomous background execution
    Auto,
}

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
    #[arg(long, value_enum, default_value_t = CliPermission::Gated)]
    permission: CliPermission, // Changed to '--permission' only (no short flag)

    /// Compilation or testing command utilized for self-healing verification loops
    #[arg(short, long, default_value = "cargo check")]
    compile_cmd: String, // Keeps '-c' and '--compile-cmd'
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

    let selected_permission = match args.permission {
        CliPermission::Gated => PermissionMode::DefaultMode,
        CliPermission::Auto => PermissionMode::Auto,
    };

    // 5. Instantiate Core Engine Runner
    let runner = CoreEngineRunner::new(canonical_workspace, api_key, args.compile_cmd);

    println!("🚀 Launching operational strategy dispatcher...");

    // 6. Handoff context directly into your multi-paradigm orchestrator
    match runner
        .dispatch_request(&args.prompt, selected_strategy, selected_permission)
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
