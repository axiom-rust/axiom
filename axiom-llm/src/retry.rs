//! Retry logic and circuit breaker implementation

use std::time::Duration;
use std::future::Future;
use tokio::time::{sleep, timeout};

use axiom_core::{Result, AxiomError};

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Jitter factor to add randomness
    pub jitter_factor: f64,
    /// Timeout for each attempt
    pub attempt_timeout: Option<Duration>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            attempt_timeout: Some(Duration::from_secs(30)),
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Set the initial delay
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set the maximum delay
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set the backoff multiplier
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Set the jitter factor
    pub fn with_jitter_factor(mut self, factor: f64) -> Self {
        self.jitter_factor = factor;
        self
    }

    /// Set the attempt timeout
    pub fn with_attempt_timeout(mut self, timeout: Duration) -> Self {
        self.attempt_timeout = Some(timeout);
        self
    }

    /// Execute a function with retry logic
    pub async fn execute_with_retry<F, Fut, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut last_error = None;
        let mut delay = self.initial_delay;

        for attempt in 0..=self.max_retries {
            let result = if let Some(timeout_duration) = self.attempt_timeout {
                match timeout(timeout_duration, operation()).await {
                    Ok(result) => result,
                    Err(_) => Err(AxiomError::Timeout(format!(
                        "Operation timed out after {:?}",
                        timeout_duration
                    ))),
                }
            } else {
                operation().await
            };

            match result {
                Ok(value) => return Ok(value),
                Err(error) => {
                    // Check if the error is retryable
                    if !error.is_retryable() {
                        return Err(error);
                    }
                    
                    last_error = Some(error.clone());

                    // If this was the last attempt, return the error
                    if attempt >= self.max_retries {
                        return Err(error);
                    }

                    // Calculate delay with exponential backoff and jitter
                    let jitter = if self.jitter_factor > 0.0 {
                        let jitter_range = delay.as_millis() as f64 * self.jitter_factor;
                        let jitter = (rand::random::<f64>() - 0.5) * 2.0 * jitter_range;
                        Duration::from_millis(jitter as u64)
                    } else {
                        Duration::from_millis(0)
                    };

                    let total_delay = delay + jitter;
                    sleep(total_delay).await;

                    // Update delay for next iteration
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.backoff_multiplier) as u64
                        ),
                        self.max_delay,
                    );
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AxiomError::Internal("Retry exhausted without success".to_string())
        }))
    }
}

/// Circuit breaker for preventing cascading failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Maximum number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Timeout for the open state
    pub timeout: Duration,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Last failure time
    pub last_failure_time: Option<std::time::Instant>,
    /// Current state
    pub state: CircuitState,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests are allowed
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, limited requests are allowed
    HalfOpen,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            timeout,
            failure_count: 0,
            last_failure_time: None,
            state: CircuitState::Closed,
        }
    }

    /// Check if a request is allowed
    pub fn is_request_allowed(&self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    last_failure.elapsed() >= self.timeout
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    /// Record a failed request
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    /// Try to transition to half-open state
    pub fn try_half_open(&mut self) {
        if self.state == CircuitState::Open {
            if let Some(last_failure) = self.last_failure_time {
                if last_failure.elapsed() >= self.timeout {
                    self.state = CircuitState::HalfOpen;
                }
            }
        }
    }
}

/// Retry with circuit breaker
pub struct RetryWithCircuitBreaker {
    retry_config: RetryConfig,
    circuit_breaker: CircuitBreaker,
}

impl RetryWithCircuitBreaker {
    /// Create a new retry with circuit breaker
    pub fn new(retry_config: RetryConfig, circuit_breaker: CircuitBreaker) -> Self {
        Self {
            retry_config,
            circuit_breaker,
        }
    }

    /// Execute a function with retry and circuit breaker
    pub async fn execute_with_retry<F, Fut, T>(&mut self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        // Check if request is allowed
        if !self.circuit_breaker.is_request_allowed() {
            return Err(AxiomError::Internal("Circuit breaker is open".to_string()));
        }

        // Try to transition to half-open if needed
        self.circuit_breaker.try_half_open();

        let result = self.retry_config.execute_with_retry(|| {
            operation()
        }).await;

        match result {
            Ok(value) => {
                self.circuit_breaker.record_success();
                Ok(value)
            }
            Err(error) => {
                self.circuit_breaker.record_failure();
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_success() {
        let config = RetryConfig::new(3);
        let mut attempt = AtomicU32::new(0);

        let result = config.execute_with_retry(|| {
            let current = attempt.fetch_add(1, Ordering::SeqCst);
            if current < 2 {
                async { Err(AxiomError::Internal("Test error".to_string())) }
            } else {
                async { Ok("success".to_string()) }
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_failure() {
        let config = RetryConfig::new(2);

        let result = config.execute_with_retry(|| {
            async { Err(AxiomError::Internal("Test error".to_string())) }
        }).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let mut breaker = CircuitBreaker::new(2, Duration::from_millis(100));

        // Should allow requests initially
        assert!(breaker.is_request_allowed());

        // Record failures
        breaker.record_failure();
        assert!(breaker.is_request_allowed());

        breaker.record_failure();
        assert!(!breaker.is_request_allowed());

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(breaker.is_request_allowed());

        // Record success
        breaker.record_success();
        assert!(breaker.is_request_allowed());
    }
}
