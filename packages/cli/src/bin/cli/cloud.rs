//! Cloud sync CLI commands
//! 
//! This module provides CLI commands for Orkee Cloud functionality.

use clap::{Args, Subcommand};
use colored::*;

#[derive(Debug, Subcommand)]
pub enum CloudCommands {
    /// Enable Orkee Cloud
    Enable,
    /// Disable Orkee Cloud
    Disable,
    /// Sync projects to cloud
    Sync(SyncArgs),
    /// Restore projects from cloud
    Restore(RestoreArgs),
    /// List cloud projects
    List,
    /// Show cloud status
    Status,
    /// Login to Orkee Cloud
    Login,
    /// Logout from Orkee Cloud
    Logout,
}

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// Specific project ID to sync
    #[arg(long)]
    pub project: Option<String>,
    /// Force sync even if no changes
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct RestoreArgs {
    /// Specific project ID to restore
    #[arg(long)]
    pub project: Option<String>,
    /// Create local backup before restore
    #[arg(long)]
    pub backup: bool,
}

/// Handle cloud commands
pub async fn handle_cloud_command(command: CloudCommands) -> anyhow::Result<()> {
    // Initialize cloud with environment variables
    let supabase_url = std::env::var("NEXT_PUBLIC_SUPABASE_URL")
        .or_else(|_| std::env::var("SUPABASE_URL"))
        .map_err(|_| anyhow::anyhow!("SUPABASE_URL not found in environment"))?;
    
    let supabase_key = std::env::var("NEXT_PUBLIC_SUPABASE_ANON_KEY")
        .or_else(|_| std::env::var("SUPABASE_ANON_KEY"))
        .map_err(|_| anyhow::anyhow!("SUPABASE_ANON_KEY not found in environment"))?;

    // Configure cloud
    use orkee_cloud::config::CloudConfigBuilder;
    let config = CloudConfigBuilder::new()
        .project_url(supabase_url)
        .anon_key(supabase_key)
        .build()?;
    
    // Save configuration
    config.save().await?;

    // Initialize cloud
    let cloud = orkee_cloud::init().await?;

    match command {
        CloudCommands::Enable => {
            println!("🚀 {}", "Enabling Orkee Cloud".bold());
            println!();
            println!("Orkee Cloud provides:");
            println!("  • {} project backups", "Automatic".cyan());
            println!("  • {} sync", "Multi-device".cyan());
            println!("  • {} collaboration", "Team".cyan());
            println!("  • {} access to your projects", "Web".cyan());
            println!();

            cloud.enable().await?;
            
            println!();
            println!("✅ {} enabled!", "Orkee Cloud".green().bold());
            println!();
            println!("Your projects will now sync to the cloud.");
            println!("Free tier includes {} projects and {} storage.", "2".yellow(), "100MB".yellow());
            Ok(())
        }

        CloudCommands::Disable => {
            println!("🔒 {}", "Disabling Orkee Cloud".bold());
            cloud.disable().await?;
            println!("✅ {} disabled", "Orkee Cloud".green().bold());
            println!("Your projects are now local-only.");
            Ok(())
        }

        CloudCommands::Login => {
            println!("🔐 {}", "Logging in to Orkee Cloud".bold());
            let auth = cloud.auth();
            auth.login().await?;
            Ok(())
        }

        CloudCommands::Logout => {
            println!("👋 {}", "Logging out of Orkee Cloud".bold());
            let auth = cloud.auth();
            auth.logout().await?;
            Ok(())
        }

        CloudCommands::Sync(args) => {
            if !cloud.is_enabled().await {
                println!("❌ {} is not enabled", "Orkee Cloud".red());
                println!("Run {} to enable cloud sync", "orkee cloud enable".yellow());
                return Ok(());
            }

            println!("🔄 {}", "Syncing projects to cloud...".bold());
            
            let result = if let Some(project_id) = args.project {
                cloud.sync_project(&project_id).await?
            } else {
                cloud.sync().await?
            };

            println!("{}", result.summary());
            Ok(())
        }

        CloudCommands::Restore(_args) => {
            if !cloud.is_enabled().await {
                println!("❌ {} is not enabled", "Orkee Cloud".red());
                println!("Run {} to enable cloud sync", "orkee cloud enable".yellow());
                return Ok(());
            }

            println!("📥 {}", "Restoring projects from cloud...".bold());
            
            let projects = cloud.restore().await?;
            
            if projects.is_empty() {
                println!("No projects found in cloud.");
            } else {
                println!("Found {} projects:", projects.len());
                for project in projects {
                    println!("  • {} - {}", 
                        project.name.green(), 
                        project.last_synced_at
                            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_else(|| "never synced".to_string())
                    );
                }
            }
            Ok(())
        }

        CloudCommands::List => {
            if !cloud.is_enabled().await {
                println!("❌ {} is not enabled", "Orkee Cloud".red());
                println!("Run {} to enable cloud sync", "orkee cloud enable".yellow());
                return Ok(());
            }

            println!("📋 {}", "Cloud projects:".bold());
            
            let projects = cloud.list().await?;
            
            if projects.is_empty() {
                println!("No projects in cloud.");
            } else {
                println!();
                for project in projects {
                    let sync_status = match project.sync_status.as_str() {
                        "synced" => "✓".green(),
                        "pending" => "⏳".yellow(),
                        "conflict" => "⚠".red(),
                        _ => "?".white(),
                    };
                    
                    println!("{} {} - {}", 
                        sync_status,
                        project.name.bold(), 
                        project.description.as_deref().unwrap_or("No description")
                    );
                    
                    if let Some(synced_at) = project.last_synced_at {
                        println!("    Last synced: {}", 
                            synced_at.format("%Y-%m-%d %H:%M").to_string().dimmed()
                        );
                    }
                }
            }
            Ok(())
        }

        CloudCommands::Status => {
            println!("☁️  {}", "Orkee Cloud Status".bold());
            println!();

            let status = cloud.status().await?;
            
            if !status.enabled {
                println!("Status: {}", "Disabled".red());
                println!("Run {} to enable cloud sync", "orkee cloud enable".yellow());
            } else if !status.authenticated {
                println!("Status: {}", "Not authenticated".yellow());
                println!("Run {} to authenticate", "orkee cloud login".yellow());
            } else {
                println!("Status: {}", "Enabled".green());
                println!();
                
                // Subscription info
                let tier = &status.subscription.tier;
                println!("📦 {} Tier", "Subscription:".bold());
                println!("  Tier: {}", tier.display_name().cyan());
                println!("  Limits: {}", status.subscription.describe_limits());
                
                // Features
                println!();
                println!("✨ {}:", "Features".bold());
                let check = |enabled: bool| if enabled { "✓".green() } else { "✗".red() };
                println!("  {} Auto-sync", check(status.subscription.auto_sync_enabled));
                println!("  {} Real-time sync", check(status.subscription.realtime_enabled));
                println!("  {} Team collaboration", check(status.subscription.collaboration_enabled));
                
                // Usage
                if let Some(usage) = &status.usage {
                    println!();
                    println!("📊 {}:", "Usage".bold());
                    println!("  Projects: {}/{}", 
                        usage.project_count,
                        if status.subscription.project_limit < 0 { 
                            "unlimited".to_string() 
                        } else { 
                            status.subscription.project_limit.to_string() 
                        }
                    );
                    println!("  Storage: {}MB/{}MB", 
                        usage.used_mb,
                        if status.subscription.storage_limit_mb < 0 { 
                            "unlimited".to_string() 
                        } else { 
                            status.subscription.storage_limit_mb.to_string() 
                        }
                    );
                }
            }
            Ok(())
        }
    }
}

/// Print cloud help information
pub fn print_help() {
    println!("{}", "Orkee Cloud Commands".bold().cyan());
    println!();
    println!("  {} - Enable cloud sync", "orkee cloud enable".yellow());
    println!("  {} - Disable cloud sync", "orkee cloud disable".yellow());
    println!("  {} - Sync all projects", "orkee cloud sync".yellow());
    println!("  {} - Sync specific project", "orkee cloud sync --project <id>".yellow());
    println!("  {} - Restore from cloud", "orkee cloud restore".yellow());
    println!("  {} - List cloud projects", "orkee cloud list".yellow());
    println!("  {} - Show cloud status", "orkee cloud status".yellow());
    println!("  {} - Login to cloud", "orkee cloud login".yellow());
    println!("  {} - Logout from cloud", "orkee cloud logout".yellow());
    println!();
    println!("Cloud sync requires authentication and an active subscription.");
    println!("Free tier includes 2 projects and 100MB storage.");
}