// ABOUTME: CLI commands for OAuth authentication with AI providers
// ABOUTME: Supports login, logout, status, and token refresh for Claude, OpenAI, Google, and xAI

use chrono::{DateTime, Utc};
use clap::Subcommand;
use colored::*;
use inquire::Select;
use orkee_auth::oauth::OAuthProvider;
use orkee_auth::{OAuthManager, OAuthToken};
use orkee_storage::EncryptionMode;
use std::fs;
use std::process::{self, Command, Stdio};

/// Claude API tokens are valid for 1 year from creation
const CLAUDE_TOKEN_VALIDITY_SECONDS: i64 = 365 * 24 * 60 * 60;

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Authenticate with an AI provider using OAuth
    Login {
        /// Provider to authenticate with (claude, openai, google, xai)
        provider: Option<String>,

        /// Import token from file (Claude only)
        #[arg(long)]
        file: Option<String>,

        /// Force re-authentication even if valid token exists
        #[arg(long)]
        force: bool,
    },

    /// Remove authentication for a provider
    Logout {
        /// Provider to logout from (claude, openai, google, xai, all)
        provider: String,
    },

    /// Show authentication status for all providers
    Status,

    /// Refresh authentication token for a provider
    Refresh {
        /// Provider to refresh (claude, openai, google, xai)
        provider: String,
    },
}

impl AuthCommands {
    pub async fn execute(&self) {
        match self {
            AuthCommands::Login {
                provider,
                file,
                force,
            } => login_command(provider.as_deref(), file.as_deref(), *force).await,
            AuthCommands::Logout { provider } => logout_command(provider).await,
            AuthCommands::Status => status_command().await,
            AuthCommands::Refresh { provider } => refresh_command(provider).await,
        }
    }
}

async fn login_command(provider_str: Option<&str>, file: Option<&str>, _force: bool) {
    // Handle Docker separately (not an OAuth provider)
    if let Some("docker") = provider_str {
        import_docker_credentials().await;
        return;
    }

    // Parse or prompt for OAuth provider
    let provider = match provider_str {
        Some(p) => match parse_provider(p) {
            Ok(provider) => provider,
            Err(e) => {
                eprintln!("{} {}", "✗".red().bold(), e);
                process::exit(1);
            }
        },
        None => match prompt_provider_selection() {
            Ok(provider) => provider,
            Err(e) => {
                eprintln!("{} {}", "✗".red().bold(), e);
                process::exit(1);
            }
        },
    };

    // Handle OAuth provider-specific authentication
    match provider {
        OAuthProvider::Claude => {
            if let Some(file_path) = file {
                import_claude_from_file(file_path).await;
            } else {
                import_claude_from_setup().await;
            }
        }
        OAuthProvider::OpenAI | OAuthProvider::Google | OAuthProvider::XAI => {
            eprintln!(
                "{} {} OAuth not yet implemented",
                "✗".red().bold(),
                provider.to_string().bold()
            );
            eprintln!();
            eprintln!("Please use API keys in Settings instead.");
            process::exit(1);
        }
    }
}

async fn logout_command(provider_str: &str) {
    // Initialize OAuth manager
    let oauth = match OAuthManager::new_default().await {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "{} Failed to initialize OAuth manager: {}",
                "✗".red().bold(),
                e
            );
            process::exit(1);
        }
    };
    let user_id = "default-user"; // TODO: Get actual user ID

    if provider_str.to_lowercase() == "all" {
        println!("{}", "🔓 Logging out from all providers...".bold().cyan());
        println!();

        let mut success_count = 0;
        let mut error_count = 0;

        for provider in OAuthProvider::all() {
            match oauth.logout(user_id, provider).await {
                Ok(_) => {
                    println!("  {} {}", "✓".green().bold(), provider);
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("  {} {}: {}", "✗".red().bold(), provider, e);
                    error_count += 1;
                }
            }
        }

        println!();
        println!(
            "Logged out from {} provider(s), {} error(s)",
            success_count.to_string().green().bold(),
            error_count
        );

        if error_count > 0 {
            process::exit(1);
        }
    } else {
        let provider = match parse_provider(provider_str) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{} {}", "✗".red().bold(), e);
                process::exit(1);
            }
        };

        println!(
            "{}",
            format!("🔓 Logging out from {}...", provider).bold().cyan()
        );

        match oauth.logout(user_id, provider).await {
            Ok(_) => {
                println!("{} Successfully logged out", "✓".green().bold());
            }
            Err(e) => {
                eprintln!("{} Logout failed: {}", "✗".red().bold(), e);
                process::exit(1);
            }
        }
    }
}

async fn status_command() {
    // Initialize OAuth manager
    let oauth = match OAuthManager::new_default().await {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "{} Failed to initialize OAuth manager: {}",
                "✗".red().bold(),
                e
            );
            process::exit(1);
        }
    };
    let user_id = "default-user"; // TODO: Get actual user ID

    println!("{}", "🔐 OAuth Authentication Status".bold().cyan());
    println!();

    match oauth.get_status(user_id).await {
        Ok(statuses) => {
            for status in statuses {
                let status_icon = if status.authenticated {
                    "✓".green().bold()
                } else {
                    "✗".red().bold()
                };

                let provider_name = format!("{:8}", status.provider.to_string());

                println!("  {} {}", status_icon, provider_name.bold());

                if status.authenticated {
                    if let Some(email) = status.account_email {
                        println!("        Account: {}", email.cyan());
                    }
                    if let Some(subscription) = status.subscription_type {
                        println!("        Subscription: {}", subscription.cyan());
                    }
                    if let Some(expires_at) = status.expires_at {
                        let expires = format_timestamp(expires_at);
                        let now = Utc::now().timestamp();
                        if expires_at < now {
                            println!("        Expires: {} {}", expires.red(), "(expired)".red());
                        } else {
                            println!("        Expires: {}", expires.green());
                        }
                    }
                } else {
                    println!("        {}", "Not authenticated".dimmed());
                }
                println!();
            }

            println!(
                "Use {} to authenticate",
                "orkee auth login <provider>".yellow()
            );
        }
        Err(e) => {
            eprintln!("{} Failed to get status: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    }
}

async fn refresh_command(provider_str: &str) {
    let provider = match parse_provider(provider_str) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    // Claude tokens cannot be refreshed
    if provider == OAuthProvider::Claude {
        eprintln!(
            "{} Claude tokens cannot be refreshed. They expire after 1 year.",
            "✗".red().bold()
        );
        eprintln!();
        eprintln!("To get a new token, run:");
        eprintln!("   {}", "orkee auth login claude".yellow());
        eprintln!();
        eprintln!("This will generate a new 1-year token.");
        process::exit(1);
    }

    eprintln!(
        "{} {} OAuth refresh not yet implemented",
        "✗".red().bold(),
        provider.to_string().bold()
    );
    eprintln!();
    eprintln!("Please use API keys in Settings instead.");
    process::exit(1);
}

async fn import_claude_from_setup() {
    println!(
        "{}",
        "🔐 Running 'claude setup-token' to generate authentication token..."
            .bold()
            .cyan()
    );
    println!("   This will open your browser for authentication.");
    println!();

    // Run claude setup-token
    let output = match Command::new("claude")
        .arg("setup-token")
        .stdin(Stdio::inherit()) // Allow browser interaction
        .stdout(Stdio::piped()) // Capture token output
        .stderr(Stdio::inherit()) // Show progress to user
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("{} Claude CLI not found", "✗".red().bold());
                eprintln!();
                eprintln!("Please install it first:");
                eprintln!("  {}", "npm install -g @anthropic-ai/claude-code".yellow());
                process::exit(1);
            } else {
                eprintln!(
                    "{} Failed to run 'claude setup-token': {}",
                    "✗".red().bold(),
                    e
                );
                process::exit(1);
            }
        }
    };

    if !output.status.success() {
        eprintln!(
            "{} claude setup-token failed. Please try again.",
            "✗".red().bold()
        );
        process::exit(1);
    }

    // Extract token from output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let token = match extract_claude_token(&stdout) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{} {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    // Store token
    if let Err(e) = store_claude_token(&token).await {
        eprintln!("{} Failed to store token: {}", "✗".red().bold(), e);
        process::exit(1);
    }

    println!();
    println!(
        "{} Claude authentication token imported successfully!",
        "✓".green().bold()
    );
    println!("   Token expires in 1 year.");

    show_encryption_warning().await;
}

async fn import_claude_from_file(file_path: &str) {
    println!(
        "{}",
        format!("📄 Importing Claude token from file: {}", file_path)
            .bold()
            .cyan()
    );

    let token = match fs::read_to_string(file_path) {
        Ok(content) => content.trim().to_string(),
        Err(e) => {
            eprintln!("{} Failed to read file: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    // Validate token format
    if !token.starts_with("sk-ant-oat01-") {
        eprintln!("{} Invalid token format", "✗".red().bold());
        eprintln!();
        eprintln!("Claude OAuth tokens should start with 'sk-ant-oat01-'.");
        eprintln!("If you have an API key (sk-ant-api03-), use Settings instead.");
        process::exit(1);
    }

    // Store token
    if let Err(e) = store_claude_token(&token).await {
        eprintln!("{} Failed to store token: {}", "✗".red().bold(), e);
        process::exit(1);
    }

    println!();
    println!(
        "{} Claude authentication token imported successfully!",
        "✓".green().bold()
    );
    println!("   Token expires in 1 year.");

    show_encryption_warning().await;
}

fn extract_claude_token(output: &str) -> Result<String, String> {
    // Look for token in output (appears after success message)
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("sk-ant-oat01-") {
            return Ok(trimmed.to_string());
        }
    }

    Err("Could not find token in command output.\n\
         The token should start with 'sk-ant-oat01-'.\n\
         Please try running 'claude setup-token' manually and use --file option."
        .to_string())
}

async fn store_claude_token(token: &str) -> Result<(), String> {
    let manager = OAuthManager::new_default()
        .await
        .map_err(|e| format!("Failed to initialize OAuth manager: {}", e))?;

    // Generate a simple ID for the token (first 21 chars of the token)
    let token_id = if token.len() >= 21 {
        token[..21].to_string()
    } else {
        token.to_string()
    };

    let oauth_token = OAuthToken {
        id: token_id,
        user_id: "default-user".to_string(), // TODO: Implement user system
        provider: "claude".to_string(),
        access_token: token.to_string(),
        refresh_token: None, // Claude tokens don't refresh
        expires_at: Utc::now().timestamp() + CLAUDE_TOKEN_VALIDITY_SECONDS,
        token_type: "Bearer".to_string(),
        scope: Some("model:claude account:read".to_string()),
        subscription_type: None, // Could detect later via API
        account_email: None,     // Could detect later via API
    };

    manager
        .import_token(oauth_token)
        .await
        .map_err(|e| format!("Failed to store token: {}", e))?;

    Ok(())
}

/// Show encryption security warning if using machine-based encryption
async fn show_encryption_warning() {
    // Get database path
    let db_path = match dirs::home_dir() {
        Some(home) => home.join(".orkee").join("orkee.db"),
        None => return, // Can't check, skip warning
    };

    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());

    // Connect to database to check encryption mode
    let pool = match sqlx::SqlitePool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(_) => return, // Can't check, skip warning
    };

    // Query encryption mode
    let mode: Option<String> =
        match sqlx::query_scalar("SELECT mode FROM encryption_settings WHERE id = 1")
            .fetch_optional(&pool)
            .await
        {
            Ok(mode) => mode,
            Err(_) => return, // Can't check, skip warning
        };

    // Only show warning for machine-based encryption
    if let Some(mode_str) = mode {
        if mode_str == EncryptionMode::Machine.to_string() {
            println!();
            println!("{}", "⚠️  SECURITY WARNING".yellow().bold());
            println!();
            println!(
                "   Tokens are encrypted with {}",
                "machine-based encryption".yellow()
            );
            println!(
                "   This provides {} - anyone with access to",
                "transport encryption only".yellow()
            );
            println!(
                "   {} can decrypt your tokens.",
                "~/.orkee/orkee.db".yellow()
            );
            println!();
            println!(
                "   For production use or shared machines, upgrade to password-based encryption:"
            );
            println!();
            println!("      {}", "orkee security set-password".green().bold());
            println!();
        }
    }
}

async fn import_docker_credentials() {
    println!("{}", "🐋 Docker Hub Authentication".bold().cyan());
    println!("This will run 'docker login' to authenticate with Docker Hub.");
    println!("Docker will store credentials securely in your system keychain.");
    println!();

    // Run docker login
    let status = Command::new("docker")
        .arg("login")
        .stdin(Stdio::inherit()) // Allow interactive input
        .stdout(Stdio::inherit()) // Show docker's output
        .stderr(Stdio::inherit()) // Show docker's errors
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => {
            println!();
            println!("{} Docker authentication successful!", "✓".green().bold());
            println!("   Credentials stored by Docker in your system keychain");
            println!("   You can now build and push images to Docker Hub");
        }
        Ok(exit_status) => {
            eprintln!();
            eprintln!(
                "{} Docker login failed with exit code: {}",
                "✗".red().bold(),
                exit_status.code().unwrap_or(-1)
            );
            process::exit(1);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("{} Docker not found", "✗".red().bold());
            eprintln!();
            eprintln!("Please install Docker first:");
            eprintln!("  https://docs.docker.com/get-docker/");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("{} Failed to run 'docker login': {}", "✗".red().bold(), e);
            process::exit(1);
        }
    }
}

fn parse_provider(provider_str: &str) -> Result<OAuthProvider, String> {
    provider_str
        .parse()
        .map_err(|_| format!("Unknown provider: {}", provider_str))
}

fn prompt_provider_selection() -> Result<OAuthProvider, String> {
    let providers = vec!["Claude", "OpenAI", "Google", "xAI", "Docker"];

    let selection = Select::new("Select AI provider:", providers)
        .prompt()
        .map_err(|e| format!("Selection cancelled: {}", e))?;

    parse_provider(&selection.to_lowercase())
}

fn format_timestamp(timestamp: i64) -> String {
    match DateTime::from_timestamp(timestamp, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        None => "Invalid date".to_string(),
    }
}
