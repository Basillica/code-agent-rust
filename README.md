# Polyglot Autonomous Engineering Agent Engine

An autonomous, self-healing, multi-file software engineering daemon built in Rust. This agent leverages upfront topological graph planning, deterministic multi-ecosystem bootstrapping, and localized syntax sanitization to design, build, and repair complex application architectures across multiple programming languages completely unsupervised.

## 🚀 Current Architecture & Capabilities

### 1. Pre-Flight Polyglot Bootstrapper

**Capability:** Automatically intercepts operations when an uninitialized or empty project workspace is detected before invoking LLM inference engines.

- **Multi-Ecosystem Support:** Dynamically classifies language environments (`Rust`, `Go`, `TypeScript/Node`, `Python`) from natural language prompt keyword arrays.

**Deterministic Trees:** Spawns idiomatic file frameworks using native system tools (`cargo init`, `go mod init`) and safely falls back to manual blueprint creation if structural local language toolchains are missing.

### 2. Dual-Tier LLM Planning & Repair Circuit

**Strategic Upfront Planner:** Queries local or remote LLMs at a highly rigid temperature (`0.2`) to decompose complex requirements into an initial flat topological task graph (`RefactorPlan`) specifying target relative paths, explicit file dependencies, and clean execution orders.

**Tactical Self-Healing Loop:** Feeds the plan into a stateful `RefactorOrchestrator` transaction layer. If the provided compilation test pipeline throws diagnostic errors, the controller catches the stderr output and passes it down to a specialized repair loop that applies targeted code patches iteratively up to a defined maximum attempt threshold.\n\n### 3. Automated Post-Write Sanitizer Layer

**Token De-contamination:** Intercepts all file-writing actions immediately before they touch disk sectors to strip accidental LLM escaping anomalies, trailing string literals, or outer wrapping quotation artifacts.

**Ecosystem Formatting Hooks:** Integrates structural command extensions (`rustfmt`, `taplo`, `black`, `gofmt`, `prettier`, `clang-format`, `shfmt`) to standardize syntax indentation and unpack collapsed multi-line statements prior to compiler invocation.

## 💎 Why This Engine Is Highly Effective

**Context Window Efficiency:** Rather than wasting thousands of input/output tokens rewriting whole multi-hundred-line code files, it parses localized, surgical `<<<<<<< SEARCH / ======= / >>>>>>> REPLACE` structural diff patch blocks.

**Zero Layout Hallucination:** Moving directory mapping out of the LLM's imagination and into hard-coded Rust tool parameters completely eliminates broken nested directory assumptions (e.g., creating a project inside an unintended subdirectory).

**Local Model Optimization:** Specifically designed to execute successfully using affordable, highly accessible weights (like Gemma and Llama configurations via Ollama) rather than depending solely on costly, high-latency cloud frontier models.

## 🚧 What Is Missing to Achieve Parity with 'Claude Code'

To elevate this local execution prototype into a world-class enterprise system on par with specialized industry tooling like Claude Code, the following layers are still required:

### 1. Advanced Repository Indexing & Semantic Search

**Current Gap:** The planner assumes a clean project or that the user defines exact targets. It cannot read a massive, messy, pre-existing repository to map out code relationships.

**Claude Code Paradigm:** Employs an ongoing background indexing engine that parses projects into an Abstract Syntax Tree (AST), structures a semantic index of internal classes, traits, and functions, and utilizes vector embeddings or optimized system regex utilities (`ripgrep`) to allow the agent to autonomously discover exactly _where_ a feature should be integrated.

### 2. Interactive Human-in-the-Loop (HITL) Guardrails

**Current Gap:** The engine runs completely unsupervised until it either achieves clean compilation or reaches the maximum repair limit.

**Claude Code Paradigm:** Introduces execution breakpoints that prompt users for clear terminal confirmation before executing destructive local shell updates, modifying config properties, or writing dangerous systemic codebase updates.

### 3. Open-Ended Sandboxed Shell Execution

**Current Gap:** The system is restricted to a rigid, hardcoded compilation check sequence string.

**Claude Code Paradigm:** Grants the model access to an abstract, stateful terminal execution tool. The agent can independently decide to run database migrations, test docker compose configurations, spin up temporary development servers, and dynamically read interactive terminal output logs to debug runtime errors.

### 4. Behavioral Evaluation via Automated Test Suites

**Current Gap:** The self-healing logic assumes that if the code compiles without syntax errors, the task is successfully resolved.

**Claude Code Paradigm:** Evaluates functional logic by scanning, executing, and monitoring test suites (`cargo test`, `jest`, `pytest`). If a code change passes compilation but breaks a unit or integration test, the agent processes the test stack trace through the self-healing loop to patch logic regression bugs.

### 5. Native Version Control System (VCS) Lifecycle Transaction Control

**Current Gap:** Mutations overwrite files directly in place within the active local workspace directory.

**Claude Code Paradigm:** Automatically initializes isolated git feature branches for new prompt sessions, performs atomic staging and clean git commits for individual sub-tasks in the execution graph, auto-reverts workspace states to a known stable commit if a repair branch breaks down completely, and automatically formats complete pull request markdown descriptions on success."
