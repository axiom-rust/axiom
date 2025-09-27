//! Metrics collection and export

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use metrics::{Counter, Gauge, Histogram, Key, KeyName, SharedString};
use metrics_exporter_prometheus::PrometheusBuilder;

use axiom_core::{Result, AxiomError};

/// Metrics collector for Axiom
pub struct MetricsCollector {
    /// Request counter
    request_counter: Counter,
    /// Error counter
    error_counter: Counter,
    /// Response time histogram
    response_time_histogram: Histogram,
    /// Active connections gauge
    active_connections_gauge: Gauge,
    /// Memory usage gauge
    memory_usage_gauge: Gauge,
    /// Custom metrics
    custom_metrics: HashMap<String, Box<dyn CustomMetric>>,
}

/// Trait for custom metrics
pub trait CustomMetric: Send + Sync {
    /// Get the metric value
    fn get_value(&self) -> f64;
    /// Get the metric name
    fn get_name(&self) -> &str;
    /// Get the metric description
    fn get_description(&self) -> &str;
}

/// Counter metric
pub struct CounterMetric {
    name: String,
    description: String,
    value: std::sync::atomic::AtomicU64,
}

impl CounterMetric {
    /// Create a new counter metric
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            value: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Increment the counter
    pub fn increment(&self) {
        self.value.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Add a value to the counter
    pub fn add(&self, value: u64) {
        self.value.fetch_add(value, std::sync::atomic::Ordering::Relaxed);
    }
}

impl CustomMetric for CounterMetric {
    fn get_value(&self) -> f64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed) as f64
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        &self.description
    }
}

/// Gauge metric
pub struct GaugeMetric {
    name: String,
    description: String,
    value: std::sync::atomic::AtomicU64,
}

impl GaugeMetric {
    /// Create a new gauge metric
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            value: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Set the gauge value
    pub fn set(&self, value: u64) {
        self.value.store(value, std::sync::atomic::Ordering::Relaxed);
    }
}

impl CustomMetric for GaugeMetric {
    fn get_value(&self) -> f64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed) as f64
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        &self.description
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Result<Self> {
        // Initialize Prometheus exporter
        let builder = PrometheusBuilder::new();
        builder.install()
            .map_err(|e| AxiomError::Monitoring(format!("Failed to install Prometheus exporter: {}", e)))?;

        // Create metrics
        let request_counter = Counter::from_arc(
            metrics::Counter::from_arc(Arc::new(
                metrics::Counter::new(Key::from_name("axiom_requests_total"))
            ))
        );

        let error_counter = Counter::from_arc(
            metrics::Counter::from_arc(Arc::new(
                metrics::Counter::new(Key::from_name("axiom_errors_total"))
            ))
        );

        let response_time_histogram = Histogram::from_arc(
            metrics::Histogram::from_arc(Arc::new(
                metrics::Histogram::new(Key::from_name("axiom_response_time_seconds"))
            ))
        );

        let active_connections_gauge = Gauge::from_arc(
            metrics::Gauge::from_arc(Arc::new(
                metrics::Gauge::new(Key::from_name("axiom_active_connections"))
            ))
        );

        let memory_usage_gauge = Gauge::from_arc(
            metrics::Gauge::from_arc(Arc::new(
                metrics::Gauge::new(Key::from_name("axiom_memory_usage_bytes"))
            ))
        );

        Ok(Self {
            request_counter,
            error_counter,
            response_time_histogram,
            active_connections_gauge,
            memory_usage_gauge,
            custom_metrics: HashMap::new(),
        })
    }

    /// Record a request
    pub fn record_request(&self, labels: &[(&str, &str)]) {
        self.request_counter.increment(1.0);
    }

    /// Record an error
    pub fn record_error(&self, labels: &[(&str, &str)]) {
        self.error_counter.increment(1.0);
    }

    /// Record response time
    pub fn record_response_time(&self, duration: Duration, labels: &[(&str, &str)]) {
        self.response_time_histogram.record(duration.as_secs_f64());
    }

    /// Set active connections
    pub fn set_active_connections(&self, count: u64) {
        self.active_connections_gauge.set(count as f64);
    }

    /// Set memory usage
    pub fn set_memory_usage(&self, bytes: u64) {
        self.memory_usage_gauge.set(bytes as f64);
    }

    /// Add a custom metric
    pub fn add_custom_metric(&mut self, name: String, metric: Box<dyn CustomMetric>) {
        self.custom_metrics.insert(name, metric);
    }

    /// Get a custom metric
    pub fn get_custom_metric(&self, name: &str) -> Option<&dyn CustomMetric> {
        self.custom_metrics.get(name).map(|m| m.as_ref())
    }

    /// Get all custom metrics
    pub fn get_custom_metrics(&self) -> &HashMap<String, Box<dyn CustomMetric>> {
        &self.custom_metrics
    }
}

/// Metrics for LLM operations
pub struct LlmMetrics {
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Total tokens
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Request rate
    pub request_rate: f64,
    /// Error rate
    pub error_rate: f64,
}

impl LlmMetrics {
    /// Create new LLM metrics
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_tokens: 0,
            total_cost: 0.0,
            avg_response_time_ms: 0.0,
            request_rate: 0.0,
            error_rate: 0.0,
        }
    }

    /// Record a request
    pub fn record_request(&mut self, success: bool, tokens: u64, cost: f64, response_time_ms: u64) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
        self.total_tokens += tokens;
        self.total_cost += cost;
        
        // Update average response time
        self.avg_response_time_ms = (self.avg_response_time_ms * (self.total_requests - 1) as f64 + response_time_ms as f64) / self.total_requests as f64;
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    /// Get error rate
    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.failed_requests as f64 / self.total_requests as f64
        }
    }
}

/// Metrics for agent operations
pub struct AgentMetrics {
    /// Total agent executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Total tool calls
    pub total_tool_calls: u64,
    /// Successful tool calls
    pub successful_tool_calls: u64,
    /// Failed tool calls
    pub failed_tool_calls: u64,
    /// Average execution time
    pub avg_execution_time_ms: f64,
}

impl AgentMetrics {
    /// Create new agent metrics
    pub fn new() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_tool_calls: 0,
            successful_tool_calls: 0,
            failed_tool_calls: 0,
            avg_execution_time_ms: 0.0,
        }
    }

    /// Record an execution
    pub fn record_execution(&mut self, success: bool, execution_time_ms: u64) {
        self.total_executions += 1;
        if success {
            self.successful_executions += 1;
        } else {
            self.failed_executions += 1;
        }
        
        // Update average execution time
        self.avg_execution_time_ms = (self.avg_execution_time_ms * (self.total_executions - 1) as f64 + execution_time_ms as f64) / self.total_executions as f64;
    }

    /// Record a tool call
    pub fn record_tool_call(&mut self, success: bool) {
        self.total_tool_calls += 1;
        if success {
            self.successful_tool_calls += 1;
        } else {
            self.failed_tool_calls += 1;
        }
    }

    /// Get execution success rate
    pub fn execution_success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.successful_executions as f64 / self.total_executions as f64
        }
    }

    /// Get tool call success rate
    pub fn tool_call_success_rate(&self) -> f64 {
        if self.total_tool_calls == 0 {
            0.0
        } else {
            self.successful_tool_calls as f64 / self.total_tool_calls as f64
        }
    }
}

/// Metrics for RAG operations
pub struct RagMetrics {
    /// Total retrievals
    pub total_retrievals: u64,
    /// Successful retrievals
    pub successful_retrievals: u64,
    /// Failed retrievals
    pub failed_retrievals: u64,
    /// Total documents indexed
    pub total_documents_indexed: u64,
    /// Total chunks indexed
    pub total_chunks_indexed: u64,
    /// Average retrieval time
    pub avg_retrieval_time_ms: f64,
    /// Average similarity score
    pub avg_similarity_score: f64,
}

impl RagMetrics {
    /// Create new RAG metrics
    pub fn new() -> Self {
        Self {
            total_retrievals: 0,
            successful_retrievals: 0,
            failed_retrievals: 0,
            total_documents_indexed: 0,
            total_chunks_indexed: 0,
            avg_retrieval_time_ms: 0.0,
            avg_similarity_score: 0.0,
        }
    }

    /// Record a retrieval
    pub fn record_retrieval(&mut self, success: bool, retrieval_time_ms: u64, similarity_score: f64) {
        self.total_retrievals += 1;
        if success {
            self.successful_retrievals += 1;
        } else {
            self.failed_retrievals += 1;
        }
        
        // Update average retrieval time
        self.avg_retrieval_time_ms = (self.avg_retrieval_time_ms * (self.total_retrievals - 1) as f64 + retrieval_time_ms as f64) / self.total_retrievals as f64;
        
        // Update average similarity score
        self.avg_similarity_score = (self.avg_similarity_score * (self.total_retrievals - 1) as f64 + similarity_score) / self.total_retrievals as f64;
    }

    /// Record document indexing
    pub fn record_document_indexed(&mut self, chunk_count: usize) {
        self.total_documents_indexed += 1;
        self.total_chunks_indexed += chunk_count as u64;
    }

    /// Get retrieval success rate
    pub fn retrieval_success_rate(&self) -> f64 {
        if self.total_retrievals == 0 {
            0.0
        } else {
            self.successful_retrievals as f64 / self.total_retrievals as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_metric() {
        let counter = CounterMetric::new("test_counter".to_string(), "Test counter".to_string());
        assert_eq!(counter.get_value(), 0.0);
        
        counter.increment();
        assert_eq!(counter.get_value(), 1.0);
        
        counter.add(5);
        assert_eq!(counter.get_value(), 6.0);
    }

    #[test]
    fn test_gauge_metric() {
        let gauge = GaugeMetric::new("test_gauge".to_string(), "Test gauge".to_string());
        assert_eq!(gauge.get_value(), 0.0);
        
        gauge.set(42);
        assert_eq!(gauge.get_value(), 42.0);
    }

    #[test]
    fn test_llm_metrics() {
        let mut metrics = LlmMetrics::new();
        assert_eq!(metrics.total_requests, 0);
        
        metrics.record_request(true, 100, 0.01, 500);
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.success_rate(), 1.0);
    }

    #[test]
    fn test_agent_metrics() {
        let mut metrics = AgentMetrics::new();
        assert_eq!(metrics.total_executions, 0);
        
        metrics.record_execution(true, 1000);
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.successful_executions, 1);
        assert_eq!(metrics.execution_success_rate(), 1.0);
    }

    #[test]
    fn test_rag_metrics() {
        let mut metrics = RagMetrics::new();
        assert_eq!(metrics.total_retrievals, 0);
        
        metrics.record_retrieval(true, 200, 0.85);
        assert_eq!(metrics.total_retrievals, 1);
        assert_eq!(metrics.successful_retrievals, 1);
        assert_eq!(metrics.retrieval_success_rate(), 1.0);
    }
}
