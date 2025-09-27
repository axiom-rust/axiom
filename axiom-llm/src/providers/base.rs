//! Base LLM Provider trait and common functionality

use async_trait::async_trait;
use std::collections::HashMap;

use axiom_ai_core::{Message, StreamResponse, Result, AxiomError};
use crate::gateway::{LlmRequest, LlmResponse};

/// Trait that all LLM providers must implement
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the name of the provider
    fn name(&self) -> &str;

    /// Get the description of the provider
    fn description(&self) -> &str;

    /// Get the list of supported models
    fn supported_models(&self) -> Vec<String>;

    /// Check if a model is supported
    fn supports_model(&self, model: &str) -> bool {
        self.supported_models().contains(&model.to_string())
    }

    /// Generate a completion
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse>;

    /// Generate a streaming completion
    async fn generate_stream(&self, request: LlmRequest) -> Result<StreamResponse>;

    /// Get the cost per token for a model (in USD)
    fn get_cost_per_token(&self, model: &str) -> Option<f64>;

    /// Get the maximum context length for a model
    fn get_max_context_length(&self, model: &str) -> Option<usize>;

    /// Validate a request before sending
    fn validate_request(&self, request: &LlmRequest) -> Result<()> {
        // Check if model is supported
        if !self.supports_model(&request.model) {
            return Err(AxiomError::LlmProvider(format!(
                "Model '{}' is not supported by provider '{}'",
                request.model,
                self.name()
            )));
        }

        // Check context length
        if let Some(max_length) = self.get_max_context_length(&request.model) {
            let total_tokens = estimate_tokens(&request.messages);
            if total_tokens > max_length {
                return Err(AxiomError::LlmProvider(format!(
                    "Request exceeds maximum context length: {} > {}",
                    total_tokens, max_length
                )));
            }
        }

        // Validate parameters
        if let Some(temperature) = request.temperature {
            if temperature < 0.0 || temperature > 2.0 {
                return Err(AxiomError::LlmProvider(
                    "Temperature must be between 0.0 and 2.0".to_string()
                ));
            }
        }

        if let Some(top_p) = request.top_p {
            if top_p < 0.0 || top_p > 1.0 {
                return Err(AxiomError::LlmProvider(
                    "Top-p must be between 0.0 and 1.0".to_string()
                ));
            }
        }

        if let Some(max_tokens) = request.max_tokens {
            if max_tokens == 0 {
                return Err(AxiomError::LlmProvider(
                    "Max tokens must be greater than 0".to_string()
                ));
            }
        }

        Ok(())
    }
}

/// Estimate the number of tokens in a list of messages
pub fn estimate_tokens(messages: &[Message]) -> usize {
    // Simple token estimation - in practice, you'd use a proper tokenizer
    let mut total = 0;
    for message in messages {
        total += estimate_message_tokens(message);
    }
    total
}

/// Estimate the number of tokens in a single message
pub fn estimate_message_tokens(message: &Message) -> usize {
    let content = match &message.content {
        axiom_ai_core::MessageContent::Text(text) => text,
        axiom_ai_core::MessageContent::Parts(parts) => {
            // Estimate tokens for all parts
            return parts.iter().map(estimate_part_tokens).sum();
        }
    };

    // Rough estimation: 1 token ≈ 4 characters for English text
    // Add overhead for role and formatting
    let base_tokens = (content.len() + 3) / 4;
    base_tokens + 10 // Add overhead for role and formatting
}

/// Estimate tokens for a message part
pub fn estimate_part_tokens(part: &axiom_ai_core::MessagePart) -> usize {
    match part {
        axiom_ai_core::MessagePart::Text(text) => (text.len() + 3) / 4 + 5,
        axiom_ai_core::MessagePart::Image { data, .. } => {
            // Images are typically represented as base64, estimate based on size
            (data.len() + 3) / 4 + 20
        }
        axiom_ai_core::MessagePart::ToolCall { name, arguments, .. } => {
            let args_str = serde_json::to_string(arguments).unwrap_or_default();
            (name.len() + args_str.len() + 3) / 4 + 15
        }
        axiom_ai_core::MessagePart::ToolResult { content, .. } => {
            (content.len() + 3) / 4 + 10
        }
    }
}

/// Common provider configuration
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// API key for the provider
    pub api_key: Option<String>,
    /// Base URL for the provider
    pub base_url: Option<String>,
    /// Default model for this provider
    pub default_model: Option<String>,
    /// Request timeout in seconds
    pub timeout_seconds: Option<u64>,
    /// Additional provider-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

impl ProviderConfig {
    /// Create a new provider configuration
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: None,
            default_model: None,
            timeout_seconds: Some(30),
            settings: HashMap::new(),
        }
    }

    /// Set the API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the default model
    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = Some(model.into());
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }

    /// Add a custom setting
    pub fn with_setting(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.settings.insert(key.into(), value);
        self
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create HTTP client with common configuration
pub fn create_http_client(config: &ProviderConfig) -> Result<reqwest::Client> {
    let mut client_builder = reqwest::Client::builder();

    // Set timeout
    if let Some(timeout) = config.timeout_seconds {
        client_builder = client_builder.timeout(std::time::Duration::from_secs(timeout));
    }

    // Set default headers
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );

    if let Some(api_key) = &config.api_key {
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", api_key).parse().unwrap(),
        );
    }

    client_builder
        .default_headers(headers)
        .build()
        .map_err(|e| AxiomError::LlmProvider(format!("Failed to create HTTP client: {}", e)))
}

/// Common error handling for provider responses
pub fn handle_provider_error(status: reqwest::StatusCode, body: &str) -> AxiomError {
    match status {
        reqwest::StatusCode::UNAUTHORIZED => {
            AxiomError::Authentication("Invalid API key or authentication failed".to_string())
        }
        reqwest::StatusCode::FORBIDDEN => {
            AxiomError::Authentication("API key does not have permission to access this resource".to_string())
        }
        reqwest::StatusCode::TOO_MANY_REQUESTS => {
            AxiomError::RateLimit("Rate limit exceeded".to_string())
        }
        reqwest::StatusCode::BAD_REQUEST => {
            AxiomError::LlmProvider(format!("Bad request: {}", body))
        }
        reqwest::StatusCode::NOT_FOUND => {
            AxiomError::LlmProvider("Resource not found".to_string())
        }
        reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
            AxiomError::LlmProvider("Internal server error from provider".to_string())
        }
        reqwest::StatusCode::SERVICE_UNAVAILABLE => {
            AxiomError::LlmProvider("Service temporarily unavailable".to_string())
        }
        _ => {
            AxiomError::LlmProvider(format!("Unexpected error: {} - {}", status, body))
        }
    }
}
