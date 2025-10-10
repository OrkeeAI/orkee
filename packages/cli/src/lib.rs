use axum::http::{header, HeaderValue, Method};
use axum_server::tls_rustls::RustlsConfig;
use std::{net::SocketAddr, time::Duration};
use tower_http::{
    cors::{AllowHeaders, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info};

pub mod api;
pub mod config;
pub mod dashboard;
pub mod error;
pub mod middleware;
pub mod tls;

#[cfg(test)]
mod tests;

use config::Config;

fn create_cors_origin(_allow_any_localhost: bool) -> AllowOrigin {
    // Always allow any localhost origin for development flexibility
    // This supports dynamic ports without configuration
    AllowOrigin::predicate(|origin: &HeaderValue, _: &_| {
        if let Ok(origin_str) = origin.to_str() {
            let allowed = origin_str.starts_with("http://localhost:")
                || origin_str.starts_with("http://127.0.0.1:")
                || origin_str.starts_with("http://[::1]:")
                || origin_str.starts_with("https://localhost:")
                || origin_str.starts_with("https://127.0.0.1:")
                || origin_str.starts_with("https://[::1]:")
                || origin_str == "tauri://localhost"
                || origin_str == "http://tauri.localhost"
                || origin_str == "https://tauri.localhost";

            if !allowed {
                eprintln!("🚫 CORS blocked origin: {}", origin_str);
            }
            allowed
        } else {
            false
        }
    })
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    run_server_with_options(None).await
}

pub async fn run_server_with_options(dashboard_path: Option<std::path::PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;

    // Log middleware configuration
    info!("Starting server with middleware configuration:");
    info!(
        "  Rate limiting: {} ({}rpm)",
        config.rate_limit.enabled, config.rate_limit.global_rpm
    );
    info!(
        "  Security headers: {} (HSTS: {})",
        config.security_headers_enabled, config.enable_hsts
    );
    info!("  Directory sandbox: {:?}", config.browse_sandbox_mode);
    info!(
        "  TLS: {} (auto-generate: {})",
        config.tls.enabled, config.tls.auto_generate
    );

    if config.tls.enabled {
        // TLS mode: run both HTTP (redirect) and HTTPS (main app) servers
        run_dual_server_mode(config, dashboard_path).await
    } else {
        // HTTP only mode
        run_http_server(config, dashboard_path).await
    }
}

async fn run_http_server(config: Config, dashboard_path: Option<std::path::PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_application_router(config.clone(), dashboard_path).await?;
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    info!("Starting HTTP server on {}", addr);
    println!("✅ HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn run_dual_server_mode(config: Config, dashboard_path: Option<std::path::PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize TLS manager
    let tls_manager = tls::TlsManager::new(config.tls.clone());
    let rustls_config = tls_manager.initialize().await?;

    // Create main application router
    let app = create_application_router(config.clone(), dashboard_path).await?;

    // Create HTTP redirect router (simpler router that just redirects to HTTPS)
    let redirect_app = create_redirect_router(config.clone()).await?;

    // HTTPS server (main application) on configured port
    let https_addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    // HTTP server (redirects only) on port 4000 (or port - 1 if custom port)
    let http_port = if config.port == 4001 {
        4000
    } else {
        config.port.saturating_sub(1)
    };
    let http_addr = SocketAddr::from(([127, 0, 0, 1], http_port));

    info!("Starting dual server mode:");
    info!("  HTTPS server (main): {}", https_addr);
    info!("  HTTP server (redirect): {}", http_addr);

    println!("✅ HTTPS server listening on {}", https_addr);
    println!("✅ HTTP redirect server listening on {}", http_addr);

    // Start both servers concurrently
    let https_server = start_https_server(https_addr, app, rustls_config);
    let http_server = start_http_redirect_server(http_addr, redirect_app);

    // Use tokio::select to run both servers and return if either fails
    tokio::select! {
        result = https_server => {
            error!("HTTPS server stopped: {:?}", result);
            result
        }
        result = http_server => {
            error!("HTTP redirect server stopped: {:?}", result);
            result
        }
    }
}

async fn start_https_server(
    addr: SocketAddr,
    app: axum::Router,
    rustls_config: RustlsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    axum_server::bind_rustls(addr, rustls_config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;
    Ok(())
}

async fn start_http_redirect_server(
    addr: SocketAddr,
    redirect_app: axum::Router,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        redirect_app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

async fn create_application_router(
    config: Config,
    dashboard_path: Option<std::path::PathBuf>,
) -> Result<axum::Router, Box<dyn std::error::Error>> {
    // Create CORS layer with specific headers only
    let allowed_headers = AllowHeaders::list([
        header::CONTENT_TYPE,
        header::ACCEPT,
        header::AUTHORIZATION, // For future API key
        header::HeaderName::from_static("x-api-key"),
    ]);

    // Create CORS origin configuration
    let cors_origin = create_cors_origin(config.cors_allow_any_localhost);

    let cors = CorsLayer::new()
        .allow_origin(cors_origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(allowed_headers)
        .allow_credentials(false) // Explicitly disable credentials for local use
        .max_age(Duration::from_secs(3600));

    // Create the router with all middleware layers (in order: outermost to innermost)
    let mut app_builder = api::create_router_with_options(dashboard_path).await;

    // Add CORS layer
    app_builder = app_builder.layer(cors);

    // Add tracing layer for request logging
    app_builder = app_builder.layer(TraceLayer::new_for_http());

    // Add rate limiting if enabled
    if config.rate_limit.enabled {
        info!(
            "Rate limiting enabled with {} global requests/minute",
            config.rate_limit.global_rpm
        );
        app_builder = app_builder.layer(axum::middleware::from_fn(
            middleware::rate_limit::rate_limit_middleware,
        ));
    }

    // Add security headers if enabled
    if config.security_headers_enabled {
        let security_layer = if config.enable_hsts {
            middleware::SecurityHeadersLayer::new().with_hsts()
        } else {
            middleware::SecurityHeadersLayer::new()
        };
        app_builder = app_builder.layer(security_layer);
        info!("Security headers enabled (HSTS: {})", config.enable_hsts);
    }

    // Add panic handler (outermost layer)
    let app = app_builder.layer(middleware::create_panic_handler());

    Ok(app)
}

async fn create_redirect_router(
    config: Config,
) -> Result<axum::Router, Box<dyn std::error::Error>> {
    use middleware::https_redirect::{https_redirect_middleware, HttpsRedirectConfig};

    // Create a minimal router that only handles redirects
    let redirect_config = HttpsRedirectConfig {
        enabled: true,
        https_port: config.port, // Redirect to the HTTPS port
        preserve_host: true,
    };

    let redirect_router = axum::Router::new()
        .fallback(|| async { "Redirecting to HTTPS..." })
        .layer(axum::middleware::from_fn(
            move |mut request: axum::http::Request<axum::body::Body>,
                  next: axum::middleware::Next| {
                let config = redirect_config.clone();
                async move {
                    request.extensions_mut().insert(config);
                    https_redirect_middleware(request, next).await
                }
            },
        ))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::create_panic_handler());

    Ok(redirect_router)
}
