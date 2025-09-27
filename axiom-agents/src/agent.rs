//! Main Agent implementation

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use axiom_core::{Message, StreamResponse, Result, AxiomError, Memory, MemoryType, Tool, ToolResult, Chain, ChainInput, ChainOutput};
use axiom_llm::{LlmGateway, LlmRequest, LlmResponse};
use crate::planner::Planner;
use crate::executor::Executor;
use crate::safety::SafetyGuard;

/// Main Agent that orchestrates planning, execution, and tool usage
pub struct Agent {
    /// LLM gateway for language model interactions
    llm_gateway: Arc<LlmGateway>,
    /// Memory system for context and conversation
    memory: Box<dyn Memory>,
    /// Tool registry for available tools
    tools: HashMap<String, Box<dyn Tool>>,
    /// Planner for creating execution plans
    planner: Box<dyn Planner>,
    /// Executor for running plans
    executor: Box<dyn Executor>,
    /// Safety guard for validation
    safety_guard: Box<dyn SafetyGuard>,
    /// Agent configuration
    config: AgentConfig,
    /// Current agent state
    state: AgentState,
}

/// Configuration for the agent
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Maximum number of tool calls per turn
    pub max_tool_calls: u32,
    /// Tool call timeout in seconds
    pub tool_call_timeout: u64,
    /// Enable tool retries
    pub enable_tool_retries: bool,
    /// Maximum tool retries
    pub max_tool_retries: u32,
    /// Enable safety checks
    pub enable_safety_checks: bool,
    /// Safety check timeout in seconds
    pub safety_check_timeout: u64,
    /// Model to use for planning
    pub planning_model: String,
    /// Model to use for execution
    pub execution_model: String,
    /// Temperature for planning
    pub planning_temperature: f32,
    /// Temperature for execution
    pub execution_temperature: f32,
    /// Maximum context length
    pub max_context_length: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_tool_calls: 10,
            tool_call_timeout: 30,
            enable_tool_retries: true,
            max_tool_retries: 3,
            enable_safety_checks: true,
            safety_check_timeout: 5,
            planning_model: "gpt-3.5-turbo".to_string(),
            execution_model: "gpt-3.5-turbo".to_string(),
            planning_temperature: 0.7,
            execution_temperature: 0.7,
            max_context_length: 4000,
        }
    }
}

/// Current state of the agent
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Current conversation
    pub conversation: Vec<Message>,
    /// Current execution plan
    pub current_plan: Option<ExecutionPlan>,
    /// Current step in the plan
    pub current_step: usize,
    /// Tool call history
    pub tool_call_history: Vec<ToolCallRecord>,
    /// Agent metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Record of a tool call
#[derive(Debug, Clone)]
pub struct ToolCallRecord {
    /// Tool name
    pub tool_name: String,
    /// Arguments passed to the tool
    pub arguments: HashMap<String, serde_json::Value>,
    /// Result of the tool call
    pub result: ToolResult,
    /// Timestamp of the call
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Result of agent execution
#[derive(Debug, Clone)]
pub struct AgentResult {
    /// Response messages
    pub messages: Vec<Message>,
    /// Tool calls made
    pub tool_calls: Vec<ToolCallRecord>,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether the agent completed successfully
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl Agent {
    /// Create a new agent
    pub fn new(
        llm_gateway: Arc<LlmGateway>,
        memory: Box<dyn Memory>,
        planner: Box<dyn Planner>,
        executor: Box<dyn Executor>,
        safety_guard: Box<dyn SafetyGuard>,
        config: AgentConfig,
    ) -> Self {
        Self {
            llm_gateway,
            memory,
            tools: HashMap::new(),
            planner,
            executor,
            safety_guard,
            config,
            state: AgentState {
                conversation: Vec::new(),
                current_plan: None,
                current_step: 0,
                tool_call_history: Vec::new(),
                metadata: HashMap::new(),
            },
        }
    }

    /// Add a tool to the agent
    pub fn add_tool(mut self, tool: Box<dyn Tool>) -> Self {
        self.tools.insert(tool.name().to_string(), tool);
        self
    }

    /// Process a user message and generate a response
    pub async fn process_message(&mut self, message: Message) -> Result<AgentResult> {
        let start_time = std::time::Instant::now();
        
        // Add message to conversation
        self.state.conversation.push(message.clone());
        
        // Store in memory
        self.memory.store(axiom_core::MemoryItem::new(
            message.text_content().unwrap_or("").to_string(),
            MemoryType::ShortTerm,
            0.5,
        )).await?;

        // Create execution plan
        let plan = self.create_execution_plan().await?;
        self.state.current_plan = Some(plan.clone());
        self.state.current_step = 0;

        // Execute the plan
        let result = self.execute_plan(plan).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(AgentResult {
            messages: result.messages,
            tool_calls: self.state.tool_call_history.clone(),
            metadata: result.metadata,
            execution_time_ms: execution_time,
            success: result.success,
            error: result.error,
        })
    }

    /// Process a message and return a streaming response
    pub async fn process_message_stream(&mut self, message: Message) -> Result<StreamResponse> {
        // Add message to conversation
        self.state.conversation.push(message.clone());
        
        // Store in memory
        self.memory.store(axiom_core::MemoryItem::new(
            message.text_content().unwrap_or("").to_string(),
            MemoryType::ShortTerm,
            0.5,
        )).await?;

        // Create execution plan
        let plan = self.create_execution_plan().await?;
        self.state.current_plan = Some(plan.clone());
        self.state.current_step = 0;

        // Execute the plan with streaming
        self.execute_plan_stream(plan).await
    }

    /// Create an execution plan for the current conversation
    async fn create_execution_plan(&self) -> Result<ExecutionPlan> {
        // Get relevant context from memory
        let context = self.get_relevant_context().await?;
        
        // Create planning request
        let planning_request = PlanningRequest {
            conversation: self.state.conversation.clone(),
            context,
            available_tools: self.get_available_tools(),
            config: self.config.clone(),
        };

        // Use planner to create plan
        self.planner.create_plan(planning_request).await
    }

    /// Execute a plan
    async fn execute_plan(&mut self, plan: ExecutionPlan) -> Result<AgentResult> {
        let mut messages = Vec::new();
        let mut tool_calls = Vec::new();
        let mut metadata = HashMap::new();

        for (step_index, step) in plan.steps.iter().enumerate() {
            self.state.current_step = step_index;
            
            // Execute the step
            let step_result = self.execute_step(step).await?;
            
            // Collect results
            messages.extend(step_result.messages);
            tool_calls.extend(step_result.tool_calls);
            metadata.extend(step_result.metadata);

            // Check if we should stop
            if step_result.should_stop {
                break;
            }
        }

        Ok(AgentResult {
            messages,
            tool_calls,
            metadata,
            execution_time_ms: 0, // Will be set by caller
            success: true,
            error: None,
        })
    }

    /// Execute a plan with streaming
    async fn execute_plan_stream(&mut self, plan: ExecutionPlan) -> Result<StreamResponse> {
        // For now, we'll execute the plan and then stream the final result
        // In a full implementation, you'd want to stream each step
        let result = self.execute_plan(plan).await?;
        
        // Convert result to stream
        let chunks = result.messages.into_iter().map(|msg| {
            axiom_core::StreamChunk {
                content: msg.text_content().unwrap_or("").to_string(),
                chunk_type: axiom_core::ChunkType::Text,
                metadata: None,
                is_final: true,
            }
        });

        let stream = tokio_stream::iter(chunks.into_iter().map(Ok));
        Ok(axiom_core::StreamResponse::new(stream))
    }

    /// Execute a single step in the plan
    async fn execute_step(&mut self, step: &PlanStep) -> Result<StepResult> {
        match &step.action {
            PlanAction::GenerateResponse { prompt } => {
                self.generate_response(prompt).await
            }
            PlanAction::CallTool { tool_name, arguments } => {
                self.call_tool(tool_name, arguments).await
            }
            PlanAction::UpdateMemory { content, memory_type } => {
                self.update_memory(content, *memory_type).await
            }
            PlanAction::RetrieveMemory { query, limit } => {
                self.retrieve_memory(query, *limit).await
            }
        }
    }

    /// Generate a response using the LLM
    async fn generate_response(&self, prompt: &str) -> Result<StepResult> {
        let request = LlmRequest::new(
            self.state.conversation.clone(),
            self.config.execution_model.clone(),
        ).with_temperature(self.config.execution_temperature);

        let response = self.llm_gateway.generate(request).await?;
        
        let message = Message::assistant(response.content);
        
        Ok(StepResult {
            messages: vec![message],
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
            should_stop: true,
        })
    }

    /// Call a tool
    async fn call_tool(&mut self, tool_name: &str, arguments: &HashMap<String, serde_json::Value>) -> Result<StepResult> {
        let tool = self.tools.get(tool_name)
            .ok_or_else(|| AxiomError::ToolExecution(format!("Tool not found: {}", tool_name)))?;

        let start_time = std::time::Instant::now();
        
        // Safety check
        if self.config.enable_safety_checks {
            let safety_result = self.safety_guard.check_tool_call(tool_name, arguments).await?;
            if !safety_result.is_safe {
                return Ok(StepResult {
                    messages: vec![Message::assistant(format!("Safety check failed: {}", safety_result.reason))],
                    tool_calls: Vec::new(),
                    metadata: HashMap::new(),
                    should_stop: true,
                });
            }
        }

        // Execute tool
        let result = tool.execute(arguments.clone()).await?;
        let execution_time = start_time.elapsed().as_millis() as u64;

        // Record tool call
        let tool_call_record = ToolCallRecord {
            tool_name: tool_name.to_string(),
            arguments: arguments.clone(),
            result: result.clone(),
            timestamp: chrono::Utc::now(),
            execution_time_ms: execution_time,
        };

        self.state.tool_call_history.push(tool_call_record.clone());

        // Create response message
        let response_message = if result.success {
            Message::assistant(format!("Tool '{}' executed successfully: {}", tool_name, result.content))
        } else {
            Message::assistant(format!("Tool '{}' failed: {}", tool_name, result.error.unwrap_or_default()))
        };

        Ok(StepResult {
            messages: vec![response_message],
            tool_calls: vec![tool_call_record],
            metadata: HashMap::new(),
            should_stop: false,
        })
    }

    /// Update memory
    async fn update_memory(&mut self, content: &str, memory_type: MemoryType) -> Result<StepResult> {
        let memory_item = axiom_core::MemoryItem::new(
            content.to_string(),
            memory_type,
            0.5,
        );

        self.memory.store(memory_item).await?;

        Ok(StepResult {
            messages: vec![Message::assistant("Memory updated".to_string())],
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
            should_stop: false,
        })
    }

    /// Retrieve memory
    async fn retrieve_memory(&mut self, query: &str, limit: usize) -> Result<StepResult> {
        let memories = self.memory.retrieve(query, Some(limit)).await?;
        
        let content = if memories.is_empty() {
            "No relevant memories found".to_string()
        } else {
            memories.iter()
                .map(|m| m.content.as_str())
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        Ok(StepResult {
            messages: vec![Message::assistant(content)],
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
            should_stop: false,
        })
    }

    /// Get relevant context from memory
    async fn get_relevant_context(&self) -> Result<Vec<axiom_core::MemoryItem>> {
        let query = self.state.conversation.last()
            .and_then(|msg| msg.text_content())
            .unwrap_or("");
        
        self.memory.retrieve(query, Some(10)).await
    }

    /// Get available tools
    fn get_available_tools(&self) -> Vec<ToolInfo> {
        self.tools.values()
            .map(|tool| ToolInfo {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameters(),
            })
            .collect()
    }

    /// Get the current agent state
    pub fn get_state(&self) -> &AgentState {
        &self.state
    }

    /// Update agent configuration
    pub fn update_config(&mut self, config: AgentConfig) {
        self.config = config;
    }

    /// Clear conversation history
    pub async fn clear_conversation(&mut self) -> Result<()> {
        self.state.conversation.clear();
        self.state.tool_call_history.clear();
        self.memory.clear().await?;
        Ok(())
    }
}

/// Result of executing a single step
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Messages generated by the step
    pub messages: Vec<Message>,
    /// Tool calls made in the step
    pub tool_calls: Vec<ToolCallRecord>,
    /// Metadata from the step
    pub metadata: HashMap<String, serde_json::Value>,
    /// Whether execution should stop
    pub should_stop: bool,
}

/// Information about an available tool
#[derive(Debug, Clone)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool parameters schema
    pub parameters: axiom_core::ToolParameters,
}

/// Request for planning
#[derive(Debug, Clone)]
pub struct PlanningRequest {
    /// Current conversation
    pub conversation: Vec<Message>,
    /// Relevant context from memory
    pub context: Vec<axiom_core::MemoryItem>,
    /// Available tools
    pub available_tools: Vec<ToolInfo>,
    /// Agent configuration
    pub config: AgentConfig,
}
