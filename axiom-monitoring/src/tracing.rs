//! Distributed tracing configuration

use std::collections::HashMap;

use axiom_core::{Result, AxiomError};

/// Tracing configuration
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Environment
    pub environment: String,
    /// Jaeger endpoint
    pub jaeger_endpoint: Option<String>,
    /// Zipkin endpoint
    pub zipkin_endpoint: Option<String>,
    /// OTLP endpoint
    pub otlp_endpoint: Option<String>,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Enable console output
    pub enable_console: bool,
    /// Log level
    pub log_level: String,
    /// Additional tags
    pub tags: HashMap<String, String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "axiom".to_string(),
            service_version: "0.1.0".to_string(),
            environment: "development".to_string(),
            jaeger_endpoint: None,
            zipkin_endpoint: None,
            otlp_endpoint: None,
            sampling_rate: 1.0,
            enable_console: true,
            log_level: "info".to_string(),
            tags: HashMap::new(),
        }
    }
}

impl TracingConfig {
    /// Create a new tracing configuration
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            ..Default::default()
        }
    }

    /// Set the service version
    pub fn with_version(mut self, version: String) -> Self {
        self.service_version = version;
        self
    }

    /// Set the environment
    pub fn with_environment(mut self, environment: String) -> Self {
        self.environment = environment;
        self
    }

    /// Set the Jaeger endpoint
    pub fn with_jaeger(mut self, endpoint: String) -> Self {
        self.jaeger_endpoint = Some(endpoint);
        self
    }

    /// Set the Zipkin endpoint
    pub fn with_zipkin(mut self, endpoint: String) -> Self {
        self.zipkin_endpoint = Some(endpoint);
        self
    }

    /// Set the OTLP endpoint
    pub fn with_otlp(mut self, endpoint: String) -> Self {
        self.otlp_endpoint = Some(endpoint);
        self
    }

    /// Set the sampling rate
    pub fn with_sampling_rate(mut self, rate: f64) -> Self {
        self.sampling_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Enable or disable console output
    pub fn with_console(mut self, enable: bool) -> Self {
        self.enable_console = enable;
        self
    }

    /// Set the log level
    pub fn with_log_level(mut self, level: String) -> Self {
        self.log_level = level;
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }

    /// Initialize tracing with this configuration
    pub fn init(&self) -> Result<()> {
        // Initialize tracing subscriber
        let subscriber = tracing_subscriber::fmt::Subscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| AxiomError::Monitoring(format!("Failed to set tracing subscriber: {}", e)))?;

        Ok(())
    }
}

/// Span context for distributed tracing
#[derive(Debug, Clone)]
pub struct SpanContext {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Parent span ID
    pub parent_span_id: Option<String>,
    /// Baggage items
    pub baggage: HashMap<String, String>,
}

impl SpanContext {
    /// Create a new span context
    pub fn new(trace_id: String, span_id: String) -> Self {
        Self {
            trace_id,
            span_id,
            parent_span_id: None,
            baggage: HashMap::new(),
        }
    }

    /// Create a child span context
    pub fn create_child(&self, span_id: String) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id,
            parent_span_id: Some(self.span_id.clone()),
            baggage: self.baggage.clone(),
        }
    }

    /// Add baggage item
    pub fn add_baggage(&mut self, key: String, value: String) {
        self.baggage.insert(key, value);
    }

    /// Get baggage item
    pub fn get_baggage(&self, key: &str) -> Option<&String> {
        self.baggage.get(key)
    }
}

/// Trace utilities
pub struct TraceUtils;

impl TraceUtils {
    /// Generate a new trace ID
    pub fn generate_trace_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Generate a new span ID
    pub fn generate_span_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Create a new span context
    pub fn create_span_context() -> SpanContext {
        SpanContext::new(
            Self::generate_trace_id(),
            Self::generate_span_id(),
        )
    }

    /// Extract span context from headers
    pub fn extract_from_headers(headers: &HashMap<String, String>) -> Option<SpanContext> {
        let trace_id = headers.get("x-trace-id")?.clone();
        let span_id = headers.get("x-span-id")?.clone();
        let parent_span_id = headers.get("x-parent-span-id").cloned();
        
        let mut context = SpanContext::new(trace_id, span_id);
        context.parent_span_id = parent_span_id;
        
        // Extract baggage items
        for (key, value) in headers {
            if key.starts_with("x-baggage-") {
                let baggage_key = key.strip_prefix("x-baggage-").unwrap().to_string();
                context.add_baggage(baggage_key, value.clone());
            }
        }
        
        Some(context)
    }

    /// Inject span context into headers
    pub fn inject_to_headers(context: &SpanContext, headers: &mut HashMap<String, String>) {
        headers.insert("x-trace-id".to_string(), context.trace_id.clone());
        headers.insert("x-span-id".to_string(), context.span_id.clone());
        
        if let Some(parent_span_id) = &context.parent_span_id {
            headers.insert("x-parent-span-id".to_string(), parent_span_id.clone());
        }
        
        for (key, value) in &context.baggage {
            headers.insert(format!("x-baggage-{}", key), value.clone());
        }
    }
}

/// Span builder for creating spans
pub struct SpanBuilder {
    /// Span name
    name: String,
    /// Span context
    context: Option<SpanContext>,
    /// Tags
    tags: HashMap<String, String>,
    /// Start time
    start_time: Option<std::time::SystemTime>,
}

impl SpanBuilder {
    /// Create a new span builder
    pub fn new(name: String) -> Self {
        Self {
            name,
            context: None,
            tags: HashMap::new(),
            start_time: None,
        }
    }

    /// Set the span context
    pub fn with_context(mut self, context: SpanContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }

    /// Set the start time
    pub fn with_start_time(mut self, start_time: std::time::SystemTime) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Build the span
    pub fn build(self) -> Span {
        Span {
            name: self.name,
            context: self.context.unwrap_or_else(TraceUtils::create_span_context),
            tags: self.tags,
            start_time: self.start_time.unwrap_or_else(std::time::SystemTime::now),
            end_time: None,
        }
    }
}

/// A tracing span
pub struct Span {
    /// Span name
    name: String,
    /// Span context
    context: SpanContext,
    /// Tags
    tags: HashMap<String, String>,
    /// Start time
    start_time: std::time::SystemTime,
    /// End time
    end_time: Option<std::time::SystemTime>,
}

impl Span {
    /// Create a new span builder
    pub fn builder(name: String) -> SpanBuilder {
        SpanBuilder::new(name)
    }

    /// Get the span name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the span context
    pub fn context(&self) -> &SpanContext {
        &self.context
    }

    /// Get the span duration
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.end_time?.duration_since(self.start_time).ok()
    }

    /// Finish the span
    pub fn finish(mut self) -> FinishedSpan {
        self.end_time = Some(std::time::SystemTime::now());
        FinishedSpan {
            name: self.name,
            context: self.context,
            tags: self.tags,
            start_time: self.start_time,
            end_time: self.end_time.unwrap(),
            duration: self.duration().unwrap(),
        }
    }
}

/// A finished span
pub struct FinishedSpan {
    /// Span name
    name: String,
    /// Span context
    context: SpanContext,
    /// Tags
    tags: HashMap<String, String>,
    /// Start time
    start_time: std::time::SystemTime,
    /// End time
    end_time: std::time::SystemTime,
    /// Duration
    duration: std::time::Duration,
}

impl FinishedSpan {
    /// Get the span name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the span context
    pub fn context(&self) -> &SpanContext {
        &self.context
    }

    /// Get the span duration
    pub fn duration(&self) -> std::time::Duration {
        self.duration
    }

    /// Get the span tags
    pub fn tags(&self) -> &HashMap<String, String> {
        &self.tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config() {
        let config = TracingConfig::new("test-service".to_string())
            .with_version("1.0.0".to_string())
            .with_environment("test".to_string())
            .with_sampling_rate(0.5);

        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.service_version, "1.0.0");
        assert_eq!(config.environment, "test");
        assert_eq!(config.sampling_rate, 0.5);
    }

    #[test]
    fn test_span_context() {
        let context = SpanContext::new("trace-1".to_string(), "span-1".to_string());
        assert_eq!(context.trace_id, "trace-1");
        assert_eq!(context.span_id, "span-1");
        assert!(context.parent_span_id.is_none());

        let child = context.create_child("span-2".to_string());
        assert_eq!(child.trace_id, "trace-1");
        assert_eq!(child.span_id, "span-2");
        assert_eq!(child.parent_span_id, Some("span-1".to_string()));
    }

    #[test]
    fn test_span_builder() {
        let span = Span::builder("test-span".to_string())
            .with_tag("key".to_string(), "value".to_string())
            .build();

        assert_eq!(span.name(), "test-span");
        assert_eq!(span.tags.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_trace_utils() {
        let trace_id = TraceUtils::generate_trace_id();
        let span_id = TraceUtils::generate_span_id();
        
        assert!(!trace_id.is_empty());
        assert!(!span_id.is_empty());
        assert_ne!(trace_id, span_id);
    }
}
