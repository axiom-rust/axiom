//! Axiom Monitoring - Observability, metrics, and monitoring

pub mod metrics;
pub mod tracing;
pub mod health;
pub mod alerts;

pub use metrics::*;
pub use tracing::*;
pub use health::*;
pub use alerts::*;

/// Re-exports commonly used types for convenience
pub mod prelude {
    pub use crate::{
        MetricsCollector, TracingConfig, HealthChecker, AlertManager,
    };
    pub use axiom_core::prelude::*;
}
