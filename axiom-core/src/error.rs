//! Error types and result handling for Axiom

use thiserror::Error;

/// Main error type for Axiom operations
#[derive(Error, Debug)]
pub enum AxiomError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("LLM provider error: {0}")]
    LlmProvider(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Chain execution error: {0}")]
    ChainExecution(String),

    #[error("WASM execution error: {0}")]
    WasmExecution(String),

    #[error("Monitoring error: {0}")]
    Monitoring(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenient result type alias
pub type Result<T> = std::result::Result<T, AxiomError>;

impl AxiomError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AxiomError::Network(_)
                | AxiomError::Timeout(_)
                | AxiomError::RateLimit(_)
                | AxiomError::LlmProvider(_)
        )
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AxiomError::Config(msg) => msg.clone(),
            AxiomError::LlmProvider(msg) => msg.clone(),
            AxiomError::Network(err) => err.to_string(),
            AxiomError::Serialization(err) => err.to_string(),
            AxiomError::Io(err) => err.to_string(),
            AxiomError::Timeout(msg) => msg.clone(),
            AxiomError::RateLimit(msg) => msg.clone(),
            AxiomError::Authentication(msg) => msg.clone(),
            AxiomError::Validation(msg) => msg.clone(),
            AxiomError::ToolExecution(msg) => msg.clone(),
            AxiomError::Memory(msg) => msg.clone(),
            AxiomError::ChainExecution(msg) => msg.clone(),
            AxiomError::WasmExecution(msg) => msg.clone(),
            AxiomError::Monitoring(msg) => msg.clone(),
            AxiomError::Internal(msg) => msg.clone(),
        }
    }
}

impl Clone for AxiomError {
    fn clone(&self) -> Self {
        match self {
            AxiomError::Config(msg) => AxiomError::Config(msg.clone()),
            AxiomError::LlmProvider(msg) => AxiomError::LlmProvider(msg.clone()),
            AxiomError::Network(_) => AxiomError::Config("Network error".to_string()),
            AxiomError::Serialization(_) => AxiomError::Config("Serialization error".to_string()),
            AxiomError::Io(_) => AxiomError::Config("IO error".to_string()),
            AxiomError::Timeout(msg) => AxiomError::Timeout(msg.clone()),
            AxiomError::RateLimit(msg) => AxiomError::RateLimit(msg.clone()),
            AxiomError::Authentication(msg) => AxiomError::Authentication(msg.clone()),
            AxiomError::Validation(msg) => AxiomError::Validation(msg.clone()),
            AxiomError::ToolExecution(msg) => AxiomError::ToolExecution(msg.clone()),
            AxiomError::Memory(msg) => AxiomError::Memory(msg.clone()),
            AxiomError::ChainExecution(msg) => AxiomError::ChainExecution(msg.clone()),
            AxiomError::WasmExecution(msg) => AxiomError::WasmExecution(msg.clone()),
            AxiomError::Monitoring(msg) => AxiomError::Monitoring(msg.clone()),
            AxiomError::Internal(msg) => AxiomError::Internal(msg.clone()),
        }
    }
}
