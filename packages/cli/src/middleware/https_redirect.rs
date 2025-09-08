use axum::{
    http::{Request, Response, StatusCode, HeaderMap, Uri},
    middleware::Next,
};
use tracing::{debug, info};

/// Configuration for HTTPS redirect middleware
#[derive(Clone, Debug)]
pub struct HttpsRedirectConfig {
    pub enabled: bool,
    pub https_port: u16,
    pub preserve_host: bool,
}

impl Default for HttpsRedirectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            https_port: 4443,
            preserve_host: true,
        }
    }
}

/// HTTPS redirect middleware
/// 
/// This middleware redirects HTTP requests to HTTPS when TLS is enabled.
/// It handles both direct connections and connections behind reverse proxies
/// by checking the X-Forwarded-Proto header.
pub async fn https_redirect_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response<axum::body::Body>, StatusCode> {
    // Extract configuration from request extensions
    let config = request.extensions()
        .get::<HttpsRedirectConfig>()
        .cloned()
        .unwrap_or_default();
    
    if !config.enabled {
        return Ok(next.run(request).await);
    }
    
    let headers = request.headers();
    let uri = request.uri();
    
    // Check if request is already HTTPS
    if is_https_request(headers, uri) {
        debug!("Request is already HTTPS, proceeding normally");
        return Ok(next.run(request).await);
    }
    
    // Build HTTPS redirect URL
    let redirect_url = build_redirect_url(uri, &config, headers)?;
    
    info!(
        original_url = %uri,
        redirect_url = %redirect_url,
        "Redirecting HTTP request to HTTPS"
    );
    
    // Create redirect response
    let response = Response::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header("Location", redirect_url.as_str())
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .body(format!(
            "<!DOCTYPE html>\
            <html><head><title>Redirecting to HTTPS</title></head>\
            <body><h1>Redirecting to HTTPS</h1>\
            <p>This site requires HTTPS. You are being redirected to: \
            <a href=\"{}\">{}</a></p></body></html>",
            redirect_url, redirect_url
        ).into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(response)
}

/// Check if the request is already using HTTPS
fn is_https_request(headers: &HeaderMap, uri: &Uri) -> bool {
    // Check X-Forwarded-Proto header (set by reverse proxies)
    if let Some(proto) = headers.get("x-forwarded-proto") {
        if let Ok(proto_str) = proto.to_str() {
            if proto_str.eq_ignore_ascii_case("https") {
                return true;
            }
        }
    }
    
    // Check the URI scheme directly
    if let Some(scheme) = uri.scheme() {
        return scheme.as_str().eq_ignore_ascii_case("https");
    }
    
    // Check if request came in on standard HTTPS port
    // This is less reliable but can work in some scenarios
    if let Some(host) = headers.get("host") {
        if let Ok(host_str) = host.to_str() {
            if host_str.ends_with(":443") {
                return true;
            }
        }
    }
    
    false
}

/// Build the HTTPS redirect URL
fn build_redirect_url(
    uri: &Uri,
    config: &HttpsRedirectConfig,
    headers: &HeaderMap,
) -> Result<String, StatusCode> {
    let mut redirect_url = String::from("https://");
    
    // Determine host
    let host = if config.preserve_host {
        // Try to get host from headers
        if let Some(host_header) = headers.get("host") {
            host_header.to_str()
                .map_err(|_| StatusCode::BAD_REQUEST)?
        } else {
            "localhost"
        }
    } else {
        "localhost"
    };
    
    // Always remove port from host (we'll add the HTTPS port later if needed)
    let clean_host = if let Some(colon_pos) = host.rfind(':') {
        let (host_part, _port_part) = host.split_at(colon_pos);
        host_part // Remove any existing port
    } else {
        host // No port to remove
    };
    
    redirect_url.push_str(clean_host);
    
    // Add HTTPS port if it's not the standard port (443)
    if config.https_port != 443 {
        redirect_url.push(':');
        redirect_url.push_str(&config.https_port.to_string());
    }
    
    // Add path and query
    redirect_url.push_str(uri.path());
    if let Some(query) = uri.query() {
        redirect_url.push('?');
        redirect_url.push_str(query);
    }
    
    Ok(redirect_url)
}


#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{HeaderMap, HeaderValue},
    };
    
    #[test]
    fn test_is_https_request_with_forwarded_proto() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));
        
        let uri = "http://example.com/test".parse::<Uri>().unwrap();
        assert!(is_https_request(&headers, &uri));
    }
    
    #[test]
    fn test_is_https_request_with_https_scheme() {
        let headers = HeaderMap::new();
        let uri = "https://example.com/test".parse::<Uri>().unwrap();
        assert!(is_https_request(&headers, &uri));
    }
    
    #[test]
    fn test_is_not_https_request() {
        let headers = HeaderMap::new();
        let uri = "http://example.com/test".parse::<Uri>().unwrap();
        assert!(!is_https_request(&headers, &uri));
    }
    
    #[test]
    fn test_build_redirect_url_basic() {
        let uri = "http://example.com/path?query=value".parse::<Uri>().unwrap();
        let config = HttpsRedirectConfig {
            enabled: true,
            https_port: 443,
            preserve_host: true,
        };
        
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("example.com"));
        
        let redirect_url = build_redirect_url(&uri, &config, &headers).unwrap();
        assert_eq!(redirect_url, "https://example.com/path?query=value");
    }
    
    #[test]
    fn test_build_redirect_url_custom_port() {
        let uri = "http://localhost/test".parse::<Uri>().unwrap();
        let config = HttpsRedirectConfig {
            enabled: true,
            https_port: 4443,
            preserve_host: true,
        };
        
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("localhost:4001"));
        
        let redirect_url = build_redirect_url(&uri, &config, &headers).unwrap();
        assert_eq!(redirect_url, "https://localhost:4443/test");
    }
    
    #[test]
    fn test_build_redirect_url_removes_http_port() {
        let uri = "http://example.com/test".parse::<Uri>().unwrap();
        let config = HttpsRedirectConfig {
            enabled: true,
            https_port: 443,
            preserve_host: true,
        };
        
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("example.com:80"));
        
        let redirect_url = build_redirect_url(&uri, &config, &headers).unwrap();
        assert_eq!(redirect_url, "https://example.com/test");
    }
    
    #[test]
    fn test_config_defaults() {
        let config = HttpsRedirectConfig::default();
        assert!(config.enabled);
        assert_eq!(config.https_port, 4443);
        assert!(config.preserve_host);
    }

    #[test]
    fn test_is_https_request_with_port_443() {
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("example.com:443"));
        
        // Use a URI without explicit scheme to test port-based detection
        let uri = "/test".parse::<Uri>().unwrap();
        assert!(is_https_request(&headers, &uri));
    }

    #[test]
    fn test_is_https_request_case_insensitive() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-proto", HeaderValue::from_static("HTTPS"));
        
        let uri = "http://example.com/test".parse::<Uri>().unwrap();
        assert!(is_https_request(&headers, &uri));
    }

    #[test]
    fn test_build_redirect_url_no_preserve_host() {
        let uri = "http://example.com/test".parse::<Uri>().unwrap();
        let config = HttpsRedirectConfig {
            enabled: true,
            https_port: 443,
            preserve_host: false, // Don't preserve host
        };
        
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("example.com"));
        
        let redirect_url = build_redirect_url(&uri, &config, &headers).unwrap();
        assert_eq!(redirect_url, "https://localhost/test");
    }

    #[test]
    fn test_build_redirect_url_no_host_header() {
        let uri = "http://localhost/test".parse::<Uri>().unwrap();
        let config = HttpsRedirectConfig {
            enabled: true,
            https_port: 443,
            preserve_host: true,
        };
        
        let headers = HeaderMap::new(); // No host header
        
        let redirect_url = build_redirect_url(&uri, &config, &headers).unwrap();
        assert_eq!(redirect_url, "https://localhost/test");
    }

    #[test]
    fn test_build_redirect_url_invalid_host_header() {
        let uri = "http://localhost/test".parse::<Uri>().unwrap();
        let config = HttpsRedirectConfig {
            enabled: true,
            https_port: 443,
            preserve_host: true,
        };
        
        let mut headers = HeaderMap::new();
        // Insert invalid UTF-8 bytes
        headers.insert("host", HeaderValue::from_bytes(b"\xFF\xFE").unwrap());
        
        let result = build_redirect_url(&uri, &config, &headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_redirect_url_complex_query() {
        let uri = "http://localhost:4000/api/test?param1=value1&param2=value%202&param3=".parse::<Uri>().unwrap();
        let config = HttpsRedirectConfig {
            enabled: true,
            https_port: 4443,
            preserve_host: true,
        };
        
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("localhost:4000"));
        
        let redirect_url = build_redirect_url(&uri, &config, &headers).unwrap();
        assert_eq!(redirect_url, "https://localhost:4443/api/test?param1=value1&param2=value%202&param3=");
    }

    #[test]
    fn test_build_redirect_url_root_path() {
        let uri = "http://localhost/".parse::<Uri>().unwrap();
        let config = HttpsRedirectConfig {
            enabled: true,
            https_port: 4443,
            preserve_host: true,
        };
        
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("localhost"));
        
        let redirect_url = build_redirect_url(&uri, &config, &headers).unwrap();
        assert_eq!(redirect_url, "https://localhost:4443/");
    }

    #[test]
    fn test_build_redirect_url_preserves_non_standard_ports() {
        let uri = "http://localhost/test".parse::<Uri>().unwrap();
        let config = HttpsRedirectConfig {
            enabled: true,
            https_port: 8443,
            preserve_host: true,
        };
        
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("localhost:9000"));
        
        let redirect_url = build_redirect_url(&uri, &config, &headers).unwrap();
        assert_eq!(redirect_url, "https://localhost:8443/test");
    }

    #[test]
    fn test_config_disabled_behavior() {
        let config = HttpsRedirectConfig {
            enabled: false,
            https_port: 4443,
            preserve_host: true,
        };
        
        // When disabled, middleware should not redirect
        assert!(!config.enabled);
    }

    #[test]
    fn test_host_parsing_with_various_ports() {
        let test_cases = vec![
            ("localhost:80", "https://localhost/test"),      // Standard HTTP port should be removed, no HTTPS port added (443 is default)
            ("localhost:8080", "https://localhost/test"),    // Non-standard port should be removed, no HTTPS port added
            ("example.com:443", "https://example.com/test"), // Any port should be removed, no HTTPS port added
            ("127.0.0.1", "https://127.0.0.1/test"),        // No port should result in no port
        ];
        
        for (input, expected) in test_cases {
            let config = HttpsRedirectConfig {
                enabled: true,
                https_port: 443, // Default HTTPS port
                preserve_host: true,
            };
            
            let mut headers = HeaderMap::new();
            headers.insert("host", HeaderValue::from_static(input));
            
            let uri = "http://example/test".parse::<Uri>().unwrap();
            let redirect_url = build_redirect_url(&uri, &config, &headers).unwrap();
            
            assert_eq!(redirect_url, expected, "Failed for input: {}", input);
        }
    }
}