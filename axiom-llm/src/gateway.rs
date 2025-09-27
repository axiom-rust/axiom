//! LLM Gateway - Unified interface for all LLM providers

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use axiom_ai_core::{Message, StreamResponse, Result, AxiomError, ChainInput, ChainOutput};
use crate::providers::LlmProvider;
use crate::retry::RetryConfig;
use crate::rate_limit::RateLimitConfig;
use crate::monitoring::LlmMetrics;

/// Main LLM Gateway that provides a unified interface to all providers
pub struct LlmGateway {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    default_provider: String,
    retry_config: RetryConfig,
    rate_limit_config: RateLimitConfig,
    metrics: Arc<RwLock<LlmMetrics>>,
}

impl LlmGateway {
    /// Create a new LLM Gateway
    pub fn new(default_provider: impl Into<String>) -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: default_provider.into(),
            retry_config: RetryConfig::default(),
            rate_limit_config: RateLimitConfig::default(),
            metrics: Arc::new(RwLock::new(LlmMetrics::new())),
        }
    }

    /// Add a provider to the gateway
    pub fn add_provider(mut self, name: impl Into<String>, provider: Arc<dyn LlmProvider>) -> Self {
        self.providers.insert(name.into(), provider);
        self
    }

    /// Set the default provider
    pub fn set_default_provider(mut self, name: impl Into<String>) -> Self {
        self.default_provider = name.into();
        self
    }

    /// Set retry configuration
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Set rate limiting configuration
    pub fn with_rate_limit_config(mut self, config: RateLimitConfig) -> Self {
        self.rate_limit_config = config;
        self
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<&Arc<dyn LlmProvider>> {
        self.providers.get(name)
    }

    /// Get the default provider
    pub fn default_provider(&self) -> Option<&Arc<dyn LlmProvider>> {
        self.providers.get(&self.default_provider)
    }

    /// List all available providers
    pub fn list_providers(&self) -> Vec<&String> {
        self.providers.keys().collect()
    }

    /// Generate a completion using the default provider
    pub async fn generate(&self, request: LlmRequest) -> Result<LlmResponse> {
        self.generate_with_provider(&self.default_provider, request).await
    }

    /// Generate a completion using a specific provider
    pub async fn generate_with_provider(
        &self,
        provider_name: &str,
        request: LlmRequest,
    ) -> Result<LlmResponse> {
        let provider = self
            .get_provider(provider_name)
            .ok_or_else(|| AxiomError::LlmProvider(format!("Provider not found: {}", provider_name)))?;

        // Apply rate limiting
        self.rate_limit_config.check_rate_limit(provider_name).await?;

        // Execute with retry logic
        let result = self.retry_config.execute_with_retry(|| {
            provider.generate(request.clone())
        }).await?;

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_request(provider_name, &request, &result);
        }

        Ok(result)
    }

    /// Generate a streaming completion using the default provider
    pub async fn generate_stream(&self, request: LlmRequest) -> Result<StreamResponse> {
        self.generate_stream_with_provider(&self.default_provider, request).await
    }

    /// Generate a streaming completion using a specific provider
    pub async fn generate_stream_with_provider(
        &self,
        provider_name: &str,
        request: LlmRequest,
    ) -> Result<StreamResponse> {
        let provider = self
            .get_provider(provider_name)
            .ok_or_else(|| AxiomError::LlmProvider(format!("Provider not found: {}", provider_name)))?;

        // Apply rate limiting
        self.rate_limit_config.check_rate_limit(provider_name).await?;

        // Execute with retry logic
        let result = self.retry_config.execute_with_retry(|| {
            provider.generate_stream(request.clone())
        }).await?;

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_stream_request(provider_name, &request);
        }

        Ok(result)
    }

    /// Get metrics for all providers
    pub async fn get_metrics(&self) -> LlmMetrics {
        self.metrics.read().await.clone()
    }

    /// Get metrics for a specific provider
    pub async fn get_provider_metrics(&self, provider_name: &str) -> Option<LlmMetrics> {
        let metrics = self.metrics.read().await;
        if metrics.provider_metrics.contains_key(provider_name) {
            Some(metrics.clone())
        } else {
            None
        }
    }
}

/// Request to an LLM provider
#[derive(Debug, Clone)]
pub struct LlmRequest {
    /// Messages in the conversation
    pub messages: Vec<Message>,
    /// Model to use
    pub model: String,
    /// Temperature for generation
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Top-p sampling parameter
    pub top_p: Option<f32>,
    /// Top-k sampling parameter
    pub top_k: Option<u32>,
    /// Presence penalty
    pub presence_penalty: Option<f32>,
    /// Frequency penalty
    pub frequency_penalty: Option<f32>,
    /// Stop sequences
    pub stop_sequences: Vec<String>,
    /// Whether to stream the response
    pub stream: bool,
    /// Additional parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

impl LlmRequest {
    /// Create a new LLM request
    pub fn new(messages: Vec<Message>, model: impl Into<String>) -> Self {
        Self {
            messages,
            model: model.into(),
            temperature: None,
            max_tokens: None,
            top_p: None,
            top_k: None,
            presence_penalty: None,
            frequency_penalty: None,
            stop_sequences: Vec::new(),
            stream: false,
            parameters: HashMap::new(),
        }
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the maximum tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set the top-p parameter
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set the top-k parameter
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Set presence penalty
    pub fn with_presence_penalty(mut self, penalty: f32) -> Self {
        self.presence_penalty = Some(penalty);
        self
    }

    /// Set frequency penalty
    pub fn with_frequency_penalty(mut self, penalty: f32) -> Self {
        self.frequency_penalty = Some(penalty);
        self
    }

    /// Add stop sequences
    pub fn with_stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.stop_sequences = sequences;
        self
    }

    /// Enable streaming
    pub fn with_streaming(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Add a custom parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.parameters.insert(key.into(), value);
        self
    }

    /// Convert from chain input
    pub fn from_chain_input(input: ChainInput) -> Self {
        Self {
            messages: input.messages,
            model: "gpt-3.5-turbo".to_string(), // Default model
            temperature: input.config.temperature,
            max_tokens: input.config.max_tokens,
            top_p: input.config.top_p,
            top_k: None,
            presence_penalty: None,
            frequency_penalty: None,
            stop_sequences: Vec::new(),
            stream: input.config.stream,
            parameters: input.config.parameters,
        }
    }
}

/// Response from an LLM provider
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// Generated content
    pub content: String,
    /// Model used for generation
    pub model: String,
    /// Provider used
    pub provider: String,
    /// Number of tokens used
    pub tokens_used: Option<u32>,
    /// Cost of the request
    pub cost: Option<f64>,
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl LlmResponse {
    /// Create a new LLM response
    pub fn new(content: impl Into<String>, model: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            model: model.into(),
            provider: provider.into(),
            tokens_used: None,
            cost: None,
            generation_time_ms: 0,
            metadata: HashMap::new(),
        }
    }

    /// Set the number of tokens used
    pub fn with_tokens_used(mut self, tokens: u32) -> Self {
        self.tokens_used = Some(tokens);
        self
    }

    /// Set the cost
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost = Some(cost);
        self
    }

    /// Set the generation time
    pub fn with_generation_time(mut self, time_ms: u64) -> Self {
        self.generation_time_ms = time_ms;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Convert to chain output
    pub fn to_chain_output(self) -> ChainOutput {
        ChainOutput {
            messages: vec![axiom_ai_core::Message::assistant(self.content)],
            data: self.metadata,
            metadata: axiom_ai_core::ChainMetadata {
                execution_time_ms: self.generation_time_ms,
                tokens_used: self.tokens_used,
                cost: self.cost,
                data: HashMap::new(),
            },
        }
    }
}
