//! Axiom LLM - Vendor-agnostic LLM gateway and provider implementations
//!
//! This crate provides a unified interface for working with different LLM providers,
//! with built-in retry logic, rate limiting, and monitoring.

pub mod gateway;
pub mod providers;
pub mod retry;
pub mod rate_limit;
pub mod monitoring;

pub use gateway::*;
pub use providers::*;
pub use retry::*;
pub use rate_limit::*;
pub use monitoring::*;

/// Re-exports commonly used types for convenience
pub mod prelude {
    pub use crate::{
        LlmGateway, LlmProvider, LlmRequest, LlmResponse,
        RetryConfig, RateLimitConfig, LlmMetrics,
    };
    pub use axiom_core::prelude::*;
}
