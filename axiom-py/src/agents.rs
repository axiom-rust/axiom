//! Agent functionality exposed to Python

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::sync::Arc;

use axiom_agents::{
    Agent as RustAgent, AgentConfig as RustAgentConfig, 
    AgentResult as RustAgentResult, AgentState as RustAgentState
};
use axiom_core::{Message, Memory, MemoryType, Tool, ToolResult, ToolParameters};
use axiom_llm::{LlmGateway, LlmRequest, LlmResponse};

use crate::core::{Message as PyMessage, MessageRole, MessageContent};
use crate::llm::{LlmGateway as PyLlmGateway, ProviderConfig};

/// Python wrapper for Agent
#[pyclass(name = "Agent")]
pub struct Agent {
    inner: RustAgent,
}

#[pymethods]
impl Agent {
    /// Create a new agent
    #[new]
    fn new(
        llm_gateway: PyLlmGateway,
        memory: Box<dyn Memory>,
        planner: Box<dyn Planner>,
        executor: Box<dyn Executor>,
        safety_guard: Box<dyn SafetyGuard>,
        config: AgentConfig,
    ) -> PyResult<Self> {
        let agent = RustAgent::new(
            llm_gateway.into(),
            memory,
            planner.into(),
            executor.into(),
            safety_guard.into(),
            config.into(),
        );
        Ok(Self { inner: agent })
    }

    /// Add a tool to the agent
    fn add_tool(mut self_: PyRef<Self>, tool: Box<dyn Tool>) -> PyRef<Self> {
        self_.inner = self_.inner.add_tool(tool);
        self_
    }

    /// Process a user message and generate a response
    fn process_message(&mut self, message: PyMessage) -> PyResult<AgentResult> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let result = rt.block_on(async {
            self.inner.process_message(message.into()).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(result.into())
    }

    /// Process a message and return a streaming response
    fn process_message_stream(&mut self, message: PyMessage) -> PyResult<StreamResponse> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let result = rt.block_on(async {
            self.inner.process_message_stream(message.into()).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(result.into())
    }

    /// Get the current agent state
    fn get_state(&self) -> AgentState {
        self.inner.get_state().clone().into()
    }

    /// Update agent configuration
    fn update_config(&mut self, config: AgentConfig) {
        self.inner.update_config(config.into());
    }

    /// Clear conversation history
    fn clear_conversation(&mut self) -> PyResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        rt.block_on(async {
            self.inner.clear_conversation().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(())
    }
}

impl From<PyLlmGateway> for Arc<LlmGateway> {
    fn from(gateway: PyLlmGateway) -> Self {
        gateway.into()
    }
}

/// Python wrapper for AgentConfig
#[pyclass(name = "AgentConfig")]
#[derive(Clone)]
pub struct AgentConfig {
    inner: RustAgentConfig,
}

#[pymethods]
impl AgentConfig {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustAgentConfig::default(),
        }
    }

    /// Set maximum number of tool calls per turn
    fn max_tool_calls(mut self_: PyRef<Self>, max: u32) -> PyRef<Self> {
        self_.inner.max_tool_calls = max;
        self_
    }

    /// Set tool call timeout in seconds
    fn tool_call_timeout(mut self_: PyRef<Self>, timeout: u64) -> PyRef<Self> {
        self_.inner.tool_call_timeout = timeout;
        self_
    }

    /// Enable tool retries
    fn enable_tool_retries(mut self_: PyRef<Self>, enabled: bool) -> PyRef<Self> {
        self_.inner.enable_tool_retries = enabled;
        self_
    }

    /// Set maximum tool retries
    fn max_tool_retries(mut self_: PyRef<Self>, max: u32) -> PyRef<Self> {
        self_.inner.max_tool_retries = max;
        self_
    }

    /// Enable safety checks
    fn enable_safety_checks(mut self_: PyRef<Self>, enabled: bool) -> PyRef<Self> {
        self_.inner.enable_safety_checks = enabled;
        self_
    }

    /// Set safety check timeout in seconds
    fn safety_check_timeout(mut self_: PyRef<Self>, timeout: u64) -> PyRef<Self> {
        self_.inner.safety_check_timeout = timeout;
        self_
    }

    /// Set planning model
    fn planning_model(mut self_: PyRef<Self>, model: &str) -> PyRef<Self> {
        self_.inner.planning_model = model.to_string();
        self_
    }

    /// Set execution model
    fn execution_model(mut self_: PyRef<Self>, model: &str) -> PyRef<Self> {
        self_.inner.execution_model = model.to_string();
        self_
    }

    /// Set planning temperature
    fn planning_temperature(mut self_: PyRef<Self>, temp: f32) -> PyRef<Self> {
        self_.inner.planning_temperature = temp;
        self_
    }

    /// Set execution temperature
    fn execution_temperature(mut self_: PyRef<Self>, temp: f32) -> PyRef<Self> {
        self_.inner.execution_temperature = temp;
        self_
    }

    /// Set maximum context length
    fn max_context_length(mut self_: PyRef<Self>, length: usize) -> PyRef<Self> {
        self_.inner.max_context_length = length;
        self_
    }
}

impl From<RustAgentConfig> for AgentConfig {
    fn from(config: RustAgentConfig) -> Self {
        Self { inner: config }
    }
}

impl From<AgentConfig> for RustAgentConfig {
    fn from(config: AgentConfig) -> Self {
        config.inner
    }
}

/// Python wrapper for AgentResult
#[pyclass(name = "AgentResult")]
pub struct AgentResult {
    inner: RustAgentResult,
}

#[pymethods]
impl AgentResult {
    /// Get response messages
    fn messages(&self) -> Vec<PyMessage> {
        self.inner.messages.iter().map(|m| m.clone().into()).collect()
    }

    /// Get tool calls made
    fn tool_calls(&self) -> Vec<ToolCallRecord> {
        self.inner.tool_calls.iter().map(|tc| tc.clone().into()).collect()
    }

    /// Get execution metadata
    fn metadata(&self) -> HashMap<String, String> {
        self.inner
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect()
    }

    /// Get total execution time in milliseconds
    fn execution_time_ms(&self) -> u64 {
        self.inner.execution_time_ms
    }

    /// Check if the agent completed successfully
    fn success(&self) -> bool {
        self.inner.success
    }

    /// Get error message if failed
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
}

impl From<RustAgentResult> for AgentResult {
    fn from(result: RustAgentResult) -> Self {
        Self { inner: result }
    }
}

/// Python wrapper for AgentState
#[pyclass(name = "AgentState")]
pub struct AgentState {
    inner: RustAgentState,
}

#[pymethods]
impl AgentState {
    /// Get conversation messages
    fn conversation(&self) -> Vec<PyMessage> {
        self.inner.conversation.iter().map(|m| m.clone().into()).collect()
    }

    /// Get current step in the plan
    fn current_step(&self) -> usize {
        self.inner.current_step
    }

    /// Get tool call history
    fn tool_call_history(&self) -> Vec<ToolCallRecord> {
        self.inner.tool_call_history.iter().map(|tc| tc.clone().into()).collect()
    }

    /// Get agent metadata
    fn metadata(&self) -> HashMap<String, String> {
        self.inner
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect()
    }
}

impl From<RustAgentState> for AgentState {
    fn from(state: RustAgentState) -> Self {
        Self { inner: state }
    }
}

/// Python wrapper for ToolCallRecord
#[pyclass(name = "ToolCallRecord")]
pub struct ToolCallRecord {
    inner: axiom_agents::ToolCallRecord,
}

#[pymethods]
impl ToolCallRecord {
    /// Get tool name
    fn tool_name(&self) -> String {
        self.inner.tool_name.clone()
    }

    /// Get arguments passed to the tool
    fn arguments(&self) -> HashMap<String, String> {
        self.inner
            .arguments
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect()
    }

    /// Get result of the tool call
    fn result(&self) -> ToolResult {
        self.inner.result.clone().into()
    }

    /// Get timestamp of the call
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    /// Get execution time in milliseconds
    fn execution_time_ms(&self) -> u64 {
        self.inner.execution_time_ms
    }
}

impl From<axiom_agents::ToolCallRecord> for ToolCallRecord {
    fn from(record: axiom_agents::ToolCallRecord) -> Self {
        Self { inner: record }
    }
}

/// Python wrapper for StreamResponse
#[pyclass(name = "StreamResponse")]
pub struct StreamResponse {
    inner: axiom_core::StreamResponse,
}

#[pymethods]
impl StreamResponse {
    /// Get the next chunk from the stream
    fn next_chunk(&mut self) -> PyResult<Option<StreamChunk>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let chunk = rt.block_on(async {
            self.inner.next().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(chunk.map(|c| c.into()))
    }
}

impl From<axiom_core::StreamResponse> for StreamResponse {
    fn from(stream: axiom_core::StreamResponse) -> Self {
        Self { inner: stream }
    }
}

/// Python wrapper for StreamChunk
#[pyclass(name = "StreamChunk")]
pub struct StreamChunk {
    inner: axiom_core::StreamChunk,
}

#[pymethods]
impl StreamChunk {
    /// Get chunk content
    fn content(&self) -> String {
        self.inner.content.clone()
    }

    /// Get chunk type
    fn chunk_type(&self) -> String {
        format!("{:?}", self.inner.chunk_type)
    }

    /// Check if this is the final chunk
    fn is_final(&self) -> bool {
        self.inner.is_final
    }

    /// Get chunk metadata
    fn metadata(&self) -> Option<HashMap<String, String>> {
        self.inner.metadata.as_ref().map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect()
        })
    }
}

impl From<axiom_core::StreamChunk> for StreamChunk {
    fn from(chunk: axiom_core::StreamChunk) -> Self {
        Self { inner: chunk }
    }
}

// Trait implementations for Python compatibility
pub trait Planner: Send + Sync {
    fn create_plan(&self, request: PlanningRequest) -> PyResult<ExecutionPlan>;
}

pub trait Executor: Send + Sync {
    fn execute(&self, context: ExecutionContext) -> PyResult<StepResult>;
}

pub trait SafetyGuard: Send + Sync {
    fn check_tool_call(&self, tool_name: &str, arguments: &HashMap<String, serde_json::Value>) -> PyResult<SafetyResult>;
}

// Placeholder types for the traits
#[pyclass(name = "PlanningRequest")]
pub struct PlanningRequest;

#[pyclass(name = "ExecutionPlan")]
pub struct ExecutionPlan;

#[pyclass(name = "ExecutionContext")]
pub struct ExecutionContext;

#[pyclass(name = "StepResult")]
pub struct StepResult;

#[pyclass(name = "SafetyResult")]
pub struct SafetyResult;

// Implement the traits for the placeholder types
impl Planner for Box<dyn Planner> {
    fn create_plan(&self, _request: PlanningRequest) -> PyResult<ExecutionPlan> {
        Ok(ExecutionPlan)
    }
}

impl Executor for Box<dyn Executor> {
    fn execute(&self, _context: ExecutionContext) -> PyResult<StepResult> {
        Ok(StepResult)
    }
}

impl SafetyGuard for Box<dyn SafetyGuard> {
    fn check_tool_call(&self, _tool_name: &str, _arguments: &HashMap<String, serde_json::Value>) -> PyResult<SafetyResult> {
        Ok(SafetyResult)
    }
}
