use crate::tools::Tool;
use async_trait::async_trait;
use serde_json::Value;

/// Tool 1: Generates highly specific, high-barrier B2B enterprise business hypotheses.
/// It explicitly bans generic consumer wrappers or commoditized ideas.
pub struct MarketHypothesisGeneratorTool;

#[async_trait]
impl Tool for MarketHypothesisGeneratorTool {
    fn name(&self) -> &'static str {
        "market_hypothesis_generator"
    }

    fn description(&self) -> &'static str {
        "Generates highly targeted, hyper-specific B2B industrial or enterprise business hypotheses based on regional regulatory bottlenecks and deep engineering stacks."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "target_sector": { "type": "string", "description": "e.g., Heavy Manufacturing, Automotive Tier-1, Logistics, Regulated Fintech" },
                "geographic_region": { "type": "string", "description": "e.g., DACH, Nordics, Pan-EU" },
                "founder_skill_profile": { "type": "string", "description": "Keywords of high-level skills to map against (e.g., Rust, Systems Architecture, Material Science)" }
            },
            "required": ["target_sector", "geographic_region", "founder_skill_profile"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let sector = args["target_sector"]
            .as_str()
            .ok_or("Missing 'target_sector'")?;
        let region = args["geographic_region"]
            .as_str()
            .ok_or("Missing 'geographic_region'")?;
        let skills = args["founder_skill_profile"]
            .as_str()
            .ok_or("Missing 'founder_skill_profile'")?;

        println!(
            "\n🧠 [Hypothesis Engine]: Synthesizing high-barrier wedge for {} in {}...",
            sector, region
        );

        let system_prompt_injection = format!(
            "### HYPOTHESIS GENERATION PROTOCOL ACTIVATED\n\
            Target Sector: {}\n\
            Region: {}\n\
            Founder Capabilities: {}\n\n\
            INSTRUCTION TO AGENT:\n\
            Formulate ONE highly specialized B2B micro-niche business hypothesis. \n\
            CRITERIA:\n\
            1. It must target an expensive, hidden operational failure or a fresh regulatory panick (e.g., EU AI Act, NIS2, local data sovereignty).\n\
            2. It must require a complex software/systems engineering stack ({}) creating an instant moat.\n\
            3. ABSOLUTELY FORBIDDEN: Generic AI wrappers, simple analytics dashboards, or consumer software.",
            sector, region, skills
        );

        Ok(system_prompt_injection)
    }
}

/// Tool 2: The Brutal Bullshit Detector and Kill-Switch Evaluator.
/// This tool forces the agent to analyze the idea through a pessimistic, anti-hype framework.
pub struct BrutalMarketCritiqueTool;

#[async_trait]
impl Tool for BrutalMarketCritiqueTool {
    fn name(&self) -> &'static str {
        "brutal_market_critique"
    }

    fn description(&self) -> &'static str {
        "Subjects a business hypothesis to a merciless market critique, checking for incumbent dominance, data compliance traps, and switching inertia."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "proposed_idea": { "type": "string", "description": "The complete breakdown of the business hypothesis." },
                "assumed_moat": { "type": "string", "description": "Why the creator thinks this cannot be easily copied or replicated by incumbents." },
                "switching_barrier": { "type": "string", "description": "What legacy systems must the client dump, and why would they realistically risk doing so?" }
            },
            "required": ["proposed_idea", "assumed_moat", "switching_barrier"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let idea = args["proposed_idea"]
            .as_str()
            .ok_or("Missing 'proposed_idea'")?;
        let moat = args["assumed_moat"]
            .as_str()
            .ok_or("Missing 'assumed_moat'")?;
        let barrier = args["switching_barrier"]
            .as_str()
            .ok_or("Missing 'switching_barrier'")?;

        println!("\n🔨 [Brutal Critique]: Deconstructing proposal for fatal flaws...");

        let critique_framework = format!(
            "### CRITICAL STRESS-TEST ENGINE UNLOCKED\n\
            Idea Evaluation Matrix:\n\
            - Proposal: {}\n\
            - Claimed Moat: {}\n\
            - Expected Inertia: {}\n\n\
            INSTRUCTION TO AGENT:\n\
            Act as a highly cynical, deeply experienced enterprise auditor. Rip this idea to shreds across these exact axes:\n\
            1. THE INCUMBENT TRAP: Identify the multi-billion dollar heavyweights (e.g., Siemens, SAP, Red Hat) already dominating this exact space or adjacent pipes. Are you just out-plumbing them?\n\
            2. THE INERTIA FACTOR: Is this product 'nice to have' or an absolute compliance/survival requirement? Why would a risk-averse enterprise executive risk their career dumping their current working pipeline for your startup software?\n\
            3. REGULATORY/GDPR DEADLOCK: Does this infrastructure route sensitive, operational metadata through unvetted channels? Is the sales cycle going to die in security committee hell?\n\n\
            OUTPUT FORMAT:\n\
            Conclude your response with an explicit verdict statement using exactly one of these two keys:\n\
            - [VERDICT: KILL PROJECT] followed by the single unfixable structural flaw.\n\
            - [VERDICT: PROCEED TO LEAN PILOT] followed by a concrete, low-cost validation blueprint.",
            idea, moat, barrier
        );

        Ok(critique_framework)
    }
}

// ============================================================================
// STAGE 1: MARKET DISCOVERY & MACROECONOMIC RESEARCH
// ============================================================================
pub struct MarketDiscoveryResearchTool;

#[async_trait]
impl Tool for MarketDiscoveryResearchTool {
    fn name(&self) -> &'static str {
        "market_discovery_research"
    }

    fn description(&self) -> &'static str {
        "Scans a given region and vertical for fresh structural pains, macro shifts, capital allocation flows, or new legislative burdens."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "geographic_region": { "type": "string", "description": "e.g., DACH, Pan-EU, Germany" },
                "industry_vertical": { "type": "string", "description": "e.g., Automotive Logistics, Energy Infrastructure, Maritime Tech" },
                "macro_catalyst": { "type": "string", "description": "e.g., NIS2 Directive, Supply Chain Act (LkSG), carbon tax scaling" }
            },
            "required": ["geographic_region", "industry_vertical", "macro_catalyst"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let region = args["geographic_region"]
            .as_str()
            .ok_or("Missing 'geographic_region'")?;
        let vertical = args["industry_vertical"]
            .as_str()
            .ok_or("Missing 'industry_vertical'")?;
        let catalyst = args["macro_catalyst"]
            .as_str()
            .ok_or("Missing 'macro_catalyst'")?;

        println!(
            "\n🔍 [STAGE 1: RESEARCH] Scanning macroeconomic vulnerability footprint for {} (Vertical: {}) under {}...",
            region, vertical, catalyst
        );

        let system_injection = format!(
            "### STEP 1: ECONOMIC SCOPE & MACRO RESEARCH DISCOVERY\n\
            Region: {}\n\
            Vertical: {}\n\
            Catalyst: {}\n\n\
            INSTRUCTION TO AGENT:\n\
            Analyze the financial impact of this macro catalyst on this industry vertical. Do not look for surface-level opportunities. \n\
            Map out exactly where corporate budgets are being forced to shift. Identify where lines item expenses are swelling due to this change.\n\
            Output a highly structured 'Market Vulnerability Report' outlining the exact mechanism of the pain, estimated operational cost of non-compliance/inefficiency, and corporate stakeholders holding the budget.",
            region, vertical, catalyst
        );

        Ok(system_injection)
    }
}

// ============================================================================
// STAGE 2: TECHNICAL FEASIBILITY & BARRIER-TO-ENTRY AUDIT
// ============================================================================
pub struct TechnicalFeasibilityAuditorTool;

#[async_trait]
impl Tool for TechnicalFeasibilityAuditorTool {
    fn name(&self) -> &'static str {
        "technical_feasibility_auditor"
    }

    fn description(&self) -> &'static str {
        "Evaluates the architectural concept against the founder's stack. Identifies commoditization risks and separates high-barrier tech from low-moat wrappers."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "proposed_architecture": { "type": "string", "description": "Detailed breakdown of the software stack, data topology, and orchestration layer." },
                "founder_core_stack": { "type": "string", "description": "Keywords of exact hard engineering skills available (e.g., Rust, Go, k3s, Linux Kernel development)." }
            },
            "required": ["proposed_architecture", "founder_core_stack"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let architecture = args["proposed_architecture"]
            .as_str()
            .ok_or("Missing 'proposed_architecture'")?;
        let stack = args["founder_core_stack"]
            .as_str()
            .ok_or("Missing 'founder_core_stack'")?;

        println!(
            "\n⚙️ [STAGE 2: HARDCORE ARCHITECTURE AUDIT] Auditing infrastructure complexity vs. commoditization..."
        );

        let system_injection = format!(
            "### STEP 2: ARCHITECTURAL FEASIBILITY & INFRASTRUCTURE MOAT AUDIT\n\
            Proposed Architecture: {}\n\
            Founder Hard Tech Capabilities: {}\n\n\
            INSTRUCTION TO AGENT:\n\
            Act as a cynical Principal Infrastructure Architect. Review the architecture. \n\
            1. If a standard web developer can construct this using a standard cloud backend or basic AI APIs over a weekend, flag it as a 'COMMODITIZED WRAPPER'.\n\
            2. Analyze if the product leverages the founder's advanced technical capabilities ({}) to create a real, defensible barrier (e.g., low-level memory handling, hard real-time scheduling, kernel-level optimizations).\n\
            3. Highlight the exact technical surface area that will require the most R&D capital, and explicitly map out the architectural points of failure.",
            architecture, stack
        );

        Ok(system_injection)
    }
}

// ============================================================================
// STAGE 3: REGULATORY & DATA SOVEREIGNTY VALIDATOR
// ============================================================================
pub struct RegulatoryViabilityAuditorTool;

#[async_trait]
impl Tool for RegulatoryViabilityAuditorTool {
    fn name(&self) -> &'static str {
        "regulatory_viability_auditor"
    }

    fn description(&self) -> &'static str {
        "Mercilessly stress-tests data transit, compliance, storage topology, and data sovereignty walls (GDPR, EU AI Act, TISAX, BaFin)."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "data_transit_topology": { "type": "string", "description": "Where does telemetry/data originate, where is it processed, and where does it land?" },
                "target_regulatory_frameworks": { "type": "string", "description": "e.g., GDPR, EU AI Act Level-4, BaFin CISO requirements, NIS2 compliance" }
            },
            "required": ["data_transit_topology", "target_regulatory_frameworks"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let topology = args["data_transit_topology"]
            .as_str()
            .ok_or("Missing 'data_transit_topology'")?;
        let regulations = args["target_regulatory_frameworks"]
            .as_str()
            .ok_or("Missing 'target_regulatory_frameworks'")?;

        println!(
            "\n⚖️ [STAGE 3: REGULATORY AUDIT] Passing architecture through compliance frameworks..."
        );

        let system_injection = format!(
            "### STEP 3: REGULATORY & COMPLIANCE SOVEREIGNTY AUDIT\n\
            Data Topology: {}\n\
            Target Frameworks: {}\n\n\
            INSTRUCTION TO AGENT:\n\
            Act as an uncompromising European Corporate Compliance Officer and Data Protection Officer (DPO).\n\
            Evaluate the data transit model. Identify every security vulnerability, legal exposure, or compliance roadblock that would cause an enterprise Legal or IT Security committee to instantly block a procurement contract.\n\
            Analyze: Does this data layout touch unvetted US hyper-scalers? Does it route PII or core machine IP outside the edge without deterministic on-prem anonymization? Pinpoint the precise compliance veto point.",
            topology, regulations
        );

        Ok(system_injection)
    }
}

// ============================================================================
// STAGE 4: COMMERCIAL SELLABILITY & INERTIA EVALUATOR
// ============================================================================
pub struct CommercialSellabilityEvaluatorTool;

#[async_trait]
impl Tool for CommercialSellabilityEvaluatorTool {
    fn name(&self) -> &'static str {
        "commercial_sellability_evaluator"
    }

    fn description(&self) -> &'static str {
        "Calculates corporate switching inertia, procurement friction, and whether the economic buyer will realistically risk their career to buy this software."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "value_proposition": { "type": "string", "description": "The exact financial or operational return promised to the economic buyer." },
                "legacy_incumbent_stack": { "type": "string", "description": "What systems (e.g., SAP, Siemens PLM, Salesforce) are currently running that handle this workflow?" },
                "corporate_buyer_persona": { "type": "string", "description": "e.g., Head of IT Procurement, VP of Production Operations, Chief Risk Officer" }
            },
            "required": ["value_proposition", "legacy_incumbent_stack", "corporate_buyer_persona"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let val_prop = args["value_proposition"]
            .as_str()
            .ok_or("Missing 'value_proposition'")?;
        let legacy = args["legacy_incumbent_stack"]
            .as_str()
            .ok_or("Missing 'legacy_incumbent_stack'")?;
        let buyer = args["corporate_buyer_persona"]
            .as_str()
            .ok_or("Missing 'corporate_buyer_persona'")?;

        println!(
            "\n💼 [STAGE 4: SELLABILITY CRITIQUE] Evaluating corporate friction and switching inertia vs. {}...",
            legacy
        );

        let system_injection = format!(
            "### STEP 4: COMMERCIAL SELLABILITY & SWITCHING INERTIA EVALUATION\n\
            Value Prop Claimed: {}\n\
            Incumbent Hegemony: {}\n\
            Target Corporate Buyer: {}\n\n\
            INSTRUCTION TO AGENT:\n\
            Act as a highly risk-averse Corporate Procurement Director. You subscribe to the motto: 'Nobody ever got fired for buying IBM/Siemens/SAP'.\n\
            Deconstruct the proposed value proposition. Evaluate the friction of integration. \n\
            Will installing this startup tool require ripping out or altering a legacy pipeline that has functioned reliably for years? \n\
            Calculate the psychological and professional risk of your target buyer ({}). Is the efficiency gain large enough to justify the career risk of bringing an unvetted startup platform into production? Identify why this sales cycle will realistically stall.",
            val_prop, legacy, buyer
        );

        Ok(system_injection)
    }
}

// ============================================================================
// STAGE 5: THE BRUTAL RED TEAM EXECUTION / TERMINATION GATEWAY
// ============================================================================
pub struct RedTeamKillSwitchTool;

#[async_trait]
impl Tool for RedTeamKillSwitchTool {
    fn name(&self) -> &'static str {
        "red_team_kill_switch"
    }

    fn description(&self) -> &'static str {
        "The ultimate gatekeeper. Processes the outputs of all previous audits and attempts to aggressively terminate the project based on fatal flaws."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "compiled_business_case_state": { "type": "string", "description": "The integrated outputs of Stage 1, 2, 3, and 4." }
            },
            "required": ["compiled_business_case_state"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let state = args["compiled_business_case_state"]
            .as_str()
            .ok_or("Missing 'compiled_business_case_state'")?;

        println!("\n💥 [STAGE 5: RED TEAM KILL SWITCH] Executing final validation stress test...");

        let system_injection = format!(
            "### STEP 5: FINAL RED TEAM KILL SWITCH PROTOCOL\n\
            Compiled Case State Data:\n\
            {}\n\n\
            INSTRUCTION TO AGENT:\n\
            Review the historical trial balance of this idea from the previous 4 steps. Your primary goal right now is to KILL this venture.\n\
            Look for the single unfixable structural flaw: market saturation by well-funded incumbents, fatal data-privacy traps, infinite procurement loops, or low engineering barriers.\n\
            Be completely honest, cynical, and brutally blunt. No soft phrases, no 'potential pivots'. Either prove the idea is a waste of time and terminate it, or demonstrate it survived every trial.\n\n\
            CRITICAL ENFORCED TERMINAL FORMAT:\n\
            Your output must conclude with an explicit, parsable uppercase block string:\n\
            - [VERDICT: TERMINATE VENTURE] followed by a detailed paragraph isolating the single, fatal, unresolvable flaw.\n\
            - [VERDICT: UNANIMOUS PASS TO FIELD LEAN PILOT] followed by a concrete, 3-step, 14-day field experiment validation plan to execute manually.",
            state
        );

        Ok(system_injection)
    }
}

pub struct TechnicalFeasibilityAuditorTool;

#[async_trait]
impl Tool for TechnicalFeasibilityAuditorTool {
    fn name(&self) -> &'static str {
        "technical_feasibility_auditor"
    }

    fn description(&self) -> &'static str {
        "Evaluates the architectural concept against the founder's stack. Identifies commoditization risks and separates high-barrier tech from low-moat wrappers."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "proposed_architecture": { "type": "string", "description": "Detailed breakdown of the software stack, data topology, and orchestration layer." },
                "founder_core_stack": { "type": "string", "description": "Keywords of exact hard engineering skills available (e.g., Rust, Go, k3s, Linux Kernel development)." }
            },
            "required": ["proposed_architecture", "founder_core_stack"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let architecture = args["proposed_architecture"]
            .as_str()
            .ok_or("Missing 'proposed_architecture'")?;
        let stack = args["founder_core_stack"]
            .as_str()
            .ok_or("Missing 'founder_core_stack'")?;

        println!(
            "\n⚙️ [STAGE 2: HARDCORE ARCHITECTURE AUDIT] Auditing infrastructure complexity vs. commoditization..."
        );

        let system_injection = format!(
            "### STEP 2: ARCHITECTURAL FEASIBILITY & INFRASTRUCTURE MOAT AUDIT\n\
            Proposed Architecture: {}\n\
            Founder Hard Tech Capabilities: {}\n\n\
            INSTRUCTION TO AGENT:\n\
            Act as a cynical Principal Infrastructure Architect. Review the architecture. \n\
            1. If a standard web developer can construct this using a standard cloud backend or basic AI APIs over a weekend, flag it as a 'COMMODITIZED WRAPPER'.\n\
            2. Analyze if the product leverages the founder's advanced technical capabilities ({}) to create a real, defensible barrier (e.g., low-level memory handling, hard real-time scheduling, kernel-level optimizations).\n\
            3. Highlight the exact technical surface area that will require the most R&D capital, and explicitly map out the architectural points of failure.",
            architecture, stack, stack
        );

        Ok(system_injection)
    }
}

pub struct CommercialSellabilityEvaluatorTool;

#[async_trait]
impl Tool for CommercialSellabilityEvaluatorTool {
    fn name(&self) -> &'static str {
        "commercial_sellability_evaluator"
    }

    fn description(&self) -> &'static str {
        "Calculates corporate switching inertia, procurement friction, and whether the economic buyer will realistically risk their career to buy this software."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "value_proposition": { "type": "string", "description": "The exact financial or operational return promised to the economic buyer." },
                "legacy_incumbent_stack": { "type": "string", "description": "What systems (e.g., SAP, Siemens PLM, Salesforce) are currently running that handle this workflow?" },
                "corporate_buyer_persona": { "type": "string", "description": "e.g., Head of IT Procurement, VP of Production Operations, Chief Risk Officer" }
            },
            "required": ["value_proposition", "legacy_incumbent_stack", "corporate_buyer_persona"]
        })
    }

    async fn execute(&self, args: &Value) -> Result<String, String> {
        let val_prop = args["value_proposition"]
            .as_str()
            .ok_or("Missing 'value_proposition'")?;
        let legacy = args["legacy_incumbent_stack"]
            .as_str()
            .ok_or("Missing 'legacy_incumbent_stack'")?;
        let buyer = args["corporate_buyer_persona"]
            .as_str()
            .ok_or("Missing 'corporate_buyer_persona'")?;

        println!(
            "\n💼 [STAGE 4: SELLABILITY CRITIQUE] Evaluating corporate friction and switching inertia vs. {}...",
            legacy
        );

        let system_injection = format!(
            "### STEP 4: COMMERCIAL SELLABILITY & SWITCHING INERTIA EVALUATION\n\
            Value Prop Claimed: {}\n\
            Incumbent Hegemony: {}\n\
            Target Corporate Buyer: {}\n\n\
            INSTRUCTION TO AGENT:\n\
            Act as a highly risk-averse Corporate Procurement Director. You subscribe to the motto: 'Nobody ever got fired for buying IBM/Siemens/SAP'.\n\
            Deconstruct the proposed value proposition. Evaluate the friction of integration. \n\
            Will installing this startup tool require ripping out or altering a legacy pipeline that has functioned reliably for years? \n\
            Calculate the psychological and professional risk of your target buyer ({}). Is the efficiency gain large enough to justify the career risk of bringing an unvetted startup platform into production? Identify why this sales cycle will realistically stall.",
            val_prop, legacy, buyer, buyer
        );

        Ok(system_injection)
    }
}
