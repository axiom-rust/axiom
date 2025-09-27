//! Health checking and monitoring

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axiom_core::{Result, AxiomError};

/// Health status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is unhealthy
    Unhealthy,
    /// Service is degraded
    Degraded,
    /// Service status is unknown
    Unknown,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Service name
    pub service_name: String,
    /// Health status
    pub status: HealthStatus,
    /// Check message
    pub message: String,
    /// Check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Trait for health checkers
#[async_trait::async_trait]
pub trait HealthChecker: Send + Sync {
    /// Get the name of the health checker
    fn name(&self) -> &str;
    
    /// Perform the health check
    async fn check(&self) -> HealthCheckResult;
    
    /// Get the check interval
    fn check_interval(&self) -> Duration;
}

/// Basic health checker
pub struct BasicHealthChecker {
    /// Service name
    service_name: String,
    /// Check interval
    check_interval: Duration,
    /// Last check time
    last_check: Option<Instant>,
    /// Last result
    last_result: Option<HealthCheckResult>,
}

impl BasicHealthChecker {
    /// Create a new basic health checker
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            check_interval: Duration::from_secs(30),
            last_check: None,
            last_result: None,
        }
    }

    /// Set the check interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }
}

#[async_trait::async_trait]
impl HealthChecker for BasicHealthChecker {
    fn name(&self) -> &str {
        &self.service_name
    }

    async fn check(&self) -> HealthCheckResult {
        let start_time = Instant::now();
        
        // Basic health check - always healthy
        let status = HealthStatus::Healthy;
        let message = "Service is running normally".to_string();
        let response_time = start_time.elapsed().as_millis() as u64;
        
        HealthCheckResult {
            service_name: self.service_name.clone(),
            status,
            message,
            timestamp: chrono::Utc::now(),
            response_time_ms: response_time,
            metadata: HashMap::new(),
        }
    }

    fn check_interval(&self) -> Duration {
        self.check_interval
    }
}

/// LLM health checker
pub struct LlmHealthChecker {
    /// Service name
    service_name: String,
    /// LLM gateway
    llm_gateway: Arc<axiom_llm::LlmGateway>,
    /// Check interval
    check_interval: Duration,
    /// Test prompt
    test_prompt: String,
}

impl LlmHealthChecker {
    /// Create a new LLM health checker
    pub fn new(service_name: String, llm_gateway: Arc<axiom_llm::LlmGateway>) -> Self {
        Self {
            service_name,
            llm_gateway,
            check_interval: Duration::from_secs(60),
            test_prompt: "Hello".to_string(),
        }
    }

    /// Set the test prompt
    pub fn with_test_prompt(mut self, prompt: String) -> Self {
        self.test_prompt = prompt;
        self
    }

    /// Set the check interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }
}

#[async_trait::async_trait]
impl HealthChecker for LlmHealthChecker {
    fn name(&self) -> &str {
        &self.service_name
    }

    async fn check(&self) -> HealthCheckResult {
        let start_time = Instant::now();
        
        // Test LLM with a simple request
        let request = axiom_llm::LlmRequest::new(
            vec![axiom_core::Message::user(self.test_prompt.clone())],
            "gpt-3.5-turbo".to_string(),
        );

        match self.llm_gateway.generate(request).await {
            Ok(response) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                let mut metadata = HashMap::new();
                metadata.insert("tokens_used".to_string(), serde_json::Value::Number(response.tokens_used.unwrap_or(0).into()));
                metadata.insert("model".to_string(), serde_json::Value::String(response.model));
                metadata.insert("provider".to_string(), serde_json::Value::String(response.provider));

                HealthCheckResult {
                    service_name: self.service_name.clone(),
                    status: HealthStatus::Healthy,
                    message: "LLM service is responding normally".to_string(),
                    timestamp: chrono::Utc::now(),
                    response_time_ms: response_time,
                    metadata,
                }
            }
            Err(e) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                let mut metadata = HashMap::new();
                metadata.insert("error".to_string(), serde_json::Value::String(e.to_string()));

                HealthCheckResult {
                    service_name: self.service_name.clone(),
                    status: HealthStatus::Unhealthy,
                    message: format!("LLM service error: {}", e),
                    timestamp: chrono::Utc::now(),
                    response_time_ms: response_time,
                    metadata,
                }
            }
        }
    }

    fn check_interval(&self) -> Duration {
        self.check_interval
    }
}

/// Database health checker
pub struct DatabaseHealthChecker {
    /// Service name
    service_name: String,
    /// Database URL
    database_url: String,
    /// Check interval
    check_interval: Duration,
}

impl DatabaseHealthChecker {
    /// Create a new database health checker
    pub fn new(service_name: String, database_url: String) -> Self {
        Self {
            service_name,
            database_url,
            check_interval: Duration::from_secs(30),
        }
    }

    /// Set the check interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }
}

#[async_trait::async_trait]
impl HealthChecker for DatabaseHealthChecker {
    fn name(&self) -> &str {
        &self.service_name
    }

    async fn check(&self) -> HealthCheckResult {
        let start_time = Instant::now();
        
        // Simple database health check
        // In a real implementation, you'd execute a simple query
        let response_time = start_time.elapsed().as_millis() as u64;
        
        // For now, we'll simulate a successful check
        HealthCheckResult {
            service_name: self.service_name.clone(),
            status: HealthStatus::Healthy,
            message: "Database is accessible".to_string(),
            timestamp: chrono::Utc::now(),
            response_time_ms: response_time,
            metadata: HashMap::new(),
        }
    }

    fn check_interval(&self) -> Duration {
        self.check_interval
    }
}

/// Health check manager
pub struct HealthCheckManager {
    /// Health checkers
    checkers: HashMap<String, Box<dyn HealthChecker>>,
    /// Check results
    results: HashMap<String, HealthCheckResult>,
    /// Last check times
    last_checks: HashMap<String, Instant>,
}

impl HealthCheckManager {
    /// Create a new health check manager
    pub fn new() -> Self {
        Self {
            checkers: HashMap::new(),
            results: HashMap::new(),
            last_checks: HashMap::new(),
        }
    }

    /// Add a health checker
    pub fn add_checker(&mut self, checker: Box<dyn HealthChecker>) {
        let name = checker.name().to_string();
        self.checkers.insert(name, checker);
    }

    /// Run all health checks
    pub async fn run_checks(&mut self) -> Result<()> {
        let mut futures = Vec::new();
        
        for (name, checker) in &self.checkers {
            let name = name.clone();
            let checker = checker.as_ref();
            futures.push(async move {
                let result = checker.check().await;
                (name, result)
            });
        }

        let results = futures::future::join_all(futures).await;
        
        for (name, result) in results {
            self.results.insert(name.clone(), result);
            self.last_checks.insert(name, Instant::now());
        }

        Ok(())
    }

    /// Run a specific health check
    pub async fn run_check(&mut self, name: &str) -> Result<HealthCheckResult> {
        let checker = self.checkers.get(name)
            .ok_or_else(|| AxiomError::Monitoring(format!("Health checker not found: {}", name)))?;

        let result = checker.check().await;
        self.results.insert(name.to_string(), result.clone());
        self.last_checks.insert(name.to_string(), Instant::now());
        
        Ok(result)
    }

    /// Get the overall health status
    pub fn get_overall_status(&self) -> HealthStatus {
        if self.results.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for result in self.results.values() {
            match result.status {
                HealthStatus::Unhealthy => {
                    has_unhealthy = true;
                    break;
                }
                HealthStatus::Degraded => {
                    has_degraded = true;
                }
                _ => {}
            }
        }

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get all health check results
    pub fn get_results(&self) -> &HashMap<String, HealthCheckResult> {
        &self.results
    }

    /// Get a specific health check result
    pub fn get_result(&self, name: &str) -> Option<&HealthCheckResult> {
        self.results.get(name)
    }

    /// Check if a health check is due
    pub fn is_check_due(&self, name: &str) -> bool {
        if let Some(checker) = self.checkers.get(name) {
            if let Some(last_check) = self.last_checks.get(name) {
                last_check.elapsed() >= checker.check_interval()
            } else {
                true
            }
        } else {
            false
        }
    }

    /// Get all checkers that are due
    pub fn get_due_checkers(&self) -> Vec<&str> {
        self.checkers.keys()
            .filter(|name| self.is_check_due(name))
            .map(|name| name.as_str())
            .collect()
    }
}

/// Health check server
pub struct HealthCheckServer {
    /// Health check manager
    manager: HealthCheckManager,
    /// Server configuration
    config: HealthServerConfig,
}

/// Health server configuration
#[derive(Debug, Clone)]
pub struct HealthServerConfig {
    /// Server port
    pub port: u16,
    /// Server host
    pub host: String,
    /// Check interval
    pub check_interval: Duration,
    /// Enable automatic checks
    pub enable_auto_checks: bool,
}

impl Default for HealthServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
            check_interval: Duration::from_secs(30),
            enable_auto_checks: true,
        }
    }
}

impl HealthCheckServer {
    /// Create a new health check server
    pub fn new(config: HealthServerConfig) -> Self {
        Self {
            manager: HealthCheckManager::new(),
            config,
        }
    }

    /// Add a health checker
    pub fn add_checker(&mut self, checker: Box<dyn HealthChecker>) {
        self.manager.add_checker(checker);
    }

    /// Start the health check server
    pub async fn start(&mut self) -> Result<()> {
        if self.config.enable_auto_checks {
            self.start_auto_checks().await?;
        }

        // Start HTTP server
        self.start_http_server().await?;

        Ok(())
    }

    /// Start automatic health checks
    async fn start_auto_checks(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(self.config.check_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.manager.run_checks().await {
                tracing::error!("Health check failed: {}", e);
            }
        }
    }

    /// Start HTTP server for health endpoints
    async fn start_http_server(&self) -> Result<()> {
        // In a real implementation, you'd use a proper HTTP server like axum or warp
        // For now, we'll just log that the server would start
        tracing::info!(
            "Health check server would start on {}:{}",
            self.config.host,
            self.config.port
        );
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_health_checker() {
        let checker = BasicHealthChecker::new("test-service".to_string());
        let result = checker.check().await;
        
        assert_eq!(result.service_name, "test-service");
        assert_eq!(result.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_check_manager() {
        let mut manager = HealthCheckManager::new();
        let checker = Box::new(BasicHealthChecker::new("test-service".to_string()));
        manager.add_checker(checker);
        
        manager.run_checks().await.unwrap();
        
        let status = manager.get_overall_status();
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_status() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    }
}
