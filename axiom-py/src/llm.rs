//! LLM providers and functionality exposed to Python

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::sync::Arc;

use axiom_llm::{
    LlmGateway as RustLlmGateway, LlmProvider, LlmRequest as RustLlmRequest,
    LlmResponse as RustLlmResponse, ProviderConfig as RustProviderConfig
};
use axiom_llm::providers::{OpenAIProvider as RustOpenAIProvider, AnthropicProvider as RustAnthropicProvider};

/// Python wrapper for LlmGateway
#[pyclass(name = "LlmGateway")]
pub struct LlmGateway {
    inner: Arc<RustLlmGateway>,
}

#[pymethods]
impl LlmGateway {
    /// Create a new LLM gateway
    #[new]
    fn new(default_provider: &str) -> Self {
        Self {
            inner: Arc::new(RustLlmGateway::new(default_provider)),
        }
    }

    /// Add a provider to the gateway
    fn add_provider(&mut self, name: String, provider: Box<dyn LlmProvider>) -> PyResult<()> {
        // This would need to be implemented based on the actual LlmGateway API
        Ok(())
    }

    /// Generate a response using the LLM
    fn generate(&self, request: LlmRequest) -> PyResult<LlmResponse> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let response = rt.block_on(async {
            self.inner.generate(request.into()).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(response.into())
    }

    /// Generate a streaming response
    fn generate_stream(&self, request: LlmRequest) -> PyResult<StreamResponse> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let response = rt.block_on(async {
            self.inner.generate_stream(request.into()).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(response.into())
    }
}

impl From<Arc<RustLlmGateway>> for LlmGateway {
    fn from(gateway: Arc<RustLlmGateway>) -> Self {
        Self { inner: gateway }
    }
}

impl From<LlmGateway> for Arc<RustLlmGateway> {
    fn from(gateway: LlmGateway) -> Self {
        gateway.inner
    }
}

/// Python wrapper for LlmRequest
#[pyclass(name = "LlmRequest")]
pub struct LlmRequest {
    inner: RustLlmRequest,
}

#[pymethods]
impl LlmRequest {
    /// Create a new LLM request
    #[new]
    fn new(messages: Vec<crate::core::Message>, model: String) -> Self {
        let rust_messages: Vec<axiom_core::Message> = messages.into_iter().map(|m| m.into()).collect();
        Self {
            inner: RustLlmRequest::new(rust_messages, model),
        }
    }

    /// Set the temperature
    fn with_temperature(mut self_: PyRef<Self>, temperature: f32) -> PyRef<Self> {
        self_.inner = self_.inner.with_temperature(temperature);
        self_
    }

    /// Set the maximum tokens
    fn with_max_tokens(mut self_: PyRef<Self>, max_tokens: u32) -> PyRef<Self> {
        self_.inner = self_.inner.with_max_tokens(max_tokens);
        self_
    }

    /// Set the top_p parameter
    fn with_top_p(mut self_: PyRef<Self>, top_p: f32) -> PyRef<Self> {
        self_.inner = self_.inner.with_top_p(top_p);
        self_
    }

    /// Set the frequency penalty
    fn with_frequency_penalty(mut self_: PyRef<Self>, penalty: f32) -> PyRef<Self> {
        self_.inner = self_.inner.with_frequency_penalty(penalty);
        self_
    }

    /// Set the presence penalty
    fn with_presence_penalty(mut self_: PyRef<Self>, penalty: f32) -> PyRef<Self> {
        self_.inner = self_.inner.with_presence_penalty(penalty);
        self_
    }

    /// Set the stop sequences
    fn with_stop_sequences(mut self_: PyRef<Self>, stop: Vec<String>) -> PyRef<Self> {
        self_.inner = self_.inner.with_stop_sequences(stop);
        self_
    }

    /// Get the model name
    fn model(&self) -> String {
        self.inner.model().to_string()
    }

    /// Get the temperature
    fn temperature(&self) -> Option<f32> {
        self.inner.temperature()
    }

    /// Get the maximum tokens
    fn max_tokens(&self) -> Option<u32> {
        self.inner.max_tokens()
    }
}

impl From<RustLlmRequest> for LlmRequest {
    fn from(request: RustLlmRequest) -> Self {
        Self { inner: request }
    }
}

impl From<LlmRequest> for RustLlmRequest {
    fn from(request: LlmRequest) -> Self {
        request.inner
    }
}

/// Python wrapper for LlmResponse
#[pyclass(name = "LlmResponse")]
pub struct LlmResponse {
    inner: RustLlmResponse,
}

#[pymethods]
impl LlmResponse {
    /// Get the response content
    fn content(&self) -> String {
        self.inner.content().to_string()
    }

    /// Get the model used
    fn model(&self) -> String {
        self.inner.model().to_string()
    }

    /// Get the usage information
    fn usage(&self) -> Option<UsageInfo> {
        self.inner.usage().map(|u| u.into())
    }

    /// Get the finish reason
    fn finish_reason(&self) -> Option<String> {
        self.inner.finish_reason().map(|r| r.to_string())
    }

    /// Get the response metadata
    fn metadata(&self) -> HashMap<String, String> {
        self.inner
            .metadata()
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect()
    }
}

impl From<RustLlmResponse> for LlmResponse {
    fn from(response: RustLlmResponse) -> Self {
        Self { inner: response }
    }
}

/// Python wrapper for UsageInfo
#[pyclass(name = "UsageInfo")]
pub struct UsageInfo {
    inner: axiom_llm::UsageInfo,
}

#[pymethods]
impl UsageInfo {
    /// Get prompt tokens
    fn prompt_tokens(&self) -> u32 {
        self.inner.prompt_tokens
    }

    /// Get completion tokens
    fn completion_tokens(&self) -> u32 {
        self.inner.completion_tokens
    }

    /// Get total tokens
    fn total_tokens(&self) -> u32 {
        self.inner.total_tokens
    }
}

impl From<axiom_llm::UsageInfo> for UsageInfo {
    fn from(usage: axiom_llm::UsageInfo) -> Self {
        Self { inner: usage }
    }
}

/// Python wrapper for OpenAIProvider
#[pyclass(name = "OpenAIProvider")]
pub struct OpenAIProvider {
    inner: RustOpenAIProvider,
}

#[pymethods]
impl OpenAIProvider {
    /// Create a new OpenAI provider
    #[new]
    fn new(config: ProviderConfig) -> PyResult<Self> {
        let provider = RustOpenAIProvider::new(config.into())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { inner: provider })
    }

    /// Get the provider name
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Check if the provider is available
    fn is_available(&self) -> bool {
        self.inner.is_available()
    }
}

impl From<RustOpenAIProvider> for OpenAIProvider {
    fn from(provider: RustOpenAIProvider) -> Self {
        Self { inner: provider }
    }
}

impl From<OpenAIProvider> for RustOpenAIProvider {
    fn from(provider: OpenAIProvider) -> Self {
        provider.inner
    }
}

/// Python wrapper for AnthropicProvider
#[pyclass(name = "AnthropicProvider")]
pub struct AnthropicProvider {
    inner: RustAnthropicProvider,
}

#[pymethods]
impl AnthropicProvider {
    /// Create a new Anthropic provider
    #[new]
    fn new(config: ProviderConfig) -> PyResult<Self> {
        let provider = RustAnthropicProvider::new(config.into())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { inner: provider })
    }

    /// Get the provider name
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Check if the provider is available
    fn is_available(&self) -> bool {
        self.inner.is_available()
    }
}

impl From<RustAnthropicProvider> for AnthropicProvider {
    fn from(provider: RustAnthropicProvider) -> Self {
        Self { inner: provider }
    }
}

impl From<AnthropicProvider> for RustAnthropicProvider {
    fn from(provider: AnthropicProvider) -> Self {
        provider.inner
    }
}

/// Python wrapper for ProviderConfig
#[pyclass(name = "ProviderConfig")]
pub struct ProviderConfig {
    inner: RustProviderConfig,
}

#[pymethods]
impl ProviderConfig {
    /// Create a new provider configuration
    #[new]
    fn new() -> Self {
        Self {
            inner: RustProviderConfig::new(),
        }
    }

    /// Set the API key
    fn with_api_key(mut self_: PyRef<Self>, api_key: &str) -> PyRef<Self> {
        self_.inner = self_.inner.with_api_key(api_key);
        self_
    }

    /// Set the base URL
    fn with_base_url(mut self_: PyRef<Self>, base_url: &str) -> PyRef<Self> {
        self_.inner = self_.inner.with_base_url(base_url);
        self_
    }

    /// Set the timeout
    fn with_timeout(mut self_: PyRef<Self>, timeout: u64) -> PyRef<Self> {
        self_.inner = self_.inner.with_timeout(timeout);
        self_
    }

    /// Set the retry configuration
    fn with_retry_config(mut self_: PyRef<Self>, max_retries: u32) -> PyRef<Self> {
        self_.inner = self_.inner.with_retry_config(max_retries);
        self_
    }

    /// Set the rate limit
    fn with_rate_limit(mut self_: PyRef<Self>, requests_per_minute: u32) -> PyRef<Self> {
        self_.inner = self_.inner.with_rate_limit(requests_per_minute);
        self_
    }

    /// Get the API key
    fn api_key(&self) -> Option<String> {
        self.inner.api_key().map(|k| k.to_string())
    }

    /// Get the base URL
    fn base_url(&self) -> Option<String> {
        self.inner.base_url().map(|u| u.to_string())
    }

    /// Get the timeout
    fn timeout(&self) -> u64 {
        self.inner.timeout()
    }
}

impl From<RustProviderConfig> for ProviderConfig {
    fn from(config: RustProviderConfig) -> Self {
        Self { inner: config }
    }
}

impl From<ProviderConfig> for RustProviderConfig {
    fn from(config: ProviderConfig) -> Self {
        config.inner
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

// Trait for LLM providers
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn generate(&self, request: RustLlmRequest) -> PyResult<RustLlmResponse>;
    fn generate_stream(&self, request: RustLlmRequest) -> PyResult<axiom_core::StreamResponse>;
}
