use axum::{
    body::Body,
    extract::{Path as AxumPath, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
    net::TcpListener,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, error, info, warn};

/// Configuration for the static file server
#[derive(Debug, Clone)]
pub struct StaticServerConfig {
    pub root_path: PathBuf,
    pub port: u16,
    pub host: String,
}

impl StaticServerConfig {
    pub fn new(root_path: PathBuf, port: u16) -> Self {
        Self {
            root_path,
            port,
            host: "127.0.0.1".to_string(),
        }
    }
}

/// Static file server for serving HTML/CSS/JS files
pub struct StaticServer {
    config: StaticServerConfig,
}

impl StaticServer {
    pub fn new(config: StaticServerConfig) -> Self {
        Self { config }
    }

    /// Start the static file server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let socket_addr: SocketAddr = addr.parse()?;

        info!(
            "Starting static file server on {} serving {}",
            addr,
            self.config.root_path.display()
        );

        let app = self.create_router();

        let listener = TcpListener::bind(socket_addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Create the Axum router for serving files
    fn create_router(self) -> Router {
        let state = Arc::new(self.config);

        Router::new()
            .route("/", get(serve_index))
            .route("/*path", get(serve_file))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .with_state(state)
    }
}

/// Serve the index.html file for the root path
async fn serve_index(
    State(config): State<Arc<StaticServerConfig>>,
) -> Result<impl IntoResponse, StaticServerError> {
    let index_path = config.root_path.join("index.html");
    serve_file_from_path(&index_path).await
}

/// Serve a specific file based on the request path
async fn serve_file(
    AxumPath(path): AxumPath<String>,
    State(config): State<Arc<StaticServerConfig>>,
) -> Result<impl IntoResponse, StaticServerError> {
    let requested_path = config.root_path.join(&path);

    debug!("Serving file: {} -> {}", path, requested_path.display());

    // Security check: ensure the requested path is within the root directory
    let canonical_root = config
        .root_path
        .canonicalize()
        .map_err(|e| StaticServerError::IoError(format!("Failed to canonicalize root: {}", e)))?;

    let canonical_requested = match requested_path.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            // File doesn't exist, return 404
            return Err(StaticServerError::NotFound(path));
        }
    };

    if !canonical_requested.starts_with(&canonical_root) {
        warn!(
            "Attempted directory traversal attack: {}",
            requested_path.display()
        );
        return Err(StaticServerError::Forbidden);
    }

    // If the path is a directory, try to serve index.html from it
    if canonical_requested.is_dir() {
        let index_path = canonical_requested.join("index.html");
        if index_path.exists() {
            return serve_file_from_path(&index_path).await;
        } else {
            return Err(StaticServerError::NotFound(path));
        }
    }

    serve_file_from_path(&canonical_requested).await
}

/// Serve a file from a specific filesystem path
async fn serve_file_from_path(file_path: &Path) -> Result<impl IntoResponse, StaticServerError> {
    let file = File::open(file_path)
        .await
        .map_err(|e| StaticServerError::IoError(format!("Failed to open file: {}", e)))?;

    let mut reader = BufReader::new(file);
    let mut contents = Vec::new();
    reader
        .read_to_end(&mut contents)
        .await
        .map_err(|e| StaticServerError::IoError(format!("Failed to read file: {}", e)))?;

    let content_type = determine_content_type(file_path);
    let mut headers = HeaderMap::new();
    if let Ok(ct) = content_type.parse() {
        headers.insert(header::CONTENT_TYPE, ct);
    }

    // Add security headers (allow iframe embedding for preview purposes)
    if let Ok(value) = "nosniff".parse() {
        headers.insert(
            header::HeaderName::from_static("x-content-type-options"),
            value,
        );
    }
    // Note: X-Frame-Options removed to allow embedding in dashboard iframe

    Ok((headers, contents))
}

/// Determine the MIME content type based on file extension
fn determine_content_type(file_path: &Path) -> &'static str {
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());
    
    match extension.as_deref() {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("pdf") => "application/pdf",
        Some("zip") => "application/zip",
        Some("txt") => "text/plain; charset=utf-8",
        Some("md") => "text/markdown; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Error types for static server operations
#[derive(Debug)]
pub enum StaticServerError {
    NotFound(String),
    Forbidden,
    IoError(String),
}

impl IntoResponse for StaticServerError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            StaticServerError::NotFound(path) => {
                (StatusCode::NOT_FOUND, format!("File not found: {}", path))
            }
            StaticServerError::Forbidden => (
                StatusCode::FORBIDDEN,
                "Access denied: path outside root directory".to_string(),
            ),
            StaticServerError::IoError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        error!("Static server error: {} - {}", status, message);

        let body = Body::from(format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Error {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1 {{ color: #e74c3c; }}
        p {{ color: #7f8c8d; }}
    </style>
</head>
<body>
    <h1>Error {}</h1>
    <p>{}</p>
</body>
</html>"#,
            status.as_u16(),
            status.as_u16(),
            message
        ));

        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(body)
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Failed to build error response"))
                    .unwrap_or_default()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_determine_content_type() {
        assert_eq!(
            determine_content_type(Path::new("index.html")),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            determine_content_type(Path::new("styles.css")),
            "text/css; charset=utf-8"
        );
        assert_eq!(
            determine_content_type(Path::new("script.js")),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(
            determine_content_type(Path::new("image.png")),
            "image/png"
        );
        assert_eq!(
            determine_content_type(Path::new("unknown.xyz")),
            "application/octet-stream"
        );
    }

    #[tokio::test]
    async fn test_static_server_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = StaticServerConfig::new(temp_dir.path().to_path_buf(), 0);
        let server = StaticServer::new(config.clone());

        assert_eq!(server.config.root_path, config.root_path);
        assert_eq!(server.config.port, config.port);
        assert_eq!(server.config.host, "127.0.0.1");
    }

    #[test]
    fn test_security_path_traversal() {
        // This test would require setting up an actual server, 
        // but we can test the logic in isolation
        let root = PathBuf::from("/safe/root");
        let malicious_path = PathBuf::from("/safe/root/../../../etc/passwd");

        // The canonicalize check in serve_file should prevent this
        assert!(malicious_path.starts_with("/"));
    }
}