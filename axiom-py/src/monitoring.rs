//! Monitoring and observability functionality exposed to Python

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use axiom_monitoring::{
    MetricsCollector as RustMetricsCollector, HealthCheckManager as RustHealthCheckManager,
    TracingConfig as RustTracingConfig, AlertManager as RustAlertManager
};

/// Python wrapper for MetricsCollector
#[pyclass(name = "MetricsCollector")]
pub struct MetricsCollector {
    inner: RustMetricsCollector,
}

#[pymethods]
impl MetricsCollector {
    /// Create a new metrics collector
    #[new]
    fn new() -> PyResult<Self> {
        let collector = RustMetricsCollector::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { inner: collector })
    }

    /// Increment a counter metric
    fn increment_counter(&self, name: &str, value: u64, labels: Option<HashMap<String, String>>) -> PyResult<()> {
        let labels = labels.unwrap_or_default();
        self.inner
            .increment_counter(name, value, &labels)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(())
    }

    /// Record a gauge metric
    fn record_gauge(&self, name: &str, value: f64, labels: Option<HashMap<String, String>>) -> PyResult<()> {
        let labels = labels.unwrap_or_default();
        self.inner
            .record_gauge(name, value, &labels)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(())
    }

    /// Record a histogram metric
    fn record_histogram(&self, name: &str, value: f64, labels: Option<HashMap<String, String>>) -> PyResult<()> {
        let labels = labels.unwrap_or_default();
        self.inner
            .record_histogram(name, value, &labels)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(())
    }

    /// Record a timing metric
    fn record_timing(&self, name: &str, duration_ms: u64, labels: Option<HashMap<String, String>>) -> PyResult<()> {
        let labels = labels.unwrap_or_default();
        self.inner
            .record_timing(name, duration_ms, &labels)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(())
    }

    /// Get metrics in Prometheus format
    fn get_metrics(&self) -> PyResult<String> {
        let metrics = self.inner
            .get_metrics()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(metrics)
    }

    /// Export metrics to a file
    fn export_metrics(&self, file_path: &str) -> PyResult<()> {
        self.inner
            .export_metrics(file_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(())
    }
}

impl From<RustMetricsCollector> for MetricsCollector {
    fn from(collector: RustMetricsCollector) -> Self {
        Self { inner: collector }
    }
}

/// Python wrapper for HealthCheckManager
#[pyclass(name = "HealthCheckManager")]
pub struct HealthCheckManager {
    inner: RustHealthCheckManager,
}

#[pymethods]
impl HealthCheckManager {
    /// Create a new health check manager
    #[new]
    fn new() -> Self {
        Self {
            inner: RustHealthCheckManager::new(),
        }
    }

    /// Add a health checker
    fn add_checker(&mut self, checker: Box<dyn HealthChecker>) -> PyResult<()> {
        // This would need to be implemented based on the actual HealthCheckManager API
        Ok(())
    }

    /// Run all health checks
    fn run_checks(&self) -> PyResult<Vec<HealthCheckResult>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let results = rt.block_on(async {
            self.inner.run_checks().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    /// Get the overall health status
    fn is_healthy(&self) -> bool {
        self.inner.is_healthy()
    }

    /// Get health status as JSON
    fn get_health_status(&self) -> PyResult<String> {
        let status = self.inner
            .get_health_status()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(status)
    }
}

impl From<RustHealthCheckManager> for HealthCheckManager {
    fn from(manager: RustHealthCheckManager) -> Self {
        Self { inner: manager }
    }
}

/// Python wrapper for HealthCheckResult
#[pyclass(name = "HealthCheckResult")]
pub struct HealthCheckResult {
    inner: axiom_monitoring::HealthCheckResult,
}

#[pymethods]
impl HealthCheckResult {
    /// Get the checker name
    fn checker_name(&self) -> String {
        self.inner.checker_name.clone()
    }

    /// Check if the health check passed
    fn is_healthy(&self) -> bool {
        self.inner.is_healthy
    }

    /// Get the health check message
    fn message(&self) -> String {
        self.inner.message.clone()
    }

    /// Get the health check duration in milliseconds
    fn duration_ms(&self) -> u64 {
        self.inner.duration_ms
    }

    /// Get the timestamp
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }
}

impl From<axiom_monitoring::HealthCheckResult> for HealthCheckResult {
    fn from(result: axiom_monitoring::HealthCheckResult) -> Self {
        Self { inner: result }
    }
}

/// Python wrapper for TracingConfig
#[pyclass(name = "TracingConfig")]
pub struct TracingConfig {
    inner: RustTracingConfig,
}

#[pymethods]
impl TracingConfig {
    /// Create a new tracing configuration
    #[new]
    fn new(service_name: String) -> Self {
        Self {
            inner: RustTracingConfig::new(service_name),
        }
    }

    /// Configure Jaeger tracing
    fn with_jaeger(mut self_: PyRef<Self>, jaeger_endpoint: String) -> PyRef<Self> {
        self_.inner = self_.inner.with_jaeger(jaeger_endpoint);
        self_
    }

    /// Configure Zipkin tracing
    fn with_zipkin(mut self_: PyRef<Self>, zipkin_endpoint: String) -> PyRef<Self> {
        self_.inner = self_.inner.with_zipkin(zipkin_endpoint);
        self_
    }

    /// Set the log level
    fn with_log_level(mut self_: PyRef<Self>, level: &str) -> PyRef<Self> {
        self_.inner = self_.inner.with_log_level(level);
        self_
    }

    /// Initialize tracing
    fn init(&self) -> PyResult<()> {
        self.inner
            .init()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(())
    }
}

impl From<RustTracingConfig> for TracingConfig {
    fn from(config: RustTracingConfig) -> Self {
        Self { inner: config }
    }
}

/// Python wrapper for AlertManager
#[pyclass(name = "AlertManager")]
pub struct AlertManager {
    inner: RustAlertManager,
}

#[pymethods]
impl AlertManager {
    /// Create a new alert manager
    #[new]
    fn new() -> Self {
        Self {
            inner: RustAlertManager::new(),
        }
    }

    /// Add an alert rule
    fn add_alert_rule(&mut self, rule: AlertRule) -> PyResult<()> {
        self.inner
            .add_alert_rule(rule.into())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(())
    }

    /// Send an alert
    fn send_alert(&self, alert: Alert) -> PyResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        rt.block_on(async {
            self.inner.send_alert(alert.into()).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(())
    }

    /// Check for triggered alerts
    fn check_alerts(&self) -> PyResult<Vec<Alert>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let alerts = rt.block_on(async {
            self.inner.check_alerts().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(alerts.into_iter().map(|a| a.into()).collect())
    }
}

impl From<RustAlertManager> for AlertManager {
    fn from(manager: RustAlertManager) -> Self {
        Self { inner: manager }
    }
}

/// Python wrapper for AlertRule
#[pyclass(name = "AlertRule")]
pub struct AlertRule {
    inner: axiom_monitoring::AlertRule,
}

#[pymethods]
impl AlertRule {
    /// Create a new alert rule
    #[new]
    fn new(name: String, condition: String, severity: AlertSeverity) -> Self {
        Self {
            inner: axiom_monitoring::AlertRule::new(name, condition, severity.into()),
        }
    }

    /// Set the alert message
    fn with_message(mut self_: PyRef<Self>, message: String) -> PyRef<Self> {
        self_.inner = self_.inner.with_message(message);
        self_
    }

    /// Set the cooldown period
    fn with_cooldown(mut self_: PyRef<Self>, cooldown_seconds: u64) -> PyRef<Self> {
        self_.inner = self_.inner.with_cooldown(cooldown_seconds);
        self_
    }

    /// Get the rule name
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get the condition
    fn condition(&self) -> String {
        self.inner.condition.clone()
    }

    /// Get the severity
    fn severity(&self) -> AlertSeverity {
        self.inner.severity.clone().into()
    }
}

impl From<axiom_monitoring::AlertRule> for AlertRule {
    fn from(rule: axiom_monitoring::AlertRule) -> Self {
        Self { inner: rule }
    }
}

impl From<AlertRule> for axiom_monitoring::AlertRule {
    fn from(rule: AlertRule) -> Self {
        rule.inner
    }
}

/// Python wrapper for Alert
#[pyclass(name = "Alert")]
pub struct Alert {
    inner: axiom_monitoring::Alert,
}

#[pymethods]
impl Alert {
    /// Create a new alert
    #[new]
    fn new(title: String, message: String, severity: AlertSeverity) -> Self {
        Self {
            inner: axiom_monitoring::Alert::new(title, message, severity.into()),
        }
    }

    /// Get the alert title
    fn title(&self) -> String {
        self.inner.title.clone()
    }

    /// Get the alert message
    fn message(&self) -> String {
        self.inner.message.clone()
    }

    /// Get the alert severity
    fn severity(&self) -> AlertSeverity {
        self.inner.severity.clone().into()
    }

    /// Get the alert timestamp
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    /// Get the alert metadata
    fn metadata(&self) -> HashMap<String, String> {
        self.inner
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect()
    }
}

impl From<axiom_monitoring::Alert> for Alert {
    fn from(alert: axiom_monitoring::Alert) -> Self {
        Self { inner: alert }
    }
}

impl From<Alert> for axiom_monitoring::Alert {
    fn from(alert: Alert) -> Self {
        alert.inner
    }
}

/// Python wrapper for AlertSeverity
#[pyclass(name = "AlertSeverity")]
#[derive(Clone)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl From<axiom_monitoring::AlertSeverity> for AlertSeverity {
    fn from(severity: axiom_monitoring::AlertSeverity) -> Self {
        match severity {
            axiom_monitoring::AlertSeverity::Low => AlertSeverity::Low,
            axiom_monitoring::AlertSeverity::Medium => AlertSeverity::Medium,
            axiom_monitoring::AlertSeverity::High => AlertSeverity::High,
            axiom_monitoring::AlertSeverity::Critical => AlertSeverity::Critical,
        }
    }
}

impl From<AlertSeverity> for axiom_monitoring::AlertSeverity {
    fn from(severity: AlertSeverity) -> Self {
        match severity {
            AlertSeverity::Low => axiom_monitoring::AlertSeverity::Low,
            AlertSeverity::Medium => axiom_monitoring::AlertSeverity::Medium,
            AlertSeverity::High => axiom_monitoring::AlertSeverity::High,
            AlertSeverity::Critical => axiom_monitoring::AlertSeverity::Critical,
        }
    }
}

// Trait for health checkers
pub trait HealthChecker: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self) -> PyResult<HealthCheckResult>;
}
