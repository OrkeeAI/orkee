// ABOUTME: CLI commands for OAuth authentication with AI providers
// ABOUTME: Supports login, logout, status, and token refresh for Claude, OpenAI, Google, and xAI

use chrono::{DateTime, Utc};
use clap::Subcommand;
use colored::*;
use inquire::Select;
use orkee_auth::{OAuthManager, OAuthProvider};
use orkee_storage::EncryptionMode;
use std::process;

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Authenticate with an AI provider using OAuth
    Login {
        /// Provider to authenticate with (claude, openai, google, xai)
        provider: Option<String>,

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
            AuthCommands::Login { provider, force } => {
                login_command(provider.as_deref(), *force).await
            }
            AuthCommands::Logout { provider } => logout_command(provider).await,
            AuthCommands::Status => status_command().await,
            AuthCommands::Refresh { provider } => refresh_command(provider).await,
        }
    }
}

async fn login_command(provider_str: Option<&str>, force: bool) {
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

    // Get default user ID
    let user_id = "default-user"; // TODO: Get actual user ID from session

    // Parse or prompt for provider
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

    println!(
        "{}",
        format!("🔐 Authenticating with {}...", provider)
            .bold()
            .cyan()
    );
    println!();

    // Check if already authenticated
    if !force {
        match oauth.get_token(user_id, provider).await {
            Ok(Some(token)) if token.is_valid() => {
                println!(
                    "{} Already authenticated with {}",
                    "✓".green().bold(),
                    provider.to_string().bold()
                );
                if let Some(email) = token.account_email {
                    println!("  Account: {}", email.cyan());
                }
                if let Some(subscription) = token.subscription_type {
                    println!("  Subscription: {}", subscription.cyan());
                }
                println!();
                println!(
                    "  Use {} to re-authenticate",
                    "orkee login <provider> --force".yellow()
                );
                return;
            }
            Ok(_) => {
                // Token expired or invalid, continue with authentication
            }
            Err(e) => {
                eprintln!("{} Warning: {}", "⚠".yellow().bold(), e);
            }
        }
    }

    // Perform OAuth authentication
    println!("📱 Opening browser for authentication...");
    println!();

    match oauth.authenticate(provider, user_id).await {
        Ok(token) => {
            println!("{} Successfully authenticated!", "✓".green().bold());
            println!();
            if let Some(email) = token.account_email {
                println!("   Account: {}", email.cyan());
            }
            if let Some(subscription) = token.subscription_type {
                println!("   Subscription: {}", subscription.cyan());
            }
            println!("   Expires: {}", format_timestamp(token.expires_at).cyan());
            println!();
            println!(
                "You can now use {} with your {} account",
                "Orkee".bold(),
                provider.to_string().bold()
            );

            // Show encryption security warning
            show_encryption_warning().await;
        }
        Err(e) => {
            eprintln!("{} Authentication failed: {}", "✗".red().bold(), e);
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

            println!("Use {} to authenticate", "orkee login <provider>".yellow());
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

    println!(
        "{}",
        format!("🔄 Refreshing token for {}...", provider)
            .bold()
            .cyan()
    );

    match oauth.refresh_token(user_id, provider).await {
        Ok(token) => {
            println!("{} Token refreshed successfully!", "✓".green().bold());
            println!("   Expires: {}", format_timestamp(token.expires_at).cyan());
        }
        Err(e) => {
            eprintln!("{} Token refresh failed: {}", "✗".red().bold(), e);
            eprintln!();
            eprintln!(
                "Try re-authenticating with: {}",
                format!("orkee login {}", provider).yellow()
            );
            process::exit(1);
        }
    }
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

fn parse_provider(provider_str: &str) -> Result<OAuthProvider, String> {
    provider_str
        .parse()
        .map_err(|_| format!("Unknown provider: {}", provider_str))
}

fn prompt_provider_selection() -> Result<OAuthProvider, String> {
    let providers = vec!["Claude", "OpenAI", "Google", "xAI"];

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
