//! Execution system for agent plans

use async_trait::async_trait;
use std::collections::HashMap;

use axiom_core::{Message, Result, AxiomError, Tool, ToolResult};
use crate::planner::{PlanStep, PlanAction};
use crate::ToolCallRecord;

/// Result of executing a plan step
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub messages: Vec<Message>,
    pub tool_calls: Vec<ToolCallRecord>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub should_stop: bool,
}

impl StepResult {
    pub fn success(step_id: String, output: String, duration_ms: u64) -> Self {
        Self {
            step_id,
            success: true,
            output: Some(output),
            error: None,
            duration_ms,
            messages: Vec::new(),
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
            should_stop: true,
        }
    }

    pub fn failure(step_id: String, error: String, duration_ms: u64) -> Self {
        Self {
            step_id,
            success: false,
            output: None,
            error: Some(error),
            duration_ms,
            messages: Vec::new(),
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
            should_stop: false,
        }
    }
}

/// Trait for executing agent plans
#[async_trait]
pub trait Executor: Send + Sync {
    /// Execute a single plan step
    async fn execute_step(&self, step: &PlanStep, context: &ExecutionContext) -> Result<StepResult>;
    
    /// Execute multiple steps in parallel
    async fn execute_steps_parallel(&self, steps: &[PlanStep], context: &ExecutionContext) -> Result<Vec<StepResult>>;
}

/// Context for plan execution
pub struct ExecutionContext {
    /// Available tools
    pub tools: HashMap<String, Box<dyn Tool>>,
    /// Current conversation
    pub conversation: Vec<Message>,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Maximum execution time per step
    pub max_step_time: std::time::Duration,
    /// Enable parallel execution
    pub enable_parallel: bool,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            conversation: Vec::new(),
            metadata: HashMap::new(),
            max_step_time: std::time::Duration::from_secs(30),
            enable_parallel: false,
        }
    }

    /// Add a tool to the context
    pub fn add_tool(mut self, name: String, tool: Box<dyn Tool>) -> Self {
        self.tools.insert(name, tool);
        self
    }

    /// Set the conversation
    pub fn with_conversation(mut self, conversation: Vec<Message>) -> Self {
        self.conversation = conversation;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set maximum step time
    pub fn with_max_step_time(mut self, duration: std::time::Duration) -> Self {
        self.max_step_time = duration;
        self
    }

    /// Enable parallel execution
    pub fn with_parallel(mut self, enable: bool) -> Self {
        self.enable_parallel = enable;
        self
    }
}

/// Simple executor that executes steps sequentially
pub struct SimpleExecutor {
    /// LLM gateway for generating responses
    llm_gateway: std::sync::Arc<axiom_llm::LlmGateway>,
    /// Model to use for execution
    model: String,
}

impl SimpleExecutor {
    /// Create a new simple executor
    pub fn new(llm_gateway: std::sync::Arc<axiom_llm::LlmGateway>, model: String) -> Self {
        Self {
            llm_gateway,
            model,
        }
    }
}

#[async_trait]
impl Executor for SimpleExecutor {
    async fn execute_step(&self, step: &PlanStep, context: &ExecutionContext) -> Result<StepResult> {
        let start_time = std::time::Instant::now();
        
        let result = match &step.action {
            crate::planner::PlanAction::GenerateResponse { prompt } => {
                self.execute_generate_response(&prompt, context).await?
            }
            crate::planner::PlanAction::CallTool { tool_name, arguments } => {
                self.execute_call_tool(&tool_name, &arguments, context).await?
            }
            crate::planner::PlanAction::UpdateMemory { content, memory_type } => {
                self.execute_update_memory(&content, memory_type.clone(), context).await?
            }
            crate::planner::PlanAction::RetrieveMemory { query, limit } => {
                self.execute_retrieve_memory(&query, *limit, context).await?
            }
        };

        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(StepResult {
            step_id: step.id.clone(),
            success: result.success,
            output: result.output,
            error: result.error,
            duration_ms: execution_time,
            messages: result.messages,
            tool_calls: result.tool_calls,
            metadata: {
                let mut meta = result.metadata;
                meta.insert("execution_time_ms".to_string(), serde_json::Value::Number(execution_time.into()));
                meta
            },
            should_stop: result.should_stop,
        })
    }

    async fn execute_steps_parallel(&self, steps: &[PlanStep], context: &ExecutionContext) -> Result<Vec<StepResult>> {
        if !context.enable_parallel {
            // Execute sequentially
            let mut results = Vec::new();
            for step in steps {
                let result = self.execute_step(step, context).await?;
                results.push(result);
            }
            return Ok(results);
        }

        // Execute in parallel
        let futures: Vec<_> = steps.iter()
            .map(|step| self.execute_step(step, context))
            .collect();

        let results = futures::future::try_join_all(futures).await?;
        Ok(results)
    }
}

impl SimpleExecutor {
    /// Execute a generate response action
    async fn execute_generate_response(&self, prompt: &str, context: &ExecutionContext) -> Result<StepResult> {
        let start_time = std::time::Instant::now();
        let request = axiom_llm::LlmRequest::new(
            context.conversation.clone(),
            self.model.clone(),
        );

        let response = self.llm_gateway.generate(request).await?;
        
        let content = response.content.clone();
        let message = Message::assistant(content.clone());
        
        Ok(StepResult {
            step_id: "generate_response".to_string(),
            success: true,
            output: Some(content),
            error: None,
            duration_ms: start_time.elapsed().as_millis() as u64,
            messages: vec![message],
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
            should_stop: true,
        })
    }

    /// Execute a call tool action
    async fn execute_call_tool(&self, tool_name: &str, arguments: &HashMap<String, serde_json::Value>, context: &ExecutionContext) -> Result<StepResult> {
        let tool = context.tools.get(tool_name)
            .ok_or_else(|| AxiomError::ToolExecution(format!("Tool not found: {}", tool_name)))?;

        let start_time = std::time::Instant::now();
        let result = tool.execute(arguments.clone()).await?;
        let execution_time = start_time.elapsed().as_millis() as u64;

        let tool_call_record = ToolCallRecord {
            tool_name: tool_name.to_string(),
            arguments: arguments.clone(),
            result: result.clone(),
            timestamp: chrono::Utc::now(),
            execution_time_ms: execution_time,
        };

        let response_message = if result.success {
            Message::assistant(format!("Tool '{}' executed successfully: {}", tool_name, result.content))
        } else {
            Message::assistant(format!("Tool '{}' failed: {}", tool_name, result.error.as_ref().unwrap_or(&"Unknown error".to_string())))
        };

        Ok(StepResult {
            step_id: "call_tool".to_string(),
            success: result.success,
            output: Some(result.content),
            error: result.error,
            duration_ms: execution_time,
            messages: vec![response_message],
            tool_calls: vec![tool_call_record],
            metadata: HashMap::new(),
            should_stop: false,
        })
    }

    /// Execute an update memory action
    async fn execute_update_memory(&self, content: &str, memory_type: axiom_core::MemoryType, context: &ExecutionContext) -> Result<StepResult> {
        // This would typically interact with a memory system
        // For now, we'll just create a response message
        let message = Message::assistant(format!("Memory updated with: {}", content));
        
        Ok(StepResult {
            step_id: "update_memory".to_string(),
            success: true,
            output: Some(format!("Updated memory with content: {}", content)),
            error: None,
            duration_ms: 0,
            messages: vec![message],
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
            should_stop: false,
        })
    }

    /// Execute a retrieve memory action
    async fn execute_retrieve_memory(&self, query: &str, limit: usize, context: &ExecutionContext) -> Result<StepResult> {
        // This would typically interact with a memory system
        // For now, we'll just create a response message
        let message = Message::assistant(format!("Retrieved memories for query: {}", query));
        
        Ok(StepResult {
            step_id: "retrieve_memory".to_string(),
            success: true,
            output: Some(format!("Retrieved memories for query: {}", query)),
            error: None,
            duration_ms: 0,
            messages: vec![message],
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
            should_stop: false,
        })
    }
}

/// Advanced executor with retry logic and error handling
pub struct AdvancedExecutor {
    /// Base executor
    base_executor: SimpleExecutor,
    /// Maximum retries per step
    max_retries: u32,
    /// Retry delay
    retry_delay: std::time::Duration,
    /// Enable circuit breaker
    enable_circuit_breaker: bool,
}

impl AdvancedExecutor {
    /// Create a new advanced executor
    pub fn new(
        llm_gateway: std::sync::Arc<axiom_llm::LlmGateway>,
        model: String,
        max_retries: u32,
    ) -> Self {
        Self {
            base_executor: SimpleExecutor::new(llm_gateway, model),
            max_retries,
            retry_delay: std::time::Duration::from_millis(1000),
            enable_circuit_breaker: true,
        }
    }

    /// Set retry delay
    pub fn with_retry_delay(mut self, delay: std::time::Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Enable or disable circuit breaker
    pub fn with_circuit_breaker(mut self, enable: bool) -> Self {
        self.enable_circuit_breaker = enable;
        self
    }
}

#[async_trait]
impl Executor for AdvancedExecutor {
    async fn execute_step(&self, step: &PlanStep, context: &ExecutionContext) -> Result<StepResult> {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match self.base_executor.execute_step(step, context).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error.clone());
                    
                    // Check if error is retryable
                    if !error.is_retryable() || attempt >= self.max_retries {
                        break;
                    }
                    
                    // Wait before retry
                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AxiomError::Internal("Execution failed after all retries".to_string())
        }))
    }

    async fn execute_steps_parallel(&self, steps: &[PlanStep], context: &ExecutionContext) -> Result<Vec<StepResult>> {
        self.base_executor.execute_steps_parallel(steps, context).await
    }
}

/// Executor with monitoring and metrics
pub struct MonitoredExecutor {
    /// Base executor
    base_executor: Box<dyn Executor>,
    /// Metrics collector
    metrics: std::sync::Arc<tokio::sync::RwLock<ExecutorMetrics>>,
}

/// Metrics for executor performance
#[derive(Debug, Clone, Default)]
pub struct ExecutorMetrics {
    /// Total steps executed
    pub total_steps: u64,
    /// Successful steps
    pub successful_steps: u64,
    /// Failed steps
    pub failed_steps: u64,
    /// Total execution time
    pub total_execution_time_ms: u64,
    /// Average step execution time
    pub avg_step_time_ms: f64,
    /// Tool call metrics
    pub tool_call_metrics: HashMap<String, ToolCallMetrics>,
}

/// Metrics for tool calls
#[derive(Debug, Clone, Default)]
pub struct ToolCallMetrics {
    /// Total calls
    pub total_calls: u64,
    /// Successful calls
    pub successful_calls: u64,
    /// Failed calls
    pub failed_calls: u64,
    /// Total execution time
    pub total_execution_time_ms: u64,
    /// Average execution time
    pub avg_execution_time_ms: f64,
}

impl MonitoredExecutor {
    /// Create a new monitored executor
    pub fn new(base_executor: Box<dyn Executor>) -> Self {
        Self {
            base_executor,
            metrics: std::sync::Arc::new(tokio::sync::RwLock::new(ExecutorMetrics::default())),
        }
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> ExecutorMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = ExecutorMetrics::default();
    }
}

#[async_trait]
impl Executor for MonitoredExecutor {
    async fn execute_step(&self, step: &PlanStep, context: &ExecutionContext) -> Result<StepResult> {
        let start_time = std::time::Instant::now();
        
        let result = self.base_executor.execute_step(step, context).await;
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_steps += 1;
            metrics.total_execution_time_ms += execution_time;
            metrics.avg_step_time_ms = metrics.total_execution_time_ms as f64 / metrics.total_steps as f64;
            
            if result.is_ok() {
                metrics.successful_steps += 1;
            } else {
                metrics.failed_steps += 1;
            }
            
            // Update tool call metrics
            if let Ok(ref step_result) = result {
                for tool_call in &step_result.tool_calls {
                    let tool_metrics = metrics.tool_call_metrics
                        .entry(tool_call.tool_name.clone())
                        .or_insert_with(ToolCallMetrics::default);
                    
                    tool_metrics.total_calls += 1;
                    tool_metrics.total_execution_time_ms += tool_call.execution_time_ms;
                    tool_metrics.avg_execution_time_ms = tool_metrics.total_execution_time_ms as f64 / tool_metrics.total_calls as f64;
                    
                    if tool_call.result.success {
                        tool_metrics.successful_calls += 1;
                    } else {
                        tool_metrics.failed_calls += 1;
                    }
                }
            }
        }
        
        result
    }

    async fn execute_steps_parallel(&self, steps: &[PlanStep], context: &ExecutionContext) -> Result<Vec<StepResult>> {
        self.base_executor.execute_steps_parallel(steps, context).await
    }
}
