//! Rate limiting implementation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use axiom_ai_core::{Result, AxiomError};

/// Rate limiter using token bucket algorithm
#[derive(Debug)]
pub struct TokenBucket {
    /// Maximum number of tokens
    capacity: u32,
    /// Current number of tokens
    tokens: u32,
    /// Rate of token refill (tokens per second)
    refill_rate: f64,
    /// Last refill time
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Try to consume tokens
    pub fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = (elapsed.as_secs_f64() * self.refill_rate) as u32;
        
        if tokens_to_add > 0 {
            self.tokens = std::cmp::min(self.capacity, self.tokens + tokens_to_add);
            self.last_refill = now;
        }
    }

    /// Get the number of available tokens
    pub fn available_tokens(&mut self) -> u32 {
        self.refill();
        self.tokens
    }

    /// Get the time until the next token is available
    pub fn time_until_next_token(&mut self) -> Duration {
        self.refill();
        
        if self.tokens > 0 {
            Duration::from_secs(0)
        } else {
            let tokens_needed = 1;
            let seconds_needed = tokens_needed as f64 / self.refill_rate;
            Duration::from_secs_f64(seconds_needed)
        }
    }
}

/// Rate limiter for a specific resource
#[derive(Debug)]
pub struct RateLimiter {
    /// Token bucket for requests
    request_bucket: TokenBucket,
    /// Token bucket for tokens
    token_bucket: Option<TokenBucket>,
    /// Maximum requests per minute
    requests_per_minute: Option<u32>,
    /// Maximum tokens per minute
    tokens_per_minute: Option<u32>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(
        requests_per_minute: Option<u32>,
        tokens_per_minute: Option<u32>,
    ) -> Self {
        let request_bucket = if let Some(rpm) = requests_per_minute {
            TokenBucket::new(rpm, rpm as f64 / 60.0)
        } else {
            TokenBucket::new(u32::MAX, f64::INFINITY)
        };

        let token_bucket = if let Some(tpm) = tokens_per_minute {
            Some(TokenBucket::new(tpm, tpm as f64 / 60.0))
        } else {
            None
        };

        Self {
            request_bucket,
            token_bucket,
            requests_per_minute,
            tokens_per_minute,
        }
    }

    /// Check if a request is allowed
    pub fn is_request_allowed(&mut self) -> bool {
        self.request_bucket.try_consume(1)
    }

    /// Check if a request with token count is allowed
    pub fn is_request_with_tokens_allowed(&mut self, token_count: u32) -> bool {
        if !self.is_request_allowed() {
            return false;
        }

        if let Some(ref mut bucket) = self.token_bucket {
            bucket.try_consume(token_count)
        } else {
            true
        }
    }

    /// Get the time until the next request is allowed
    pub fn time_until_next_request(&mut self) -> Duration {
        self.request_bucket.time_until_next_token()
    }

    /// Get the time until enough tokens are available
    pub fn time_until_tokens_available(&mut self, token_count: u32) -> Duration {
        if let Some(ref mut bucket) = self.token_bucket {
            let available = bucket.available_tokens();
            if available >= token_count {
                Duration::from_secs(0)
            } else {
                let needed = token_count - available;
                let seconds_needed = needed as f64 / bucket.refill_rate;
                Duration::from_secs_f64(seconds_needed)
            }
        } else {
            Duration::from_secs(0)
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Rate limiters by provider name
    limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
    /// Default rate limits
    default_requests_per_minute: Option<u32>,
    /// Default tokens per minute
    default_tokens_per_minute: Option<u32>,
}

impl RateLimitConfig {
    /// Create a new rate limit configuration
    pub fn new() -> Self {
        Self {
            limiters: Arc::new(RwLock::new(HashMap::new())),
            default_requests_per_minute: Some(60),
            default_tokens_per_minute: Some(10000),
        }
    }

    /// Set default rate limits
    pub fn with_default_limits(
        mut self,
        requests_per_minute: Option<u32>,
        tokens_per_minute: Option<u32>,
    ) -> Self {
        self.default_requests_per_minute = requests_per_minute;
        self.default_tokens_per_minute = tokens_per_minute;
        self
    }

    /// Add a rate limiter for a specific provider
    pub async fn add_limiter(
        &self,
        provider_name: String,
        requests_per_minute: Option<u32>,
        tokens_per_minute: Option<u32>,
    ) {
        let mut limiters = self.limiters.write().await;
        limiters.insert(
            provider_name,
            RateLimiter::new(requests_per_minute, tokens_per_minute),
        );
    }

    /// Check if a request is allowed for a provider
    pub async fn check_rate_limit(&self, provider_name: &str) -> Result<()> {
        let mut limiters = self.limiters.write().await;
        
        // Get or create limiter for this provider
        if !limiters.contains_key(provider_name) {
            limiters.insert(
                provider_name.to_string(),
                RateLimiter::new(
                    self.default_requests_per_minute,
                    self.default_tokens_per_minute,
                ),
            );
        }

        let limiter = limiters.get_mut(provider_name).unwrap();
        
        if !limiter.is_request_allowed() {
            let wait_time = limiter.time_until_next_request();
            return Err(AxiomError::RateLimit(format!(
                "Rate limit exceeded for provider '{}'. Try again in {:?}",
                provider_name, wait_time
            )));
        }

        Ok(())
    }

    /// Check if a request with token count is allowed
    pub async fn check_rate_limit_with_tokens(
        &self,
        provider_name: &str,
        token_count: u32,
    ) -> Result<()> {
        let mut limiters = self.limiters.write().await;
        
        // Get or create limiter for this provider
        if !limiters.contains_key(provider_name) {
            limiters.insert(
                provider_name.to_string(),
                RateLimiter::new(
                    self.default_requests_per_minute,
                    self.default_tokens_per_minute,
                ),
            );
        }

        let limiter = limiters.get_mut(provider_name).unwrap();
        
        if !limiter.is_request_with_tokens_allowed(token_count) {
            let wait_time = std::cmp::max(
                limiter.time_until_next_request(),
                limiter.time_until_tokens_available(token_count),
            );
            return Err(AxiomError::RateLimit(format!(
                "Rate limit exceeded for provider '{}' with {} tokens. Try again in {:?}",
                provider_name, token_count, wait_time
            )));
        }

        Ok(())
    }

    /// Get rate limit status for a provider
    pub async fn get_status(&self, provider_name: &str) -> Option<RateLimitStatus> {
        let mut limiters = self.limiters.write().await;
        
        if let Some(limiter) = limiters.get_mut(provider_name) {
            Some(RateLimitStatus {
                requests_per_minute: limiter.requests_per_minute,
                tokens_per_minute: limiter.tokens_per_minute,
                available_requests: limiter.request_bucket.available_tokens(),
                available_tokens: limiter.token_bucket.as_mut()
                    .map(|b| b.available_tokens())
                    .unwrap_or(u32::MAX),
                time_until_next_request: limiter.time_until_next_request(),
            })
        } else {
            None
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limit status for a provider
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    /// Maximum requests per minute
    pub requests_per_minute: Option<u32>,
    /// Maximum tokens per minute
    pub tokens_per_minute: Option<u32>,
    /// Available requests
    pub available_requests: u32,
    /// Available tokens
    pub available_tokens: u32,
    /// Time until next request is allowed
    pub time_until_next_request: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10, 1.0); // 10 tokens, 1 per second

        // Should allow consuming all tokens initially
        assert!(bucket.try_consume(10));
        assert!(!bucket.try_consume(1));

        // Wait for refill
        sleep(Duration::from_secs(1)).await;
        assert!(bucket.try_consume(1));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(Some(60), Some(1000)); // 60 RPM, 1000 TPM

        // Should allow requests initially
        assert!(limiter.is_request_allowed());
        assert!(limiter.is_request_with_tokens_allowed(100));

        // Consume all requests
        for _ in 0..60 {
            assert!(limiter.is_request_allowed());
        }
        assert!(!limiter.is_request_allowed());
    }

    #[tokio::test]
    async fn test_rate_limit_config() {
        let config = RateLimitConfig::new();

        // Should allow requests initially
        assert!(config.check_rate_limit("test").await.is_ok());

        // Add a limiter with low limits
        config.add_limiter("limited".to_string(), Some(1), Some(100)).await;

        // First request should succeed
        assert!(config.check_rate_limit("limited").await.is_ok());

        // Second request should fail
        assert!(config.check_rate_limit("limited").await.is_err());
    }
}
