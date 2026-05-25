use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct BootstrapProjectTool {
    pub project_root: PathBuf,
}

#[async_trait]
impl Tool for BootstrapProjectTool {
    fn name(&self) -> &'static str {
        "bootstrap_project"
    }

    fn description(&self) -> &'static str {
        "Initializes a clean, idiomatically structured directory and config skeleton for a specific programming language directly at the workspace root."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "language": {
                    "type": "string",
                    "enum": ["rust", "go", "typescript", "python"],
                    "description": "The target programming language ecosystem to establish."
                },
                "project_name": {
                    "type": "string",
                    "description": "The systemic snake_case name of the application (e.g., 'time_server')."
                }
            },
            "required": ["language", "project_name"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let language = args["language"]
            .as_str()
            .ok_or("Missing parameter 'language'")?;
        let project_name = args["project_name"]
            .as_str()
            .ok_or("Missing parameter 'project_name'")?;

        // Sanitize project name to be idiomatic snake_case
        let sanitized_name = project_name.trim().to_lowercase().replace("-", "_");

        match language {
            "rust" => self.bootstrap_rust(&sanitized_name).await,
            "go" => self.bootstrap_go(&sanitized_name).await,
            "typescript" => self.bootstrap_typescript(&sanitized_name).await,
            "python" => self.bootstrap_python(&sanitized_name).await,
            _ => Err(format!(
                "Unsupported ecosystem language environment: {}",
                language
            )),
        }
    }
}

impl BootstrapProjectTool {
    pub fn new(path: PathBuf) -> Self {
        Self { project_root: path }
    }
    /// Establishes an idiomatic Rust binary binary layout
    async fn bootstrap_rust(&self, name: &str) -> Result<String, String> {
        let cargo_toml = self.project_root.join("Cargo.toml");
        let src_dir = self.project_root.join("src");
        let main_rs = src_dir.join("main.rs");

        // If cargo toml already exists, skip initialization to preserve history
        if cargo_toml.exists() {
            return Ok("A Cargo workspace already exists at this root. Skipping bootstrap initialization step.".to_string());
        }

        // Try using system native cargo init if available
        if Command::new("cargo").arg("-v").output().is_ok() {
            let output = Command::new("cargo")
                .args(["init", "--bin", "--name", name])
                .current_dir(&self.project_root)
                .output()
                .map_err(|e| format!("Cargo native shell initialization crashed: {}", e))?;

            if output.status.success() {
                return Ok(format!(
                    "Successfully initialized native Cargo binary project: '{}'",
                    name
                ));
            }
        }

        // Fallback: Manually build a structurally perfect Rust workspace file tree
        fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;

        let initial_toml = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n",
            name
        );
        fs::write(cargo_toml, initial_toml).map_err(|e| e.to_string())?;
        fs::write(
            main_rs,
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        )
        .map_err(|e| e.to_string())?;

        Ok(format!(
            "Successfully bootstrapped file tree for Rust binary package: '{}'",
            name
        ))
    }

    /// Establishes an idiomatic Go Module structure
    async fn bootstrap_go(&self, name: &str) -> Result<String, String> {
        let go_mod = self.project_root.join("go.mod");
        let main_go = self.project_root.join("main.go");

        if go_mod.exists() {
            return Ok("Go module environment already exists here.".to_string());
        }

        if Command::new("go").arg("version").output().is_ok() {
            Command::new("go")
                .args(["mod", "init", name])
                .current_dir(&self.project_root)
                .status()
                .map_err(|e| e.to_string())?;
        } else {
            let fallback_mod = format!("module {}\n\ngo 1.22\n", name);
            fs::write(go_mod, fallback_mod).map_err(|e| e.to_string())?;
        }

        let fallback_main =
            "package main\n\nimport \"fmt\"\n\nfn main() {\n\tfmt.Println(\"Hello World\")\n}\n";
        fs::write(main_go, fallback_main).map_err(|e| e.to_string())?;

        Ok(format!(
            "Successfully structured Go Module workspace for '{}'",
            name
        ))
    }

    /// Establishes an idiomatic full-stack TypeScript/Node workspace layout
    async fn bootstrap_typescript(&self, name: &str) -> Result<String, String> {
        let pkg_json = self.project_root.join("package.json");
        let ts_config = self.project_root.join("tsconfig.json");
        let src_dir = self.project_root.join("src");

        fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;

        let initial_pkg = serde_json::json!({
            "name": name,
            "version": "1.0.0",
            "description": "",
            "main": "dist/index.js",
            "scripts": {
                "build": "tsc",
                "start": "node dist/index.js",
                "dev": "ts-node src/index.ts"
            },
            "dependencies": {},
            "devDependencies": {
                "typescript": "^5.0.0",
                "@types/node": "^20.0.0",
                "ts-node": "^10.9.0"
            }
        });

        let initial_tsconfig = serde_json::json!({
            "compilerOptions": {
                "target": "ES2022",
                "module": "CommonJS",
                "outDir": "./dist",
                "rootDir": "./src",
                "strict": true,
                "esModuleInterop": true,
                "skipLibCheck": true,
                "forceConsistentCasingInFileNames": true
            },
            "include": ["src/**/*"]
        });

        fs::write(
            pkg_json,
            serde_json::to_string_pretty(&initial_pkg).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            ts_config,
            serde_json::to_string_pretty(&initial_tsconfig).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            src_dir.join("index.ts"),
            "console.log('Hello via TypeScript!');\n",
        )
        .map_err(|e| e.to_string())?;

        Ok(format!(
            "Successfully built modern TypeScript package scaffolding for '{}'",
            name
        ))
    }

    /// Establishes an idiomatic modern Python project layout using pyproject.toml
    async fn bootstrap_python(&self, name: &str) -> Result<String, String> {
        let pyproject = self.project_root.join("pyproject.toml");
        let app_dir = self.project_root.join(name);

        fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

        let initial_pyproject = format!(
            "[project]\nname = \"{}\"\nversion = \"0.1.0\"\ndescription = \"Autogenerated project\"\ndependencies = []\n",
            name
        );

        fs::write(pyproject, initial_pyproject).map_err(|e| e.to_string())?;
        fs::write(app_dir.join("__init__.py"), "").map_err(|e| e.to_string())?;
        fs::write(
            app_dir.join("main.py"),
            "def main():\n    print('Hello World!')\n\nif __name__ == '__main__':\n    main()\n",
        )
        .map_err(|e| e.to_string())?;

        Ok(format!(
            "Successfully initialized modern Python project layout for '{}'",
            name
        ))
    }
}
