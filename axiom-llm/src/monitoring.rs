//! Monitoring and metrics for LLM operations

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use axiom_core::AxiomError;
use crate::gateway::{LlmRequest, LlmResponse};

/// Metrics for LLM operations
#[derive(Debug, Clone)]
pub struct LlmMetrics {
    /// Total number of requests
    pub total_requests: u64,
    /// Total number of successful requests
    pub successful_requests: u64,
    /// Total number of failed requests
    pub failed_requests: u64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Total cost in USD
    pub total_cost: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Provider-specific metrics
    pub provider_metrics: HashMap<String, ProviderMetrics>,
    /// Request rate (requests per minute)
    pub request_rate: f64,
    /// Error rate (errors per minute)
    pub error_rate: f64,
    /// Last updated timestamp
    pub last_updated: Instant,
}

/// Metrics for a specific provider
#[derive(Debug, Clone)]
pub struct ProviderMetrics {
    /// Provider name
    pub provider_name: String,
    /// Total requests for this provider
    pub total_requests: u64,
    /// Successful requests for this provider
    pub successful_requests: u64,
    /// Failed requests for this provider
    pub failed_requests: u64,
    /// Total tokens used by this provider
    pub total_tokens: u64,
    /// Total cost for this provider
    pub total_cost: f64,
    /// Average response time for this provider
    pub avg_response_time_ms: f64,
    /// Last request time
    pub last_request_time: Option<Instant>,
    /// Request history (for rate calculation)
    pub request_history: Vec<Instant>,
    /// Error history (for error rate calculation)
    pub error_history: Vec<Instant>,
}

impl LlmMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_tokens: 0,
            total_cost: 0.0,
            avg_response_time_ms: 0.0,
            provider_metrics: HashMap::new(),
            request_rate: 0.0,
            error_rate: 0.0,
            last_updated: Instant::now(),
        }
    }

    /// Record a request
    pub fn record_request(&mut self, provider_name: &str, request: &LlmRequest, response: &LlmResponse) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.total_tokens += response.tokens_used.unwrap_or(0) as u64;
        self.total_cost += response.cost.unwrap_or(0.0);

        // Update provider metrics
        let provider_metrics = self.provider_metrics.entry(provider_name.to_string())
            .or_insert_with(|| ProviderMetrics::new(provider_name.to_string()));
        
        provider_metrics.record_request(request, response, true);
        
        self.update_rates();
        self.last_updated = Instant::now();
    }

    /// Record a failed request
    pub fn record_failed_request(&mut self, provider_name: &str, request: &LlmRequest, error: &AxiomError) {
        self.total_requests += 1;
        self.failed_requests += 1;

        // Update provider metrics
        let provider_metrics = self.provider_metrics.entry(provider_name.to_string())
            .or_insert_with(|| ProviderMetrics::new(provider_name.to_string()));
        
        provider_metrics.record_failed_request(request, error);
        
        self.update_rates();
        self.last_updated = Instant::now();
    }

    /// Record a streaming request
    pub fn record_stream_request(&mut self, provider_name: &str, request: &LlmRequest) {
        // For streaming requests, we'll just record the request
        // The actual metrics will be updated when the stream completes
        self.total_requests += 1;

        let provider_metrics = self.provider_metrics.entry(provider_name.to_string())
            .or_insert_with(|| ProviderMetrics::new(provider_name.to_string()));
        
        provider_metrics.record_stream_request(request);
        
        self.update_rates();
        self.last_updated = Instant::now();
    }

    /// Update request and error rates
    fn update_rates(&mut self) {
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);

        // Calculate request rate
        let recent_requests = self.provider_metrics.values()
            .flat_map(|pm| &pm.request_history)
            .filter(|&&time| time > one_minute_ago)
            .count();
        self.request_rate = recent_requests as f64 / 60.0;

        // Calculate error rate
        let recent_errors = self.provider_metrics.values()
            .flat_map(|pm| &pm.error_history)
            .filter(|&&time| time > one_minute_ago)
            .count();
        self.error_rate = recent_errors as f64 / 60.0;
    }

    /// Get metrics for a specific provider
    pub fn get_provider_metrics(&self, provider_name: &str) -> Option<&ProviderMetrics> {
        self.provider_metrics.get(provider_name)
    }

    /// Get all provider names
    pub fn get_provider_names(&self) -> Vec<&String> {
        self.provider_metrics.keys().collect()
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl ProviderMetrics {
    /// Create new provider metrics
    pub fn new(provider_name: String) -> Self {
        Self {
            provider_name,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_tokens: 0,
            total_cost: 0.0,
            avg_response_time_ms: 0.0,
            last_request_time: None,
            request_history: Vec::new(),
            error_history: Vec::new(),
        }
    }

    /// Record a successful request
    pub fn record_request(&mut self, _request: &LlmRequest, response: &LlmResponse, success: bool) {
        let now = Instant::now();
        
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
            self.total_tokens += response.tokens_used.unwrap_or(0) as u64;
            self.total_cost += response.cost.unwrap_or(0.0);
        } else {
            self.failed_requests += 1;
        }

        // Update average response time
        let response_time = response.generation_time_ms as f64;
        self.avg_response_time_ms = (self.avg_response_time_ms * (self.total_requests - 1) as f64 + response_time) / self.total_requests as f64;

        self.last_request_time = Some(now);
        self.request_history.push(now);

        // Clean up old history (keep only last hour)
        let one_hour_ago = now - Duration::from_secs(3600);
        self.request_history.retain(|&time| time > one_hour_ago);
    }

    /// Record a failed request
    pub fn record_failed_request(&mut self, _request: &LlmRequest, _error: &AxiomError) {
        let now = Instant::now();
        
        self.total_requests += 1;
        self.failed_requests += 1;

        self.last_request_time = Some(now);
        self.request_history.push(now);
        self.error_history.push(now);

        // Clean up old history
        let one_hour_ago = now - Duration::from_secs(3600);
        self.request_history.retain(|&time| time > one_hour_ago);
        self.error_history.retain(|&time| time > one_hour_ago);
    }

    /// Record a streaming request
    pub fn record_stream_request(&mut self, _request: &LlmRequest) {
        let now = Instant::now();
        
        self.total_requests += 1;
        self.last_request_time = Some(now);
        self.request_history.push(now);

        // Clean up old history
        let one_hour_ago = now - Duration::from_secs(3600);
        self.request_history.retain(|&time| time > one_hour_ago);
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

    /// Get requests per minute
    pub fn requests_per_minute(&self) -> f64 {
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);
        
        let recent_requests = self.request_history.iter()
            .filter(|&&time| time > one_minute_ago)
            .count();
        
        recent_requests as f64
    }

    /// Get errors per minute
    pub fn errors_per_minute(&self) -> f64 {
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);
        
        let recent_errors = self.error_history.iter()
            .filter(|&&time| time > one_minute_ago)
            .count();
        
        recent_errors as f64
    }
}

/// Health check for LLM services
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// Provider name
    pub provider_name: String,
    /// Whether the provider is healthy
    pub is_healthy: bool,
    /// Last check time
    pub last_check: Instant,
    /// Error message if unhealthy
    pub error_message: Option<String>,
    /// Response time for the health check
    pub response_time_ms: Option<u64>,
}

impl HealthCheck {
    /// Create a new health check
    pub fn new(provider_name: String) -> Self {
        Self {
            provider_name,
            is_healthy: true,
            last_check: Instant::now(),
            error_message: None,
            response_time_ms: None,
        }
    }

    /// Mark as healthy
    pub fn mark_healthy(&mut self, response_time_ms: Option<u64>) {
        self.is_healthy = true;
        self.last_check = Instant::now();
        self.error_message = None;
        self.response_time_ms = response_time_ms;
    }

    /// Mark as unhealthy
    pub fn mark_unhealthy(&mut self, error_message: String) {
        self.is_healthy = false;
        self.last_check = Instant::now();
        self.error_message = Some(error_message);
        self.response_time_ms = None;
    }

    /// Check if the health check is stale
    pub fn is_stale(&self, max_age: Duration) -> bool {
        self.last_check.elapsed() > max_age
    }
}

/// Monitoring service for LLM operations
pub struct MonitoringService {
    metrics: Arc<RwLock<LlmMetrics>>,
    health_checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
    max_health_check_age: Duration,
}

impl MonitoringService {
    /// Create a new monitoring service
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(LlmMetrics::new())),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            max_health_check_age: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Get metrics
    pub async fn get_metrics(&self) -> LlmMetrics {
        self.metrics.read().await.clone()
    }

    /// Record a request
    pub async fn record_request(&self, provider_name: &str, request: &LlmRequest, response: &LlmResponse) {
        let mut metrics = self.metrics.write().await;
        metrics.record_request(provider_name, request, response);
    }

    /// Record a failed request
    pub async fn record_failed_request(&self, provider_name: &str, request: &LlmRequest, error: &AxiomError) {
        let mut metrics = self.metrics.write().await;
        metrics.record_failed_request(provider_name, request, error);
    }

    /// Record a streaming request
    pub async fn record_stream_request(&self, provider_name: &str, request: &LlmRequest) {
        let mut metrics = self.metrics.write().await;
        metrics.record_stream_request(provider_name, request);
    }

    /// Update health check for a provider
    pub async fn update_health_check(&self, provider_name: &str, is_healthy: bool, error_message: Option<String>, response_time_ms: Option<u64>) {
        let mut health_checks = self.health_checks.write().await;
        let health_check = health_checks.entry(provider_name.to_string())
            .or_insert_with(|| HealthCheck::new(provider_name.to_string()));

        if is_healthy {
            health_check.mark_healthy(response_time_ms);
        } else {
            health_check.mark_unhealthy(error_message.unwrap_or_else(|| "Unknown error".to_string()));
        }
    }

    /// Get health status for all providers
    pub async fn get_health_status(&self) -> HashMap<String, HealthCheck> {
        self.health_checks.read().await.clone()
    }

    /// Get health status for a specific provider
    pub async fn get_provider_health(&self, provider_name: &str) -> Option<HealthCheck> {
        self.health_checks.read().await.get(provider_name).cloned()
    }

    /// Check if a provider is healthy
    pub async fn is_provider_healthy(&self, provider_name: &str) -> bool {
        if let Some(health_check) = self.get_provider_health(provider_name).await {
            health_check.is_healthy && !health_check.is_stale(self.max_health_check_age)
        } else {
            false
        }
    }

    /// Get unhealthy providers
    pub async fn get_unhealthy_providers(&self) -> Vec<String> {
        let health_checks = self.health_checks.read().await;
        health_checks.iter()
            .filter(|(_, health_check)| !health_check.is_healthy || health_check.is_stale(self.max_health_check_age))
            .map(|(name, _)| name.clone())
            .collect()
    }
}

impl Default for MonitoringService {
    fn default() -> Self {
        Self::new()
    }
}
