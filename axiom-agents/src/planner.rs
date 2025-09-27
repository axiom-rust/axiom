//! Planning system for agent execution

use async_trait::async_trait;
use std::collections::HashMap;

use axiom_core::{Message, Result, AxiomError, MemoryType};
// Types are defined in this module
use crate::agent::{PlanningRequest, ToolInfo};

/// Trait for planning agent execution
#[async_trait]
pub trait Planner: Send + Sync {
    /// Create an execution plan based on the request
    async fn create_plan(&self, request: PlanningRequest) -> Result<ExecutionPlan>;
}

/// An execution plan consisting of multiple steps
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// Steps in the plan
    pub steps: Vec<PlanStep>,
    /// Plan metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Estimated execution time in milliseconds
    pub estimated_time_ms: u64,
}

/// A single step in an execution plan
#[derive(Debug, Clone)]
pub struct PlanStep {
    /// Step identifier
    pub id: String,
    /// Action to perform
    pub action: PlanAction,
    /// Dependencies on other steps
    pub dependencies: Vec<String>,
    /// Estimated execution time in milliseconds
    pub estimated_time_ms: u64,
    /// Priority (higher = more important)
    pub priority: u32,
}

/// Actions that can be performed in a plan step
#[derive(Debug, Clone)]
pub enum PlanAction {
    /// Generate a response using the LLM
    GenerateResponse { prompt: String },
    /// Call a tool
    CallTool { tool_name: String, arguments: HashMap<String, serde_json::Value> },
    /// Update memory
    UpdateMemory { content: String, memory_type: MemoryType },
    /// Retrieve memory
    RetrieveMemory { query: String, limit: usize },
}

/// Simple planner that creates basic execution plans
pub struct SimplePlanner {
    /// Maximum number of steps in a plan
    max_steps: usize,
    /// Whether to enable parallel execution
    enable_parallel: bool,
}

impl SimplePlanner {
    /// Create a new simple planner
    pub fn new(max_steps: usize) -> Self {
        Self {
            max_steps,
            enable_parallel: false,
        }
    }

    /// Enable parallel execution
    pub fn with_parallel(mut self, enable: bool) -> Self {
        self.enable_parallel = enable;
        self
    }
}

#[async_trait]
impl Planner for SimplePlanner {
    async fn create_plan(&self, request: PlanningRequest) -> Result<ExecutionPlan> {
        let mut steps = Vec::new();
        let mut step_counter = 0;

        // Analyze the conversation to determine what needs to be done
        let last_message = request.conversation.last()
            .and_then(|msg| msg.text_content())
            .unwrap_or("");

        // Check if we need to retrieve memory
        if self.should_retrieve_memory(last_message, &request.context) {
            steps.push(PlanStep {
                id: format!("retrieve_memory_{}", step_counter),
                action: PlanAction::RetrieveMemory {
                    query: last_message.to_string(),
                    limit: 5,
                },
                dependencies: Vec::new(),
                estimated_time_ms: 100,
                priority: 1,
            });
            step_counter += 1;
        }

        // Check if we need to call tools
        let tool_calls = self.identify_tool_calls(last_message, &request.available_tools);
        for tool_call in tool_calls {
            steps.push(PlanStep {
                id: format!("call_tool_{}", step_counter),
                action: PlanAction::CallTool {
                    tool_name: tool_call.tool_name,
                    arguments: tool_call.arguments,
                },
                dependencies: Vec::new(),
                estimated_time_ms: 1000,
                priority: 2,
            });
            step_counter += 1;
        }

        // Always generate a response
        steps.push(PlanStep {
            id: format!("generate_response_{}", step_counter),
            action: PlanAction::GenerateResponse {
                prompt: last_message.to_string(),
            },
            dependencies: (0..step_counter).map(|i| format!("retrieve_memory_{}", i)).collect(),
            estimated_time_ms: 2000,
            priority: 3,
        });

        // Limit the number of steps
        if steps.len() > self.max_steps {
            steps.truncate(self.max_steps);
        }

        // Calculate total estimated time
        let estimated_time = steps.iter().map(|s| s.estimated_time_ms).sum();

        Ok(ExecutionPlan {
            steps,
            metadata: HashMap::new(),
            estimated_time_ms: estimated_time,
        })
    }
}

impl SimplePlanner {
    /// Check if we should retrieve memory
    fn should_retrieve_memory(&self, message: &str, context: &[axiom_core::MemoryItem]) -> bool {
        // Simple heuristic: retrieve memory if context is empty or message asks for information
        context.is_empty() || message.to_lowercase().contains("remember") || message.to_lowercase().contains("recall")
    }

    /// Identify tool calls needed based on the message
    fn identify_tool_calls(&self, message: &str, available_tools: &[ToolInfo]) -> Vec<ToolCall> {
        let mut tool_calls = Vec::new();
        let message_lower = message.to_lowercase();

        for tool in available_tools {
            if self.should_call_tool(&message_lower, tool) {
                tool_calls.push(ToolCall {
                    tool_name: tool.name.clone(),
                    arguments: self.extract_tool_arguments(message, tool),
                });
            }
        }

        tool_calls
    }

    /// Check if a tool should be called based on the message
    fn should_call_tool(&self, message: &str, tool: &ToolInfo) -> bool {
        let tool_name_lower = tool.name.to_lowercase();
        let description_lower = tool.description.to_lowercase();

        // Simple keyword matching
        message.contains(&tool_name_lower) ||
        message.contains("search") && description_lower.contains("search") ||
        message.contains("calculate") && description_lower.contains("calculate") ||
        message.contains("weather") && description_lower.contains("weather") ||
        message.contains("time") && description_lower.contains("time")
    }

    /// Extract arguments for a tool call
    fn extract_tool_arguments(&self, message: &str, tool: &ToolInfo) -> HashMap<String, serde_json::Value> {
        let mut arguments = HashMap::new();

        // Simple argument extraction based on tool name
        match tool.name.as_str() {
            "search" => {
                arguments.insert("query".to_string(), serde_json::Value::String(message.to_string()));
            }
            "calculate" => {
                // Extract mathematical expression
                let expr = self.extract_math_expression(message);
                arguments.insert("expression".to_string(), serde_json::Value::String(expr));
            }
            "weather" => {
                // Extract location
                let location = self.extract_location(message);
                arguments.insert("location".to_string(), serde_json::Value::String(location));
            }
            _ => {
                // Default: pass the entire message
                arguments.insert("input".to_string(), serde_json::Value::String(message.to_string()));
            }
        }

        arguments
    }

    /// Extract mathematical expression from message
    fn extract_math_expression(&self, message: &str) -> String {
        // Simple regex-like extraction for math expressions
        // In a real implementation, you'd use a proper regex library
        let words: Vec<&str> = message.split_whitespace().collect();
        let mut expr = String::new();
        
        for word in words {
            if word.chars().any(|c| c.is_numeric() || "+-*/()".contains(c)) {
                expr.push_str(word);
                expr.push(' ');
            }
        }
        
        expr.trim().to_string()
    }

    /// Extract location from message
    fn extract_location(&self, message: &str) -> String {
        // Simple location extraction
        // In a real implementation, you'd use NER or more sophisticated methods
        let words: Vec<&str> = message.split_whitespace().collect();
        let mut location = String::new();
        
        for (i, word) in words.iter().enumerate() {
            if word.to_lowercase() == "in" && i + 1 < words.len() {
                location = words[i + 1..].join(" ");
                break;
            }
        }
        
        if location.is_empty() {
            "current location".to_string()
        } else {
            location
        }
    }
}

/// A tool call identified by the planner
#[derive(Debug, Clone)]
struct ToolCall {
    tool_name: String,
    arguments: HashMap<String, serde_json::Value>,
}

/// Advanced planner that uses LLM for planning
pub struct LLMPlanner {
    /// LLM gateway for planning
    llm_gateway: std::sync::Arc<axiom_llm::LlmGateway>,
    /// Maximum number of steps
    max_steps: usize,
    /// Planning model
    model: String,
}

impl LLMPlanner {
    /// Create a new LLM planner
    pub fn new(llm_gateway: std::sync::Arc<axiom_llm::LlmGateway>, model: String) -> Self {
        Self {
            llm_gateway,
            max_steps: 10,
            model,
        }
    }

    /// Set maximum number of steps
    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }
}

#[async_trait]
impl Planner for LLMPlanner {
    async fn create_plan(&self, request: PlanningRequest) -> Result<ExecutionPlan> {
        // Create a planning prompt
        let prompt = self.create_planning_prompt(&request);
        
        // Call LLM to generate plan
        let llm_request = axiom_llm::LlmRequest::new(
            vec![axiom_core::Message::user(prompt)],
            self.model.clone(),
        );

        let response = self.llm_gateway.generate(llm_request).await?;
        
        // Parse the response into a plan
        self.parse_plan_response(&response.content, &request)
    }
}

impl LLMPlanner {
    /// Create a planning prompt
    fn create_planning_prompt(&self, request: &PlanningRequest) -> String {
        let mut prompt = String::new();
        
        prompt.push_str("You are an AI planning agent. Create an execution plan based on the following conversation and available tools.\n\n");
        
        prompt.push_str("Conversation:\n");
        for message in &request.conversation {
            prompt.push_str(&format!("{}: {}\n", 
                match message.role {
                    axiom_core::MessageRole::User => "User",
                    axiom_core::MessageRole::Assistant => "Assistant",
                    axiom_core::MessageRole::System => "System",
                    axiom_core::MessageRole::Tool => "Tool",
                },
                message.text_content().unwrap_or("")
            ));
        }
        
        prompt.push_str("\nAvailable tools:\n");
        for tool in &request.available_tools {
            prompt.push_str(&format!("- {}: {}\n", tool.name, tool.description));
        }
        
        prompt.push_str("\nCreate a step-by-step execution plan. Each step should be in the format:\n");
        prompt.push_str("STEP: <action_type> | <parameters> | <dependencies>\n");
        prompt.push_str("Available actions: generate_response, call_tool, update_memory, retrieve_memory\n");
        
        prompt
    }

    /// Parse LLM response into a plan
    fn parse_plan_response(&self, response: &str, request: &PlanningRequest) -> Result<ExecutionPlan> {
        let mut steps = Vec::new();
        let mut step_counter = 0;

        for line in response.lines() {
            if line.starts_with("STEP:") {
                let step = self.parse_step_line(line, &mut step_counter, request)?;
                steps.push(step);
            }
        }

        // Limit steps
        if steps.len() > self.max_steps {
            steps.truncate(self.max_steps);
        }

        let estimated_time = steps.iter().map(|s| s.estimated_time_ms).sum();

        Ok(ExecutionPlan {
            steps,
            metadata: HashMap::new(),
            estimated_time_ms: estimated_time,
        })
    }

    /// Parse a single step line
    fn parse_step_line(&self, line: &str, step_counter: &mut usize, request: &PlanningRequest) -> Result<PlanStep> {
        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        
        if parts.len() < 2 {
            return Err(AxiomError::ChainExecution("Invalid step format".to_string()));
        }

        let action_part = parts[0].replace("STEP:", "").trim().to_string();
        let params_part = parts[1];
        let deps_part = if parts.len() > 2 { parts[2] } else { "" };

        let action = self.parse_action(&action_part, params_part, request)?;
        let dependencies = if deps_part.is_empty() {
            Vec::new()
        } else {
            deps_part.split(',').map(|s| s.trim().to_string()).collect()
        };

        *step_counter += 1;

        Ok(PlanStep {
            id: format!("step_{}", step_counter),
            action,
            dependencies,
            estimated_time_ms: 1000, // Default estimate
            priority: 1,
        })
    }

    /// Parse action from text
    fn parse_action(&self, action_type: &str, params: &str, request: &PlanningRequest) -> Result<PlanAction> {
        match action_type.trim() {
            "generate_response" => {
                Ok(PlanAction::GenerateResponse {
                    prompt: params.to_string(),
                })
            }
            "call_tool" => {
                let tool_name = params.split_whitespace().next()
                    .ok_or_else(|| AxiomError::ChainExecution("Missing tool name".to_string()))?;
                
                // Find the tool in available tools
                let tool = request.available_tools.iter()
                    .find(|t| t.name == tool_name)
                    .ok_or_else(|| AxiomError::ChainExecution(format!("Tool not found: {}", tool_name)))?;
                
                // Extract arguments (simplified)
                let mut arguments = HashMap::new();
                arguments.insert("input".to_string(), serde_json::Value::String(params.to_string()));
                
                Ok(PlanAction::CallTool {
                    tool_name: tool_name.to_string(),
                    arguments,
                })
            }
            "update_memory" => {
                Ok(PlanAction::UpdateMemory {
                    content: params.to_string(),
                    memory_type: MemoryType::LongTerm,
                })
            }
            "retrieve_memory" => {
                Ok(PlanAction::RetrieveMemory {
                    query: params.to_string(),
                    limit: 5,
                })
            }
            _ => {
                Err(AxiomError::ChainExecution(format!("Unknown action type: {}", action_type)))
            }
        }
    }
}
