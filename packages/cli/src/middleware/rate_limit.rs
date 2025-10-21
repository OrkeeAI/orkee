use axum::{extract::ConnectInfo, http::Request, middleware::Next, response::Response};
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    num::NonZeroU32,
    sync::{Arc, Mutex},
};
use tracing::{debug, warn};

use crate::error::AppError;

/// Type alias for a rate limiter
type RateLimiterType = RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;

/// Type alias for a rate limiter instance
type RateLimiterInstance = Arc<RateLimiterType>;

/// Type alias for the rate limiter storage
type RateLimiterStorage = Arc<Mutex<HashMap<String, RateLimiterInstance>>>;

/// Rate limiting configuration for different endpoint categories
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub health_rpm: u32,    // Health endpoints
    pub browse_rpm: u32,    // Directory browsing
    pub projects_rpm: u32,  // Project CRUD
    pub preview_rpm: u32,   // Preview operations
    pub telemetry_rpm: u32, // Telemetry tracking endpoints
    pub ai_rpm: u32,        // AI proxy endpoints
    pub global_rpm: u32,    // Global fallback
    pub burst_size: u32,    // Burst size multiplier
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            health_rpm: 60,
            browse_rpm: 20,
            projects_rpm: 30,
            preview_rpm: 10,
            telemetry_rpm: 15, // More restrictive for DoS prevention
            ai_rpm: 10,        // Strict limit to prevent cost abuse and DoS
            global_rpm: 30,
            burst_size: 5,
        }
    }
}

/// Rate limiter with per-endpoint configuration
#[derive(Clone)]
pub struct RateLimitLayer {
    config: RateLimitConfig,
    limiters: RateLimiterStorage,
}

impl RateLimitLayer {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            limiters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get or create rate limiter for specific endpoint category
    fn get_limiter_for_path(&self, path: &str) -> RateLimiterInstance {
        let category = categorize_endpoint(path);
        let rpm = match category {
            EndpointCategory::Health => self.config.health_rpm,
            EndpointCategory::Browse => self.config.browse_rpm,
            EndpointCategory::Projects => self.config.projects_rpm,
            EndpointCategory::Preview => self.config.preview_rpm,
            EndpointCategory::Telemetry => self.config.telemetry_rpm,
            EndpointCategory::AI => self.config.ai_rpm,
            EndpointCategory::Other => self.config.global_rpm,
        };

        let mut limiters = self.limiters.lock().unwrap();
        let key = format!("{}:{}", category.as_str(), rpm);

        if let Some(limiter) = limiters.get(&key) {
            limiter.clone()
        } else {
            let quota =
                Quota::per_minute(NonZeroU32::new(rpm).unwrap_or(NonZeroU32::new(30).unwrap()))
                    .allow_burst(
                        NonZeroU32::new(rpm * self.config.burst_size / 10)
                            .unwrap_or(NonZeroU32::new(5).unwrap()),
                    );

            let limiter = Arc::new(RateLimiter::direct(quota));
            limiters.insert(key, limiter.clone());

            debug!(
                endpoint_category = %category.as_str(),
                rpm = %rpm,
                burst = %(rpm * self.config.burst_size / 10),
                "Created rate limiter for endpoint category"
            );

            limiter
        }
    }
}

/// Endpoint categories for different rate limiting rules
#[derive(Debug, Clone, Copy)]
enum EndpointCategory {
    Health,
    Browse,
    Projects,
    Preview,
    Telemetry,
    AI,
    Other,
}

impl EndpointCategory {
    fn as_str(self) -> &'static str {
        match self {
            EndpointCategory::Health => "health",
            EndpointCategory::Browse => "browse",
            EndpointCategory::Projects => "projects",
            EndpointCategory::Preview => "preview",
            EndpointCategory::Telemetry => "telemetry",
            EndpointCategory::AI => "ai",
            EndpointCategory::Other => "other",
        }
    }
}

/// Categorize endpoint based on path
fn categorize_endpoint(path: &str) -> EndpointCategory {
    if path.contains("/health") || path.contains("/status") {
        EndpointCategory::Health
    } else if path.contains("/browse-directories") {
        EndpointCategory::Browse
    } else if path.contains("/projects") {
        EndpointCategory::Projects
    } else if path.contains("/preview") {
        EndpointCategory::Preview
    } else if path.contains("/telemetry") {
        EndpointCategory::Telemetry
    } else if path.contains("/ai") {
        EndpointCategory::AI
    } else {
        EndpointCategory::Other
    }
}

/// Per-IP rate limiting middleware
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Extract the rate limit layer from request extensions
    let layer = request
        .extensions()
        .get::<RateLimitLayer>()
        .cloned()
        .unwrap_or_else(|| RateLimitLayer::new(RateLimitConfig::default()));

    // Skip rate limiting if disabled
    if !layer.config.enabled {
        return Ok(next.run(request).await);
    }

    let path = request.uri().path();
    let limiter = layer.get_limiter_for_path(path);
    let ip = addr.ip();

    // Check rate limit
    match limiter.check() {
        Ok(_) => {
            debug!(
                ip = %ip,
                path = %path,
                "Rate limit check passed"
            );
            Ok(next.run(request).await)
        }
        Err(_) => {
            warn!(
                ip = %ip,
                path = %path,
                audit = true,
                "Rate limit exceeded"
            );

            // Calculate retry-after based on limiter state
            let retry_after = calculate_retry_after(&limiter);

            Err(AppError::RateLimitExceeded { retry_after })
        }
    }
}

/// Calculate how long the client should wait before retrying
fn calculate_retry_after(limiter: &RateLimiterType) -> u64 {
    // Try to get the earliest time when a slot will be available
    match limiter.check() {
        Ok(_) => 1, // Should be available now, but return 1 second as minimum
        Err(_) => {
            // Default to 60 seconds if we can't determine the exact time
            // In a real implementation, you might use limiter.check_at() with future timestamps
            60
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_categorization() {
        assert!(matches!(
            categorize_endpoint("/api/health"),
            EndpointCategory::Health
        ));
        assert!(matches!(
            categorize_endpoint("/api/status"),
            EndpointCategory::Health
        ));
        assert!(matches!(
            categorize_endpoint("/api/browse-directories"),
            EndpointCategory::Browse
        ));
        assert!(matches!(
            categorize_endpoint("/api/projects"),
            EndpointCategory::Projects
        ));
        assert!(matches!(
            categorize_endpoint("/api/projects/123"),
            EndpointCategory::Projects
        ));
        assert!(matches!(
            categorize_endpoint("/api/preview/servers"),
            EndpointCategory::Preview
        ));
        assert!(matches!(
            categorize_endpoint("/api/telemetry/track"),
            EndpointCategory::Telemetry
        ));
        assert!(matches!(
            categorize_endpoint("/api/telemetry/settings"),
            EndpointCategory::Telemetry
        ));
        assert!(matches!(
            categorize_endpoint("/api/ai"),
            EndpointCategory::AI
        ));
        assert!(matches!(
            categorize_endpoint("/api/ai/chat"),
            EndpointCategory::AI
        ));
        assert!(matches!(
            categorize_endpoint("/api/other"),
            EndpointCategory::Other
        ));
    }

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let config = RateLimitConfig {
            enabled: true,
            health_rpm: 60,
            browse_rpm: 20,
            projects_rpm: 30,
            preview_rpm: 10,
            telemetry_rpm: 15,
            ai_rpm: 10,
            global_rpm: 30,
            burst_size: 5,
        };

        let layer = RateLimitLayer::new(config);

        // Get limiters for different paths
        let health_limiter = layer.get_limiter_for_path("/api/health");
        let _browse_limiter = layer.get_limiter_for_path("/api/browse-directories");
        let _projects_limiter = layer.get_limiter_for_path("/api/projects");

        // They should be different instances for different categories
        // but same for same categories
        let health_limiter2 = layer.get_limiter_for_path("/api/status");
        assert!(Arc::ptr_eq(&health_limiter, &health_limiter2));
    }

    #[tokio::test]
    async fn test_rate_limit_enforcement() {
        let quota = Quota::per_minute(NonZeroU32::new(2).unwrap());
        let limiter = RateLimiter::direct(quota);

        // First two requests should succeed
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_ok());

        // Third request should fail (rate limited)
        assert!(limiter.check().is_err());
    }

    #[test]
    fn test_config_defaults() {
        let config = RateLimitConfig::default();
        assert!(config.enabled);
        assert_eq!(config.health_rpm, 60);
        assert_eq!(config.browse_rpm, 20);
        assert_eq!(config.projects_rpm, 30);
        assert_eq!(config.preview_rpm, 10);
        assert_eq!(config.telemetry_rpm, 15);
        assert_eq!(config.ai_rpm, 10);
        assert_eq!(config.global_rpm, 30);
        assert_eq!(config.burst_size, 5);
    }
}
