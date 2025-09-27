//! Safety guards and validation for agent execution

use async_trait::async_trait;
use std::collections::HashMap;

use axiom_core::{Result, AxiomError};

/// Trait for safety guards that validate agent actions
#[async_trait]
pub trait SafetyGuard: Send + Sync {
    /// Check if a tool call is safe
    async fn check_tool_call(&self, tool_name: &str, arguments: &HashMap<String, serde_json::Value>) -> Result<SafetyResult>;
    
    /// Check if a response is safe
    async fn check_response(&self, response: &str) -> Result<SafetyResult>;
    
    /// Check if a plan is safe
    async fn check_plan(&self, plan: &crate::planner::ExecutionPlan) -> Result<SafetyResult>;
}

/// Result of a safety check
#[derive(Debug, Clone)]
pub struct SafetyResult {
    /// Whether the action is safe
    pub is_safe: bool,
    /// Reason for the safety decision
    pub reason: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SafetyResult {
    /// Create a safe result
    pub fn safe(reason: impl Into<String>) -> Self {
        Self {
            is_safe: true,
            reason: reason.into(),
            confidence: 1.0,
            metadata: HashMap::new(),
        }
    }

    /// Create an unsafe result
    pub fn unsafe_result(reason: impl Into<String>) -> Self {
        Self {
            is_safe: false,
            reason: reason.into(),
            confidence: 1.0,
            metadata: HashMap::new(),
        }
    }

    /// Create a result with confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Simple safety guard with basic checks
pub struct SimpleSafetyGuard {
    /// Allowed tools
    allowed_tools: Vec<String>,
    /// Blocked tools
    blocked_tools: Vec<String>,
    /// Blocked keywords
    blocked_keywords: Vec<String>,
    /// Enable content filtering
    enable_content_filter: bool,
}

impl SimpleSafetyGuard {
    /// Create a new simple safety guard
    pub fn new() -> Self {
        Self {
            allowed_tools: Vec::new(),
            blocked_tools: Vec::new(),
            blocked_keywords: vec![
                "hack".to_string(),
                "exploit".to_string(),
                "malware".to_string(),
                "virus".to_string(),
                "phishing".to_string(),
                "spam".to_string(),
            ],
            enable_content_filter: true,
        }
    }

    /// Add an allowed tool
    pub fn allow_tool(mut self, tool_name: impl Into<String>) -> Self {
        self.allowed_tools.push(tool_name.into());
        self
    }

    /// Add a blocked tool
    pub fn block_tool(mut self, tool_name: impl Into<String>) -> Self {
        self.blocked_tools.push(tool_name.into());
        self
    }

    /// Add a blocked keyword
    pub fn block_keyword(mut self, keyword: impl Into<String>) -> Self {
        self.blocked_keywords.push(keyword.into());
        self
    }

    /// Enable or disable content filtering
    pub fn with_content_filter(mut self, enable: bool) -> Self {
        self.enable_content_filter = enable;
        self
    }
}

#[async_trait]
impl SafetyGuard for SimpleSafetyGuard {
    async fn check_tool_call(&self, tool_name: &str, arguments: &HashMap<String, serde_json::Value>) -> Result<SafetyResult> {
        // Check if tool is blocked
        if self.blocked_tools.contains(&tool_name.to_string()) {
            return Ok(SafetyResult::unsafe_result(format!("Tool '{}' is blocked", tool_name)));
        }

        // Check if tool is in allowed list (if specified)
        if !self.allowed_tools.is_empty() && !self.allowed_tools.contains(&tool_name.to_string()) {
            return Ok(SafetyResult::unsafe_result(format!("Tool '{}' is not in allowed list", tool_name)));
        }

        // Check arguments for blocked keywords
        if self.enable_content_filter {
            for (_, value) in arguments {
                if let Some(text) = value.as_str() {
                    for keyword in &self.blocked_keywords {
                        if text.to_lowercase().contains(keyword) {
                            return Ok(SafetyResult::unsafe_result(format!(
                                "Tool call contains blocked keyword: '{}'", keyword
                            )));
                        }
                    }
                }
            }
        }

        Ok(SafetyResult::safe("Tool call is safe"))
    }

    async fn check_response(&self, response: &str) -> Result<SafetyResult> {
        if !self.enable_content_filter {
            return Ok(SafetyResult::safe("Content filtering disabled"));
        }

        let response_lower = response.to_lowercase();
        
        // Check for blocked keywords
        for keyword in &self.blocked_keywords {
            if response_lower.contains(keyword) {
                return Ok(SafetyResult::unsafe_result(format!(
                    "Response contains blocked keyword: '{}'", keyword
                )));
            }
        }

        // Check for potentially harmful patterns
        if self.contains_harmful_patterns(response) {
            return Ok(SafetyResult::unsafe_result("Response contains potentially harmful patterns"));
        }

        Ok(SafetyResult::safe("Response is safe"))
    }

    async fn check_plan(&self, plan: &crate::planner::ExecutionPlan) -> Result<SafetyResult> {
        // Check each step in the plan
        for step in &plan.steps {
            match &step.action {
                crate::planner::PlanAction::CallTool { tool_name, arguments } => {
                    let tool_result = self.check_tool_call(tool_name, arguments).await?;
                    if !tool_result.is_safe {
                        return Ok(tool_result);
                    }
                }
                crate::planner::PlanAction::GenerateResponse { prompt } => {
                    let response_result = self.check_response(prompt).await?;
                    if !response_result.is_safe {
                        return Ok(response_result);
                    }
                }
                _ => {
                    // Other actions are generally safe
                }
            }
        }

        Ok(SafetyResult::safe("Plan is safe"))
    }
}

impl SimpleSafetyGuard {
    /// Check if text contains harmful patterns
    fn contains_harmful_patterns(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        
        // Check for common harmful patterns
        let harmful_patterns = [
            "execute code",
            "run command",
            "delete file",
            "format disk",
            "shutdown system",
            "restart computer",
            "kill process",
            "sudo rm -rf",
            "chmod 777",
        ];

        for pattern in &harmful_patterns {
            if text_lower.contains(pattern) {
                return true;
            }
        }

        false
    }
}

/// Advanced safety guard using LLM for content analysis
pub struct LLMSafetyGuard {
    /// LLM gateway for safety checks
    llm_gateway: std::sync::Arc<axiom_llm::LlmGateway>,
    /// Model to use for safety checks
    model: String,
    /// Safety threshold (0.0 to 1.0)
    safety_threshold: f32,
    /// Base safety guard for basic checks
    base_guard: SimpleSafetyGuard,
}

impl LLMSafetyGuard {
    /// Create a new LLM safety guard
    pub fn new(
        llm_gateway: std::sync::Arc<axiom_llm::LlmGateway>,
        model: String,
        safety_threshold: f32,
    ) -> Self {
        Self {
            llm_gateway,
            model,
            safety_threshold: safety_threshold.clamp(0.0, 1.0),
            base_guard: SimpleSafetyGuard::new(),
        }
    }

    /// Set safety threshold
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.safety_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Check content safety using LLM
    async fn check_content_safety(&self, content: &str, content_type: &str) -> Result<SafetyResult> {
        let prompt = format!(
            "Analyze the following {} for safety. Consider if it contains harmful, inappropriate, or dangerous content.\n\nContent: {}\n\nRespond with a JSON object containing:\n- 'safe': boolean indicating if the content is safe\n- 'reason': string explaining the decision\n- 'confidence': number between 0.0 and 1.0 indicating confidence\n- 'risk_level': string indicating risk level (low, medium, high)",
            content_type, content
        );

        let request = axiom_llm::LlmRequest::new(
            vec![axiom_core::Message::user(prompt)],
            self.model.clone(),
        );

        let response = self.llm_gateway.generate(request).await?;
        
        // Parse LLM response
        self.parse_safety_response(&response.content)
    }

    /// Parse LLM safety response
    fn parse_safety_response(&self, response: &str) -> Result<SafetyResult> {
        // Try to parse JSON response
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            let is_safe = parsed.get("safe")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            let reason = parsed.get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown reason")
                .to_string();
            
            let confidence = parsed.get("confidence")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32)
                .unwrap_or(0.5);
            
            let risk_level = parsed.get("risk_level")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let mut metadata = HashMap::new();
            metadata.insert("risk_level".to_string(), serde_json::Value::String(risk_level));

            Ok(SafetyResult {
                is_safe,
                reason,
                confidence,
                metadata,
            })
        } else {
            // Fallback to basic analysis
            Ok(SafetyResult::safe("Could not parse LLM response, assuming safe"))
        }
    }
}

#[async_trait]
impl SafetyGuard for LLMSafetyGuard {
    async fn check_tool_call(&self, tool_name: &str, arguments: &HashMap<String, serde_json::Value>) -> Result<SafetyResult> {
        // First check with base guard
        let base_result = self.base_guard.check_tool_call(tool_name, arguments).await?;
        if !base_result.is_safe {
            return Ok(base_result);
        }

        // Then check with LLM
        let content = format!("Tool: {}\nArguments: {}", tool_name, serde_json::to_string(arguments)?);
        let llm_result = self.check_content_safety(&content, "tool call").await?;

        // Combine results
        if llm_result.confidence >= self.safety_threshold {
            Ok(llm_result)
        } else {
            Ok(SafetyResult::safe("Low confidence, but no clear safety issues"))
        }
    }

    async fn check_response(&self, response: &str) -> Result<SafetyResult> {
        // First check with base guard
        let base_result = self.base_guard.check_response(response).await?;
        if !base_result.is_safe {
            return Ok(base_result);
        }

        // Then check with LLM
        let llm_result = self.check_content_safety(response, "response").await?;

        // Combine results
        if llm_result.confidence >= self.safety_threshold {
            Ok(llm_result)
        } else {
            Ok(SafetyResult::safe("Low confidence, but no clear safety issues"))
        }
    }

    async fn check_plan(&self, plan: &crate::planner::ExecutionPlan) -> Result<SafetyResult> {
        // First check with base guard
        let base_result = self.base_guard.check_plan(plan).await?;
        if !base_result.is_safe {
            return Ok(base_result);
        }

        // Then check with LLM
        let plan_description = format!("Plan with {} steps", plan.steps.len());
        let llm_result = self.check_content_safety(&plan_description, "execution plan").await?;

        // Combine results
        if llm_result.confidence >= self.safety_threshold {
            Ok(llm_result)
        } else {
            Ok(SafetyResult::safe("Low confidence, but no clear safety issues"))
        }
    }
}

/// Safety guard that combines multiple guards
pub struct CompositeSafetyGuard {
    /// List of safety guards
    guards: Vec<Box<dyn SafetyGuard>>,
    /// How to combine results (all must pass, any can pass, etc.)
    combination_strategy: CombinationStrategy,
}

/// Strategy for combining multiple safety guard results
#[derive(Debug, Clone)]
pub enum CombinationStrategy {
    /// All guards must pass
    AllMustPass,
    /// Any guard can pass
    AnyCanPass,
    /// Majority must pass
    MajorityMustPass,
    /// Weighted combination
    Weighted(Vec<f32>),
}

impl CompositeSafetyGuard {
    /// Create a new composite safety guard
    pub fn new(strategy: CombinationStrategy) -> Self {
        Self {
            guards: Vec::new(),
            combination_strategy: strategy,
        }
    }

    /// Add a safety guard
    pub fn add_guard(mut self, guard: Box<dyn SafetyGuard>) -> Self {
        self.guards.push(guard);
        self
    }

    /// Combine multiple safety results
    fn combine_results(&self, results: Vec<SafetyResult>) -> SafetyResult {
        if results.is_empty() {
            return SafetyResult::safe("No guards to check");
        }

        match &self.combination_strategy {
            CombinationStrategy::AllMustPass => {
                let all_safe = results.iter().all(|r| r.is_safe);
                let reasons: Vec<String> = results.iter().map(|r| r.reason.clone()).collect();
                
                SafetyResult {
                    is_safe: all_safe,
                    reason: if all_safe {
                        "All guards passed".to_string()
                    } else {
                        format!("Some guards failed: {}", reasons.join(", "))
                    },
                    confidence: results.iter().map(|r| r.confidence).sum::<f32>() / results.len() as f32,
                    metadata: HashMap::new(),
                }
            }
            CombinationStrategy::AnyCanPass => {
                let any_safe = results.iter().any(|r| r.is_safe);
                let safe_reasons: Vec<String> = results.iter()
                    .filter(|r| r.is_safe)
                    .map(|r| r.reason.clone())
                    .collect();
                
                SafetyResult {
                    is_safe: any_safe,
                    reason: if any_safe {
                        format!("At least one guard passed: {}", safe_reasons.join(", "))
                    } else {
                        "All guards failed".to_string()
                    },
                    confidence: results.iter().map(|r| r.confidence).sum::<f32>() / results.len() as f32,
                    metadata: HashMap::new(),
                }
            }
            CombinationStrategy::MajorityMustPass => {
                let safe_count = results.iter().filter(|r| r.is_safe).count();
                let majority_safe = safe_count > results.len() / 2;
                
                SafetyResult {
                    is_safe: majority_safe,
                    reason: if majority_safe {
                        format!("Majority of guards passed ({}/{})", safe_count, results.len())
                    } else {
                        format!("Majority of guards failed ({}/{})", results.len() - safe_count, results.len())
                    },
                    confidence: results.iter().map(|r| r.confidence).sum::<f32>() / results.len() as f32,
                    metadata: HashMap::new(),
                }
            }
            CombinationStrategy::Weighted(weights) => {
                if weights.len() != results.len() {
                    return SafetyResult::unsafe_result("Weight count mismatch");
                }
                
                let weighted_score: f32 = results.iter()
                    .zip(weights.iter())
                    .map(|(result, weight)| if result.is_safe { *weight } else { 0.0 })
                    .sum();
                
                let total_weight: f32 = weights.iter().sum();
                let threshold = total_weight / 2.0;
                
                SafetyResult {
                    is_safe: weighted_score >= threshold,
                    reason: format!("Weighted score: {:.2}/{:.2}", weighted_score, total_weight),
                    confidence: weighted_score / total_weight,
                    metadata: HashMap::new(),
                }
            }
        }
    }
}

#[async_trait]
impl SafetyGuard for CompositeSafetyGuard {
    async fn check_tool_call(&self, tool_name: &str, arguments: &HashMap<String, serde_json::Value>) -> Result<SafetyResult> {
        let futures: Vec<_> = self.guards.iter()
            .map(|guard| guard.check_tool_call(tool_name, arguments))
            .collect();
        
        let results = futures::future::try_join_all(futures).await?;
        Ok(self.combine_results(results))
    }

    async fn check_response(&self, response: &str) -> Result<SafetyResult> {
        let futures: Vec<_> = self.guards.iter()
            .map(|guard| guard.check_response(response))
            .collect();
        
        let results = futures::future::try_join_all(futures).await?;
        Ok(self.combine_results(results))
    }

    async fn check_plan(&self, plan: &crate::planner::ExecutionPlan) -> Result<SafetyResult> {
        let futures: Vec<_> = self.guards.iter()
            .map(|guard| guard.check_plan(plan))
            .collect();
        
        let results = futures::future::try_join_all(futures).await?;
        Ok(self.combine_results(results))
    }
}
