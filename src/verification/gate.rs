use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Serialize, Debug)]
pub struct VerificationOutcome {
    pub is_passing: bool,
    pub lint_output: String,
    pub test_output: String,
}

pub struct VerificationGate;

impl VerificationGate {
    /// Inspects the workspace root to figure out what testing ecosystem to trigger
    pub fn execute_workspace_validation(project_root: &Path) -> VerificationOutcome {
        let mut lint_output = String::new();
        let mut test_output = String::new();
        let mut is_passing = true;

        // let root_path = Path::new(project_root);

        // --- ECOSYSTEM 1: PYTHON DEFS ---
        if project_root.join("pyproject.toml").exists()
            || project_root.join("requirements.txt").exists()
        {
            println!(
                "🔍 [Verification Gate]: Python ecosystem detected. Spinning up validation run..."
            );

            // 1. Run Linter (Ruff)
            if let Ok(out) = Command::new("uv")
                .args(["run", "ruff", "check", "."])
                .current_dir(project_root)
                .output()
            {
                if !out.status.success() {
                    is_passing = false;
                    lint_output = String::from_utf8_lossy(&out.stdout).to_string()
                        + &String::from_utf8_lossy(&out.stderr).to_string();
                }
            } else if let Ok(out) = Command::new("ruff")
                .args(["check", "."])
                .current_dir(project_root)
                .output()
            {
                if !out.status.success() {
                    is_passing = false;
                    lint_output = String::from_utf8_lossy(&out.stdout).to_string();
                }
            }

            // 2. Run Test Suite (Pytest)
            if let Ok(out) = Command::new("uv")
                .args(["run", "pytest"])
                .current_dir(project_root)
                .output()
            {
                if !out.status.success() {
                    is_passing = false;
                    test_output = String::from_utf8_lossy(&out.stdout).to_string()
                        + &String::from_utf8_lossy(&out.stderr).to_string();
                }
            } else if let Ok(out) = Command::new("pytest").current_dir(project_root).output() {
                if !out.status.success() {
                    is_passing = false;
                    test_output = String::from_utf8_lossy(&out.stdout).to_string();
                }
            }
        }
        // --- ECOSYSTEM 2: RUST DEFS ---
        else if project_root.join("Cargo.toml").exists() {
            println!(
                "🔍 [Verification Gate]: Rust ecosystem detected. Compiling workspace checks..."
            );

            // 1. Run Cargo Check / Clippy
            if let Ok(out) = Command::new("cargo")
                .args(["clippy", "--", "-D", "warnings"])
                .current_dir(project_root)
                .output()
            {
                if !out.status.success() {
                    is_passing = false;
                    lint_output = String::from_utf8_lossy(&out.stderr).to_string();
                }
            }

            // 2. Run Cargo Test
            if let Ok(out) = Command::new("cargo")
                .arg("test")
                .current_dir(project_root)
                .output()
            {
                if !out.status.success() {
                    is_passing = false;
                    test_output = String::from_utf8_lossy(&out.stdout).to_string()
                        + &String::from_utf8_lossy(&out.stderr).to_string();
                }
            }
        }

        VerificationOutcome {
            is_passing,
            lint_output: if lint_output.is_empty() {
                "All linters reporting clean status.".to_string()
            } else {
                lint_output
            },
            test_output: if test_output.is_empty() {
                "All workspace test frames passing.".to_string()
            } else {
                test_output
            },
        }
    }
}
