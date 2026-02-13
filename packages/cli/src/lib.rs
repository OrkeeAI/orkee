use axum::http::{header, HeaderValue, Method};
use axum_server::tls_rustls::RustlsConfig;
use colored::Colorize;
use std::{net::SocketAddr, time::Duration};
use tower_http::{
    cors::{AllowHeaders, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info};

pub mod api;
pub mod auth;
pub mod config;
pub mod dashboard;
pub mod error;
pub mod middleware;
pub mod telemetry;
pub mod tls;

#[cfg(test)]
mod tests;

use config::Config;

fn create_cors_origin(allow_any_localhost: bool, configured_origin: &str) -> AllowOrigin {
    if allow_any_localhost {
        // Allow any localhost origin for development flexibility
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
                    error!("CORS blocked origin: {}", origin_str);
                }
                allowed
            } else {
                false
            }
        })
    } else {
        // Strict mode: only allow the configured origin and Tauri origins
        let allowed = configured_origin.to_string();
        AllowOrigin::predicate(move |origin: &HeaderValue, _: &_| {
            if let Ok(origin_str) = origin.to_str() {
                let is_allowed = origin_str == allowed
                    || origin_str == "tauri://localhost"
                    || origin_str == "http://tauri.localhost"
                    || origin_str == "https://tauri.localhost";

                if !is_allowed {
                    error!("CORS blocked origin: {}", origin_str);
                }
                is_allowed
            } else {
                false
            }
        })
    }
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    run_server_with_options(None).await
}

pub async fn run_server_with_options(
    dashboard_path: Option<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber if not already initialized
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

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

/// Initialize API token on first startup
/// Generates a default token if none exist, logs it once, and stores it in ~/.orkee/api-token
async fn initialize_api_token() {
    match orkee_projects::DbState::init().await {
        Ok(db_state) => {
            // Check if any active tokens exist
            match db_state.token_storage.count_active_tokens().await {
                Ok(0) => {
                    // No tokens exist - generate default token
                    match db_state
                        .token_storage
                        .create_token("Default API Token")
                        .await
                    {
                        Ok(token_gen) => {
                            // Token file path
                            let token_file = orkee_projects::orkee_dir().join("api-token");

                            // Check if file already exists (shouldn't happen, but be safe)
                            if token_file.exists() {
                                info!("API token file already exists at: {}", token_file.display());
                            } else {
                                // Write token to file with secure permissions
                                match std::fs::write(&token_file, &token_gen.token) {
                                    Ok(_) => {
                                        // Set file permissions to 0600 (owner read/write only)
                                        #[cfg(unix)]
                                        {
                                            use std::os::unix::fs::PermissionsExt;
                                            let mut perms = std::fs::metadata(&token_file)
                                                .expect("Failed to get file metadata")
                                                .permissions();
                                            perms.set_mode(0o600);
                                            let _ = std::fs::set_permissions(&token_file, perms);
                                        }

                                        println!("\n{}", "🔑 API Token Generated".green().bold());
                                        println!("   Token: {}", token_gen.token.cyan().bold());
                                        println!(
                                            "   Stored in: {}",
                                            token_file.display().to_string().yellow()
                                        );
                                        println!("\n   {} This token is required for API authentication.", "IMPORTANT:".red().bold());
                                        println!("   Keep it secure and do not share it.");
                                        println!(
                                            "   The dashboard will automatically use this token.\n"
                                        );
                                        info!("API token generated and stored successfully");
                                    }
                                    Err(e) => {
                                        error!("Failed to write API token file: {}", e);
                                        println!(
                                            "\n{} Failed to write API token to file: {}",
                                            "⚠️".yellow(),
                                            e
                                        );
                                        println!(
                                            "   Your token: {}",
                                            token_gen.token.cyan().bold()
                                        );
                                        println!("   Please save this token manually.\n");
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to generate API token: {}", e);
                        }
                    }
                }
                Ok(_) => {
                    // Tokens exist - check if token file exists
                    let token_file = orkee_projects::orkee_dir().join("api-token");
                    if !token_file.exists() {
                        info!("API tokens exist in database but token file is missing");
                        println!("\n{} API token file not found", "⚠️".yellow());
                        println!("   If you need a new token, use: orkee tokens regenerate\n");
                    }
                }
                Err(e) => {
                    error!("Failed to check for existing API tokens: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to initialize database for token check: {}", e);
        }
    }
}

/// Check if there are API keys in environment variables that should be migrated to the database
async fn check_api_key_migration() {
    match orkee_projects::DbState::init().await {
        Ok(db_state) => {
            match orkee_projects::UserStorage::new(db_state.pool.clone()) {
                Ok(user_storage) => {
                    match user_storage.check_env_key_migration("default-user").await {
                        Ok(keys_to_migrate) if !keys_to_migrate.is_empty() => {
                            // Show migration notice
                            println!(
                                "\n⚠️  {} detected API keys in environment variables:",
                                "MIGRATION NOTICE:".yellow().bold()
                            );
                            println!(
                                "   Found API keys for: {}",
                                keys_to_migrate.join(", ").cyan()
                            );
                            println!("\n   These keys are currently being used from environment variables, but should");
                            println!("   be stored securely in the database for better security and persistence.");
                            println!("\n   {} You can manage your API keys in the Settings page of the dashboard.", "Recommendation:".green().bold());
                            println!("   Navigate to Settings > API Keys to save your credentials to the database.");
                            println!("   Once saved, you can remove the environment variables.\n");
                        }
                        Ok(_) => {
                            // No migration needed
                        }
                        Err(e) => {
                            // Log error but don't fail server startup
                            tracing::debug!("Could not check for API key migration: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!(
                        "Could not initialize user storage for migration check: {}",
                        e
                    );
                }
            }
        }
        Err(e) => {
            tracing::debug!("Could not initialize database for migration check: {}", e);
        }
    }
}

async fn run_http_server(
    config: Config,
    dashboard_path: Option<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_application_router(config.clone(), dashboard_path).await?;

    // Initialize API token if needed
    initialize_api_token().await;

    // Check for API key migration from environment variables to database
    check_api_key_migration().await;

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

async fn run_dual_server_mode(
    config: Config,
    dashboard_path: Option<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize TLS manager
    let tls_manager = tls::TlsManager::new(config.tls.clone());
    let rustls_config = tls_manager.initialize().await?;

    // Create main application router
    let app = create_application_router(config.clone(), dashboard_path).await?;

    // Initialize API token if needed
    initialize_api_token().await;

    // Check for API key migration from environment variables to database
    check_api_key_migration().await;

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
        header::USER_AGENT,
        header::HeaderName::from_static("x-api-key"),
        header::HeaderName::from_static("x-api-token"), // API token authentication
        header::HeaderName::from_static("x-csrf-token"), // CSRF protection
        header::HeaderName::from_static("anthropic-version"), // For Anthropic API proxy
        header::HeaderName::from_static("anthropic-dangerous-direct-browser-access"), // For Anthropic SDK browser safety check
    ]);

    // Create CORS origin configuration
    let cors_origin = create_cors_origin(config.cors_allow_any_localhost, &config.cors_origin);

    let cors = CorsLayer::new()
        .allow_origin(cors_origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(allowed_headers)
        .allow_credentials(false) // Explicitly disable credentials for local use
        .max_age(Duration::from_secs(3600));

    // Create the router with all middleware layers
    // IMPORTANT: In Axum, layers are applied in REVERSE order (last added = first executed)
    // Execution order: CORS → Security → Tracing → Rate Limit → Auth → CSRF → Handlers
    let (mut app_builder, db_state) =
        crate::api::create_router_with_options(dashboard_path, None).await;

    // Create CSRF layer for CSRF protection
    let csrf_layer = middleware::CsrfLayer::new();
    info!("CSRF protection enabled");

    // Add CSRF protection middleware (runs just before handlers)
    app_builder = app_builder.layer(axum::middleware::from_fn(middleware::csrf::csrf_middleware));
    info!("CSRF middleware enabled for password management endpoints");

    // Add CSRF layer as extension (available to all handlers and middleware)
    // Extensions must be added AFTER the middleware that uses them
    app_builder = app_builder.layer(axum::Extension(csrf_layer));

    // Add API token authentication middleware
    app_builder = app_builder.layer(axum::middleware::from_fn_with_state(
        db_state.clone(),
        middleware::api_token_middleware,
    ));
    info!("API token authentication middleware enabled");

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

    // Add tracing layer for request logging
    app_builder = app_builder.layer(TraceLayer::new_for_http());

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

    // Add CORS layer (outermost - runs first to handle OPTIONS preflight)
    app_builder = app_builder.layer(cors);

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
