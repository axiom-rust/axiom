//! Alerting and notification system

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axiom_core::{Result, AxiomError};

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Alert status
#[derive(Debug, Clone, PartialEq)]
pub enum AlertStatus {
    /// Alert is active
    Active,
    /// Alert is resolved
    Resolved,
    /// Alert is acknowledged
    Acknowledged,
    /// Alert is suppressed
    Suppressed,
}

/// An alert
#[derive(Debug, Clone)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert status
    pub status: AlertStatus,
    /// Service name
    pub service_name: String,
    /// Alert source
    pub source: String,
    /// Alert timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
    /// Alert metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Alert tags
    pub tags: HashMap<String, String>,
}

/// Alert rule
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Condition to trigger alert
    pub condition: AlertCondition,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert title template
    pub title_template: String,
    /// Alert description template
    pub description_template: String,
    /// Cooldown period
    pub cooldown: Duration,
    /// Rule enabled
    pub enabled: bool,
}

/// Alert condition
#[derive(Debug, Clone)]
pub enum AlertCondition {
    /// Metric threshold condition
    MetricThreshold {
        metric_name: String,
        operator: ThresholdOperator,
        value: f64,
        duration: Duration,
    },
    /// Health check failure condition
    HealthCheckFailure {
        service_name: String,
        duration: Duration,
    },
    /// Error rate condition
    ErrorRate {
        service_name: String,
        threshold: f64,
        duration: Duration,
    },
    /// Response time condition
    ResponseTime {
        service_name: String,
        threshold_ms: u64,
        duration: Duration,
    },
}

/// Threshold operator
#[derive(Debug, Clone, PartialEq)]
pub enum ThresholdOperator {
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// Equal
    Equal,
    /// Not equal
    NotEqual,
}

/// Alert notification
#[derive(Debug, Clone)]
pub struct AlertNotification {
    /// Alert
    pub alert: Alert,
    /// Notification channels
    pub channels: Vec<String>,
    /// Notification timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Trait for notification channels
#[async_trait::async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Get the channel name
    fn name(&self) -> &str;
    
    /// Send a notification
    async fn send(&self, notification: &AlertNotification) -> Result<()>;
}

/// Email notification channel
pub struct EmailNotificationChannel {
    /// Channel name
    name: String,
    /// SMTP server
    smtp_server: String,
    /// SMTP port
    smtp_port: u16,
    /// Username
    username: String,
    /// Password
    password: String,
    /// From address
    from_address: String,
    /// To addresses
    to_addresses: Vec<String>,
}

impl EmailNotificationChannel {
    /// Create a new email notification channel
    pub fn new(
        name: String,
        smtp_server: String,
        smtp_port: u16,
        username: String,
        password: String,
        from_address: String,
        to_addresses: Vec<String>,
    ) -> Self {
        Self {
            name,
            smtp_server,
            smtp_port,
            username,
            password,
            from_address,
            to_addresses,
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for EmailNotificationChannel {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, notification: &AlertNotification) -> Result<()> {
        // In a real implementation, you'd send an actual email
        tracing::info!(
            "Email notification sent to {:?} for alert: {}",
            self.to_addresses,
            notification.alert.title
        );
        Ok(())
    }
}

/// Slack notification channel
pub struct SlackNotificationChannel {
    /// Channel name
    name: String,
    /// Webhook URL
    webhook_url: String,
    /// Channel name
    channel: String,
}

impl SlackNotificationChannel {
    /// Create a new Slack notification channel
    pub fn new(name: String, webhook_url: String, channel: String) -> Self {
        Self {
            name,
            webhook_url,
            channel,
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for SlackNotificationChannel {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, notification: &AlertNotification) -> Result<()> {
        // In a real implementation, you'd send a Slack message
        tracing::info!(
            "Slack notification sent to #{} for alert: {}",
            self.channel,
            notification.alert.title
        );
        Ok(())
    }
}

/// Alert manager
pub struct AlertManager {
    /// Alert rules
    rules: HashMap<String, AlertRule>,
    /// Active alerts
    active_alerts: HashMap<String, Alert>,
    /// Notification channels
    channels: HashMap<String, Box<dyn NotificationChannel>>,
    /// Last rule evaluations
    last_evaluations: HashMap<String, Instant>,
    /// Alert cooldowns
    cooldowns: HashMap<String, Instant>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            active_alerts: HashMap::new(),
            channels: HashMap::new(),
            last_evaluations: HashMap::new(),
            cooldowns: HashMap::new(),
        }
    }

    /// Add an alert rule
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.insert(rule.name.clone(), rule);
    }

    /// Add a notification channel
    pub fn add_channel(&mut self, channel: Box<dyn NotificationChannel>) {
        let name = channel.name().to_string();
        self.channels.insert(name, channel);
    }

    /// Evaluate all alert rules
    pub async fn evaluate_rules(&mut self, metrics: &HashMap<String, f64>) -> Result<()> {
        for (rule_name, rule) in &self.rules {
            if !rule.enabled {
                continue;
            }

            // Check cooldown
            if let Some(last_eval) = self.last_evaluations.get(rule_name) {
                if last_eval.elapsed() < Duration::from_secs(60) {
                    continue;
                }
            }

            // Check if rule is in cooldown
            if let Some(cooldown_end) = self.cooldowns.get(rule_name) {
                if cooldown_end > &Instant::now() {
                    continue;
                }
            }

            // Evaluate rule condition
            if self.evaluate_condition(&rule.condition, metrics).await? {
                // Trigger alert
                self.trigger_alert(rule).await?;
                
                // Set cooldown
                self.cooldowns.insert(rule_name.clone(), Instant::now() + rule.cooldown);
            }

            self.last_evaluations.insert(rule_name.clone(), Instant::now());
        }

        Ok(())
    }

    /// Evaluate a specific condition
    async fn evaluate_condition(&self, condition: &AlertCondition, metrics: &HashMap<String, f64>) -> Result<bool> {
        match condition {
            AlertCondition::MetricThreshold { metric_name, operator, value, .. } => {
                if let Some(metric_value) = metrics.get(metric_name) {
                    match operator {
                        ThresholdOperator::GreaterThan => Ok(metric_value > value),
                        ThresholdOperator::GreaterThanOrEqual => Ok(metric_value >= value),
                        ThresholdOperator::LessThan => Ok(metric_value < value),
                        ThresholdOperator::LessThanOrEqual => Ok(metric_value <= value),
                        ThresholdOperator::Equal => Ok((metric_value - value).abs() < f64::EPSILON),
                        ThresholdOperator::NotEqual => Ok((metric_value - value).abs() >= f64::EPSILON),
                    }
                } else {
                    Ok(false)
                }
            }
            AlertCondition::HealthCheckFailure { .. } => {
                // In a real implementation, you'd check health status
                Ok(false)
            }
            AlertCondition::ErrorRate { .. } => {
                // In a real implementation, you'd calculate error rate
                Ok(false)
            }
            AlertCondition::ResponseTime { .. } => {
                // In a real implementation, you'd check response times
                Ok(false)
            }
        }
    }

    /// Trigger an alert
    async fn trigger_alert(&mut self, rule: &AlertRule) -> Result<()> {
        let alert_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let alert = Alert {
            id: alert_id.clone(),
            title: rule.title_template.clone(),
            description: rule.description_template.clone(),
            severity: rule.severity.clone(),
            status: AlertStatus::Active,
            service_name: "unknown".to_string(),
            source: rule.name.clone(),
            timestamp: now,
            last_updated: now,
            metadata: HashMap::new(),
            tags: HashMap::new(),
        };

        self.active_alerts.insert(alert_id.clone(), alert.clone());

        // Send notifications
        let notification = AlertNotification {
            alert,
            channels: self.channels.keys().cloned().collect(),
            timestamp: now,
        };

        for channel in self.channels.values() {
            if let Err(e) = channel.send(&notification).await {
                tracing::error!("Failed to send notification via {}: {}", channel.name(), e);
            }
        }

        Ok(())
    }

    /// Resolve an alert
    pub fn resolve_alert(&mut self, alert_id: &str) -> Result<()> {
        if let Some(alert) = self.active_alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Resolved;
            alert.last_updated = chrono::Utc::now();
        }
        Ok(())
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> Result<()> {
        if let Some(alert) = self.active_alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Acknowledged;
            alert.last_updated = chrono::Utc::now();
        }
        Ok(())
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<&Alert> {
        self.active_alerts.values()
            .filter(|alert| alert.status == AlertStatus::Active)
            .collect()
    }

    /// Get all alerts
    pub fn get_all_alerts(&self) -> Vec<&Alert> {
        self.active_alerts.values().collect()
    }

    /// Get alerts by severity
    pub fn get_alerts_by_severity(&self, severity: AlertSeverity) -> Vec<&Alert> {
        self.active_alerts.values()
            .filter(|alert| alert.severity == severity)
            .collect()
    }

    /// Get alert statistics
    pub fn get_statistics(&self) -> AlertStatistics {
        let total_alerts = self.active_alerts.len();
        let active_alerts = self.get_active_alerts().len();
        let critical_alerts = self.get_alerts_by_severity(AlertSeverity::Critical).len();
        let high_alerts = self.get_alerts_by_severity(AlertSeverity::High).len();
        let medium_alerts = self.get_alerts_by_severity(AlertSeverity::Medium).len();
        let low_alerts = self.get_alerts_by_severity(AlertSeverity::Low).len();

        AlertStatistics {
            total_alerts,
            active_alerts,
            critical_alerts,
            high_alerts,
            medium_alerts,
            low_alerts,
        }
    }
}

/// Alert statistics
#[derive(Debug, Clone)]
pub struct AlertStatistics {
    /// Total number of alerts
    pub total_alerts: usize,
    /// Number of active alerts
    pub active_alerts: usize,
    /// Number of critical alerts
    pub critical_alerts: usize,
    /// Number of high severity alerts
    pub high_alerts: usize,
    /// Number of medium severity alerts
    pub medium_alerts: usize,
    /// Number of low severity alerts
    pub low_alerts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            id: "test-alert".to_string(),
            title: "Test Alert".to_string(),
            description: "This is a test alert".to_string(),
            severity: AlertSeverity::High,
            status: AlertStatus::Active,
            service_name: "test-service".to_string(),
            source: "test-source".to_string(),
            timestamp: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            metadata: HashMap::new(),
            tags: HashMap::new(),
        };

        assert_eq!(alert.id, "test-alert");
        assert_eq!(alert.severity, AlertSeverity::High);
        assert_eq!(alert.status, AlertStatus::Active);
    }

    #[test]
    fn test_alert_rule() {
        let rule = AlertRule {
            name: "test-rule".to_string(),
            description: "Test rule".to_string(),
            condition: AlertCondition::MetricThreshold {
                metric_name: "cpu_usage".to_string(),
                operator: ThresholdOperator::GreaterThan,
                value: 80.0,
                duration: Duration::from_secs(300),
            },
            severity: AlertSeverity::High,
            title_template: "High CPU Usage".to_string(),
            description_template: "CPU usage is above 80%".to_string(),
            cooldown: Duration::from_secs(300),
            enabled: true,
        };

        assert_eq!(rule.name, "test-rule");
        assert_eq!(rule.severity, AlertSeverity::High);
        assert!(rule.enabled);
    }

    #[tokio::test]
    async fn test_alert_manager() {
        let mut manager = AlertManager::new();
        
        let rule = AlertRule {
            name: "test-rule".to_string(),
            description: "Test rule".to_string(),
            condition: AlertCondition::MetricThreshold {
                metric_name: "cpu_usage".to_string(),
                operator: ThresholdOperator::GreaterThan,
                value: 80.0,
                duration: Duration::from_secs(300),
            },
            severity: AlertSeverity::High,
            title_template: "High CPU Usage".to_string(),
            description_template: "CPU usage is above 80%".to_string(),
            cooldown: Duration::from_secs(300),
            enabled: true,
        };

        manager.add_rule(rule);

        let metrics = HashMap::from([
            ("cpu_usage".to_string(), 90.0),
        ]);

        manager.evaluate_rules(&metrics).await.unwrap();

        let stats = manager.get_statistics();
        assert_eq!(stats.total_alerts, 0); // No alerts should be triggered due to cooldown
    }
}
