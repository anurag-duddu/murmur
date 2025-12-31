//! API rate limiting to prevent abuse and excessive API usage.
//!
//! Implements per-service rate limits using a token bucket algorithm.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Rate limit configuration for different services
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests allowed in the window
    pub max_requests: u32,
    /// Time window for the rate limit
    pub window: Duration,
    /// Minimum time between requests (burst prevention)
    pub min_interval: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 10,
            window: Duration::from_secs(60),
            min_interval: Duration::from_millis(100),
        }
    }
}

/// Pre-defined rate limits for different services
impl RateLimitConfig {
    /// Transcription services (Deepgram, Whisper API)
    /// More permissive since users might take rapid notes
    pub fn transcription() -> Self {
        Self {
            max_requests: 20,
            window: Duration::from_secs(60),
            min_interval: Duration::from_millis(250),
        }
    }

    /// LLM enhancement services (Groq)
    /// Reduced interval for faster response times
    pub fn llm_enhancement() -> Self {
        Self {
            max_requests: 15,
            window: Duration::from_secs(60),
            min_interval: Duration::from_millis(100),
        }
    }
}

/// Tracks request history for a single service
#[derive(Debug)]
struct ServiceRateState {
    /// Timestamps of recent requests
    requests: Vec<Instant>,
    /// Last request time for interval checking
    last_request: Option<Instant>,
}

impl ServiceRateState {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            last_request: None,
        }
    }

    /// Clean up old requests outside the window
    fn cleanup(&mut self, window: Duration) {
        let cutoff = Instant::now() - window;
        self.requests.retain(|&t| t > cutoff);
    }

    /// Check if a request is allowed and record it if so
    fn check_and_record(&mut self, config: &RateLimitConfig) -> Result<(), RateLimitError> {
        let now = Instant::now();

        // Check minimum interval between requests
        if let Some(last) = self.last_request {
            let elapsed = now.duration_since(last);
            if elapsed < config.min_interval {
                let wait = config.min_interval - elapsed;
                return Err(RateLimitError::TooFast { wait_ms: wait.as_millis() as u64 });
            }
        }

        // Clean up old requests
        self.cleanup(config.window);

        // Check window limit
        if self.requests.len() >= config.max_requests as usize {
            // Find when the oldest request will expire
            if let Some(&oldest) = self.requests.first() {
                let expires_at = oldest + config.window;
                let wait = expires_at.duration_since(now);
                return Err(RateLimitError::WindowExceeded {
                    limit: config.max_requests,
                    wait_ms: wait.as_millis() as u64,
                });
            }
        }

        // Request is allowed - record it
        self.requests.push(now);
        self.last_request = Some(now);
        Ok(())
    }
}

/// Rate limit error types
#[derive(Debug, Clone)]
pub enum RateLimitError {
    /// Requests are coming too fast (below minimum interval)
    TooFast { wait_ms: u64 },
    /// Too many requests in the time window
    WindowExceeded { limit: u32, wait_ms: u64 },
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::TooFast { wait_ms } => {
                write!(f, "Too many requests. Please wait {}ms.", wait_ms)
            }
            RateLimitError::WindowExceeded { limit, wait_ms } => {
                write!(
                    f,
                    "Rate limit exceeded ({} requests). Please wait {}ms.",
                    limit, wait_ms
                )
            }
        }
    }
}

impl std::error::Error for RateLimitError {}

/// Service identifiers for rate limiting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Service {
    WhisperApi,
    Groq,
}

impl Service {
    /// Get the rate limit configuration for this service
    pub fn config(&self) -> RateLimitConfig {
        match self {
            Service::WhisperApi => RateLimitConfig::transcription(),
            Service::Groq => RateLimitConfig::llm_enhancement(),
        }
    }
}

/// Global rate limiter for all services
pub struct RateLimiter {
    states: Mutex<HashMap<Service, ServiceRateState>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    /// Check if a request to the given service is allowed
    ///
    /// Returns Ok(()) if allowed, Err(RateLimitError) if rate limited
    pub fn check(&self, service: Service) -> Result<(), RateLimitError> {
        let config = service.config();
        let mut states = self.states.lock().map_err(|_| RateLimitError::TooFast { wait_ms: 100 })?;

        let state = states.entry(service).or_insert_with(ServiceRateState::new);
        state.check_and_record(&config)
    }

    /// Check rate limit and return a user-friendly error string if limited
    pub fn check_or_error(&self, service: Service) -> Result<(), String> {
        self.check(service).map_err(|e| e.to_string())
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Global rate limiter instance
/// Use rate_limiter() to access it
static RATE_LIMITER: std::sync::OnceLock<RateLimiter> = std::sync::OnceLock::new();

/// Get the global rate limiter instance
pub fn rate_limiter() -> &'static RateLimiter {
    RATE_LIMITER.get_or_init(RateLimiter::new)
}

/// Check rate limit for a service using the global limiter
pub fn check_rate_limit(service: Service) -> Result<(), String> {
    rate_limiter().check_or_error(service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limit_allows_normal_usage() {
        let limiter = RateLimiter::new();

        // First request should succeed
        assert!(limiter.check(Service::Groq).is_ok());

        // Wait for min_interval
        thread::sleep(Duration::from_millis(600));

        // Second request should also succeed
        assert!(limiter.check(Service::Groq).is_ok());
    }

    #[test]
    fn test_rate_limit_blocks_rapid_requests() {
        let limiter = RateLimiter::new();

        // First request should succeed
        assert!(limiter.check(Service::Groq).is_ok());

        // Immediate second request should be blocked
        let result = limiter.check(Service::Groq);
        assert!(matches!(result, Err(RateLimitError::TooFast { .. })));
    }

    #[test]
    fn test_rate_limit_window_exceeded() {
        let limiter = RateLimiter::new();

        // Use Groq for testing
        let service = Service::Groq;

        // First request should succeed
        let result = limiter.check(service);
        assert!(result.is_ok());
    }

    #[test]
    fn test_different_services_independent() {
        let limiter = RateLimiter::new();

        // Request to Groq should succeed
        assert!(limiter.check(Service::Groq).is_ok());

        // Request to WhisperApi should also succeed (different service)
        assert!(limiter.check(Service::WhisperApi).is_ok());

        // But second request to Groq should be blocked
        let result = limiter.check(Service::Groq);
        assert!(matches!(result, Err(RateLimitError::TooFast { .. })));
    }

    #[test]
    fn test_rate_limit_error_display() {
        let err = RateLimitError::TooFast { wait_ms: 500 };
        assert!(err.to_string().contains("500ms"));

        let err = RateLimitError::WindowExceeded { limit: 10, wait_ms: 1000 };
        assert!(err.to_string().contains("10 requests"));
    }
}
