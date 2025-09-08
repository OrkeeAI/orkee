use axum::{
    http::{HeaderValue, Request, Response},
    middleware::Next,
};
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Security headers middleware to add essential security headers to all responses
#[derive(Clone)]
pub struct SecurityHeadersLayer {
    enable_hsts: bool,
}

impl SecurityHeadersLayer {
    pub fn new() -> Self {
        Self { enable_hsts: false }
    }

    /// Enable HSTS (only use when HTTPS is properly configured)
    pub fn with_hsts(mut self) -> Self {
        self.enable_hsts = true;
        self
    }
}

impl Default for SecurityHeadersLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersService<S>;

    fn layer(&self, service: S) -> Self::Service {
        SecurityHeadersService {
            service,
            enable_hsts: self.enable_hsts,
        }
    }
}

/// Service that applies security headers
#[derive(Clone)]
pub struct SecurityHeadersService<S> {
    service: S,
    enable_hsts: bool,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for SecurityHeadersService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = SecurityHeadersFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let enable_hsts = self.enable_hsts;
        let future = self.service.call(request);
        SecurityHeadersFuture {
            future,
            enable_hsts,
        }
    }
}

/// Future that adds security headers to the response
#[pin_project::pin_project]
pub struct SecurityHeadersFuture<F> {
    #[pin]
    future: F,
    enable_hsts: bool,
}

impl<F, ResBody, E> std::future::Future for SecurityHeadersFuture<F>
where
    F: std::future::Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let response = std::task::ready!(this.future.poll(cx))?;

        let mut response = response;
        let headers = response.headers_mut();

        // Essential security headers for all responses
        add_security_headers(headers, *this.enable_hsts);

        Poll::Ready(Ok(response))
    }
}

/// Add all security headers to the response
fn add_security_headers(headers: &mut axum::http::HeaderMap, enable_hsts: bool) {
    // Prevent MIME type sniffing
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));

    // XSS protection (legacy but still useful)
    headers.insert(
        "x-xss-protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer policy
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Content Security Policy - restrictive but allows development
    let csp = "default-src 'self'; \
               script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
               style-src 'self' 'unsafe-inline'; \
               img-src 'self' data: https:; \
               connect-src 'self' ws: wss:; \
               font-src 'self'; \
               object-src 'none'; \
               base-uri 'self'; \
               form-action 'self'";

    headers.insert(
        "content-security-policy",
        HeaderValue::from_str(csp)
            .unwrap_or_else(|_| HeaderValue::from_static("default-src 'self'")),
    );

    // Permissions Policy - disable potentially dangerous features
    let permissions_policy = "geolocation=(), \
                             microphone=(), \
                             camera=(), \
                             payment=(), \
                             usb=(), \
                             magnetometer=(), \
                             gyroscope=(), \
                             accelerometer=()";

    headers.insert(
        "permissions-policy",
        HeaderValue::from_str(permissions_policy).unwrap_or_else(|_| {
            HeaderValue::from_static("geolocation=(), microphone=(), camera=()")
        }),
    );

    // HSTS - only enable when HTTPS is properly configured
    if enable_hsts {
        headers.insert(
            "strict-transport-security",
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        );
    }

    // Remove server information leakage
    headers.remove("server");
}

/// Axum middleware function for easier integration
pub async fn add_security_headers_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Response<axum::body::Body> {
    let mut response = next.run(request).await;
    add_security_headers(response.headers_mut(), false);
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get, Router};
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "Hello, World!"
    }

    #[tokio::test]
    async fn test_security_headers_applied() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(SecurityHeadersLayer::new());

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        let headers = response.headers();

        // Check all security headers are present
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
        assert_eq!(headers.get("x-xss-protection").unwrap(), "1; mode=block");
        assert_eq!(
            headers.get("referrer-policy").unwrap(),
            "strict-origin-when-cross-origin"
        );

        // CSP should be present
        assert!(headers.get("content-security-policy").is_some());
        let csp = headers
            .get("content-security-policy")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(csp.contains("default-src 'self'"));

        // Permissions policy should be present
        assert!(headers.get("permissions-policy").is_some());

        // Server header should be removed (if it was there)
        assert!(headers.get("server").is_none());
    }

    #[tokio::test]
    async fn test_hsts_header_when_enabled() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(SecurityHeadersLayer::new().with_hsts());

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        let headers = response.headers();

        // HSTS should be present when explicitly enabled
        assert!(headers.get("strict-transport-security").is_some());
        let hsts = headers
            .get("strict-transport-security")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(hsts.contains("max-age=31536000"));
        assert!(hsts.contains("includeSubDomains"));
    }

    #[tokio::test]
    async fn test_no_hsts_header_by_default() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(SecurityHeadersLayer::new());

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        let headers = response.headers();

        // HSTS should NOT be present by default (for local development)
        assert!(headers.get("strict-transport-security").is_none());
    }
}
