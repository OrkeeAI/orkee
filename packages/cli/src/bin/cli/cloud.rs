//! Cloud sync CLI commands
//!
//! This module provides CLI commands for Orkee Cloud functionality.

use clap::{Args, Subcommand};
use colored::*;
use orkee_projects::manager::ProjectsManager;

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
    /// Check for sync conflicts
    Conflicts(ConflictsArgs),
    /// Push incremental changes
    Push(PushArgs),
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

#[derive(Debug, Args)]
pub struct ConflictsArgs {
    /// Project ID to check for conflicts
    #[arg(long)]
    pub project: String,
}

#[derive(Debug, Args)]
pub struct PushArgs {
    /// Specific project ID to push
    #[arg(long)]
    pub project: Option<String>,
    /// Push only changed fields
    #[arg(long)]
    pub incremental: bool,
}

/// Handle cloud commands
pub async fn handle_cloud_command(command: CloudCommands) -> anyhow::Result<()> {
    #[cfg(not(feature = "cloud"))]
    {
        println!("❌ {} feature is not enabled", "Cloud".red());
        println!(
            "Build with {} to enable cloud functionality",
            "--features cloud".yellow()
        );
        return Ok(());
    }

    #[cfg(feature = "cloud")]
    {
        use orkee_cloud::{api::CloudProject as ApiCloudProject, CloudClient};

        // Initialize cloud client
        let api_url = std::env::var("ORKEE_CLOUD_API_URL")
            .unwrap_or_else(|_| "https://api.orkee.ai".to_string());

        let mut cloud_client = match CloudClient::new(api_url).await {
            Ok(client) => client,
            Err(e) => {
                println!("❌ Failed to initialize cloud client: {}", e);
                return Ok(());
            }
        };

        match command {
            CloudCommands::Login => {
                println!("🔐 {}", "Orkee Cloud Authentication".bold());

                match cloud_client.login().await {
                    Ok(token_info) => {
                        println!(
                            "✅ Successfully logged in as {}",
                            token_info.user_name.green()
                        );
                        println!("🎉 Orkee Cloud is now ready to use!");
                    }
                    Err(e) => {
                        println!("❌ Authentication failed: {}", e);
                        println!("💡 Make sure you have access to Orkee Cloud and try again.");
                    }
                }
            }

            CloudCommands::Logout => {
                println!("👋 {}", "Orkee Cloud Logout".bold());

                match cloud_client.logout().await {
                    Ok(_) => {
                        println!("✅ Successfully logged out from Orkee Cloud");
                    }
                    Err(e) => {
                        println!("❌ Logout failed: {}", e);
                    }
                }
            }

            CloudCommands::Status => {
                println!("☁️  {}", "Orkee Cloud Status".bold());
                println!();

                match cloud_client.get_status().await {
                    Ok(status) => {
                        if status.authenticated {
                            println!("Status: {}", "✅ Authenticated".green());
                            if let Some(email) = &status.user_email {
                                println!("User: {}", email);
                            }
                            if let Some(name) = &status.user_name {
                                println!("Name: {}", name);
                            }
                            println!("Projects: {}", status.projects_count);
                            if let Some(tier) = &status.subscription_tier {
                                println!("Tier: {}", tier);
                            }
                        } else {
                            println!("Status: {}", "❌ Not authenticated".red());
                            println!("Run {} to get started", "orkee cloud login".yellow());
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to get status: {}", e);
                    }
                }
            }

            CloudCommands::List => {
                println!("📋 {}", "Orkee Cloud Projects".bold());

                if !cloud_client.is_authenticated() {
                    println!(
                        "❌ Not authenticated. Run {} first",
                        "orkee cloud login".yellow()
                    );
                    return Ok(());
                }

                match cloud_client.list_projects().await {
                    Ok(projects) => {
                        if projects.is_empty() {
                            println!("No projects found in the cloud.");
                        } else {
                            println!();
                            for project in projects {
                                println!("• {} ({})", project.name.cyan(), project.id);
                                if let Some(desc) = &project.description {
                                    println!("  {}", desc.dimmed());
                                }
                                if let Some(last_sync) = project.last_sync {
                                    println!(
                                        "  Last sync: {}",
                                        last_sync.format("%Y-%m-%d %H:%M:%S")
                                    );
                                }
                                println!();
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to list projects: {}", e);
                    }
                }
            }

            CloudCommands::Sync(args) => {
                println!("🔄 {}", "Orkee Cloud Sync".bold());

                if !cloud_client.is_authenticated() {
                    println!(
                        "❌ Not authenticated. Run {} first",
                        "orkee cloud login".yellow()
                    );
                    return Ok(());
                }

                // Initialize project manager
                let project_manager = ProjectsManager::new().await?;

                if let Some(project_id) = args.project {
                    // Sync specific project
                    match project_manager.get_project(&project_id).await {
                        Ok(Some(project)) => {
                            // Convert to cloud project with all enhanced fields
                            let cloud_project = ApiCloudProject {
                                id: project.id.clone(),
                                name: project.name.clone(),
                                path: project.project_root.clone(),
                                description: project.description.clone(),
                                setup_script: project.setup_script.clone(),
                                dev_script: project.dev_script.clone(),
                                cleanup_script: project.cleanup_script.clone(),
                                tags: project.tags.clone().unwrap_or_default(),
                                status: match project.status {
                                    orkee_projects::ProjectStatus::Active => "active".to_string(),
                                    orkee_projects::ProjectStatus::Archived => {
                                        "archived".to_string()
                                    }
                                },
                                priority: match project.priority {
                                    orkee_projects::Priority::High => "high".to_string(),
                                    orkee_projects::Priority::Medium => "medium".to_string(),
                                    orkee_projects::Priority::Low => "low".to_string(),
                                },
                                rank: project.rank,
                                task_source: project.task_source.as_ref().map(|ts| match ts {
                                    orkee_projects::TaskSource::Taskmaster => {
                                        "taskmaster".to_string()
                                    }
                                    orkee_projects::TaskSource::Manual => "manual".to_string(),
                                }),
                                mcp_servers: project.mcp_servers.clone().unwrap_or_default(),
                                git_repository: project.git_repository.as_ref().map(|gr| {
                                    orkee_cloud::api::GitRepositoryInfo {
                                        owner: gr.owner.clone(),
                                        repo: gr.repo.clone(),
                                        url: gr.url.clone(),
                                        branch: gr.branch.clone(),
                                    }
                                }),
                                manual_tasks: project.manual_tasks.as_ref().map(|tasks| {
                                    tasks
                                        .iter()
                                        .map(|t| serde_json::to_value(t).unwrap())
                                        .collect()
                                }),
                                created_at: project.created_at,
                                updated_at: project.updated_at,
                                last_sync: None,
                            };

                            // Serialize project for sync
                            let project_data = serde_json::to_value(&project)?;

                            match cloud_client.sync_project(cloud_project, project_data).await {
                                Ok(snapshot_id) => {
                                    println!("✅ Project '{}' synced successfully", project.name);
                                    println!("   Snapshot ID: {}", snapshot_id);
                                }
                                Err(e) => {
                                    println!("❌ Failed to sync project: {}", e);
                                }
                            }
                        }
                        Ok(None) => {
                            println!("❌ Project '{}' not found", project_id);
                        }
                        Err(e) => {
                            println!("❌ Failed to get project: {}", e);
                        }
                    }
                } else {
                    // Sync all projects
                    match project_manager.list_projects().await {
                        Ok(projects) => {
                            if projects.is_empty() {
                                println!("No local projects to sync.");
                            } else {
                                println!("Syncing {} projects...", projects.len());
                                let mut synced = 0;
                                let mut failed = 0;

                                for project in projects {
                                    let cloud_project = ApiCloudProject {
                                        id: project.id.clone(),
                                        name: project.name.clone(),
                                        path: project.project_root.clone(),
                                        description: project.description.clone(),
                                        setup_script: project.setup_script.clone(),
                                        dev_script: project.dev_script.clone(),
                                        cleanup_script: project.cleanup_script.clone(),
                                        tags: project.tags.clone().unwrap_or_default(),
                                        status: match project.status {
                                            orkee_projects::ProjectStatus::Active => {
                                                "active".to_string()
                                            }
                                            orkee_projects::ProjectStatus::Archived => {
                                                "archived".to_string()
                                            }
                                        },
                                        priority: match project.priority {
                                            orkee_projects::Priority::High => "high".to_string(),
                                            orkee_projects::Priority::Medium => {
                                                "medium".to_string()
                                            }
                                            orkee_projects::Priority::Low => "low".to_string(),
                                        },
                                        rank: project.rank,
                                        task_source: project.task_source.as_ref().map(
                                            |ts| match ts {
                                                orkee_projects::TaskSource::Taskmaster => {
                                                    "taskmaster".to_string()
                                                }
                                                orkee_projects::TaskSource::Manual => {
                                                    "manual".to_string()
                                                }
                                            },
                                        ),
                                        mcp_servers: project
                                            .mcp_servers
                                            .clone()
                                            .unwrap_or_default(),
                                        git_repository: project.git_repository.as_ref().map(|gr| {
                                            orkee_cloud::api::GitRepositoryInfo {
                                                owner: gr.owner.clone(),
                                                repo: gr.repo.clone(),
                                                url: gr.url.clone(),
                                                branch: gr.branch.clone(),
                                            }
                                        }),
                                        manual_tasks: project.manual_tasks.as_ref().map(|tasks| {
                                            tasks
                                                .iter()
                                                .map(|t| serde_json::to_value(t).unwrap())
                                                .collect()
                                        }),
                                        created_at: project.created_at,
                                        updated_at: project.updated_at,
                                        last_sync: None,
                                    };

                                    let project_data = serde_json::to_value(&project)?;

                                    match cloud_client
                                        .sync_project(cloud_project, project_data)
                                        .await
                                    {
                                        Ok(_) => {
                                            println!("  ✅ {}", project.name);
                                            synced += 1;
                                        }
                                        Err(e) => {
                                            println!("  ❌ {}: {}", project.name, e);
                                            failed += 1;
                                        }
                                    }
                                }

                                println!();
                                println!("Sync complete: {} succeeded, {} failed", synced, failed);
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to list projects: {}", e);
                        }
                    }
                }
            }

            CloudCommands::Restore(args) => {
                println!("📥 {}", "Orkee Cloud Restore".bold());

                if !cloud_client.is_authenticated() {
                    println!(
                        "❌ Not authenticated. Run {} first",
                        "orkee cloud login".yellow()
                    );
                    return Ok(());
                }

                if let Some(project_id) = args.project {
                    match cloud_client.restore_project(&project_id).await {
                        Ok(project_data) => {
                            // Convert back to project
                            match serde_json::from_value::<orkee_projects::types::Project>(
                                project_data,
                            ) {
                                Ok(project) => {
                                    let project_manager = ProjectsManager::new().await?;
                                    let project_name = project.name.clone();
                                    // Convert Project to ProjectCreateInput
                                    let project_input = orkee_projects::types::ProjectCreateInput {
                                        name: project.name,
                                        project_root: project.project_root,
                                        setup_script: project.setup_script,
                                        dev_script: project.dev_script,
                                        cleanup_script: project.cleanup_script,
                                        tags: project.tags,
                                        description: project.description,
                                        status: Some(project.status),
                                        rank: project.rank,
                                        priority: Some(project.priority),
                                        task_source: project.task_source,
                                        manual_tasks: project.manual_tasks,
                                        mcp_servers: project.mcp_servers,
                                    };
                                    match project_manager.create_project(project_input).await {
                                        Ok(_) => {
                                            println!(
                                                "✅ Project '{}' restored successfully",
                                                project_name
                                            );
                                        }
                                        Err(e) => {
                                            println!("❌ Failed to save restored project: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("❌ Failed to deserialize project data: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to restore project: {}", e);
                        }
                    }
                } else {
                    println!("❌ Please specify a project ID to restore");
                    println!(
                        "Run {} to see available projects",
                        "orkee cloud list".yellow()
                    );
                }
            }

            CloudCommands::Enable => {
                println!("🚀 {}", "Orkee Cloud".bold());
                println!();

                if !cloud_client.is_authenticated() {
                    println!("To enable Orkee Cloud, first authenticate:");
                    println!("  {}", "orkee cloud login".yellow());
                } else {
                    println!("✅ Orkee Cloud is already enabled and authenticated!");
                    if let Some((_, email, name)) = cloud_client.user_info() {
                        println!("   Logged in as: {} ({})", name, email);
                    }
                }

                println!();
                println!("Orkee Cloud features:");
                println!("  • {} project backups", "Automatic".cyan());
                println!("  • {} sync", "Multi-device".cyan());
                println!("  • {} collaboration", "Team".cyan());
                println!("  • {} access to your projects", "Web dashboard".cyan());
            }

            CloudCommands::Disable => {
                println!("🔒 {}", "Orkee Cloud".bold());
                println!();

                if cloud_client.is_authenticated() {
                    println!("To disable cloud features, logout:");
                    println!("  {}", "orkee cloud logout".yellow());
                } else {
                    println!("✅ Cloud features are already disabled");
                }

                println!();
                println!("Your local projects will continue to work normally.");
            }

            CloudCommands::Conflicts(args) => {
                println!("🔍 {}", "Checking for Sync Conflicts".bold());

                if !cloud_client.is_authenticated() {
                    println!(
                        "❌ Not authenticated. Run {} first",
                        "orkee cloud login".yellow()
                    );
                    return Ok(());
                }

                match cloud_client.check_conflicts(&args.project).await {
                    Ok(report) => {
                        if report.has_conflicts {
                            println!("⚠️  {} conflicts found:", report.conflicts.len());
                            for conflict in report.conflicts {
                                println!("\n  Field: {}", conflict.field.yellow());
                                println!("    Local:  {:?}", conflict.local_value);
                                println!("    Cloud:  {:?}", conflict.cloud_value);
                            }
                        } else {
                            println!("✅ No conflicts found");
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to check conflicts: {}", e);
                    }
                }
            }

            CloudCommands::Push(args) => {
                println!("⬆️  {}", "Push to Cloud".bold());

                if !cloud_client.is_authenticated() {
                    println!(
                        "❌ Not authenticated. Run {} first",
                        "orkee cloud login".yellow()
                    );
                    return Ok(());
                }

                let project_manager = ProjectsManager::new().await?;

                if let Some(project_id) = args.project {
                    match project_manager.get_project(&project_id).await {
                        Ok(Some(project)) => {
                            if args.incremental {
                                // TODO: Implement change detection for incremental sync
                                println!(
                                    "⚠️  Incremental sync not yet implemented, using full sync"
                                );
                            }

                            // Use the same sync logic as sync command
                            let cloud_project = ApiCloudProject {
                                id: project.id.clone(),
                                name: project.name.clone(),
                                path: project.project_root.clone(),
                                description: project.description.clone(),
                                setup_script: project.setup_script.clone(),
                                dev_script: project.dev_script.clone(),
                                cleanup_script: project.cleanup_script.clone(),
                                tags: project.tags.clone().unwrap_or_default(),
                                status: match project.status {
                                    orkee_projects::ProjectStatus::Active => "active".to_string(),
                                    orkee_projects::ProjectStatus::Archived => {
                                        "archived".to_string()
                                    }
                                },
                                priority: match project.priority {
                                    orkee_projects::Priority::High => "high".to_string(),
                                    orkee_projects::Priority::Medium => "medium".to_string(),
                                    orkee_projects::Priority::Low => "low".to_string(),
                                },
                                rank: project.rank,
                                task_source: project.task_source.as_ref().map(|ts| match ts {
                                    orkee_projects::TaskSource::Taskmaster => {
                                        "taskmaster".to_string()
                                    }
                                    orkee_projects::TaskSource::Manual => "manual".to_string(),
                                }),
                                mcp_servers: project.mcp_servers.clone().unwrap_or_default(),
                                git_repository: project.git_repository.as_ref().map(|gr| {
                                    orkee_cloud::api::GitRepositoryInfo {
                                        owner: gr.owner.clone(),
                                        repo: gr.repo.clone(),
                                        url: gr.url.clone(),
                                        branch: gr.branch.clone(),
                                    }
                                }),
                                manual_tasks: project.manual_tasks.as_ref().map(|tasks| {
                                    tasks
                                        .iter()
                                        .map(|t| serde_json::to_value(t).unwrap())
                                        .collect()
                                }),
                                created_at: project.created_at,
                                updated_at: project.updated_at,
                                last_sync: None,
                            };

                            let project_data = serde_json::to_value(&project)?;

                            match cloud_client.sync_project(cloud_project, project_data).await {
                                Ok(project_id) => {
                                    println!("✅ Project pushed successfully (ID: {})", project_id);
                                }
                                Err(e) => {
                                    println!("❌ Failed to push project: {}", e);
                                }
                            }
                        }
                        Ok(None) => {
                            println!("❌ Project '{}' not found", project_id);
                        }
                        Err(e) => {
                            println!("❌ Failed to get project: {}", e);
                        }
                    }
                } else {
                    println!("⚠️  Please specify a project ID with --project");
                }
            }
        }

        Ok(())
    }
}

/// Print cloud help information
#[allow(dead_code)]
pub fn print_help() {
    println!("{}", "Orkee Cloud Commands".bold().cyan());
    println!();
    println!(
        "  {} - Authenticate with Orkee Cloud",
        "orkee cloud login".yellow()
    );
    println!(
        "  {} - Sign out of Orkee Cloud",
        "orkee cloud logout".yellow()
    );
    println!(
        "  {} - Show authentication status",
        "orkee cloud status".yellow()
    );
    println!(
        "  {} - Enable cloud features",
        "orkee cloud enable".yellow()
    );
    println!(
        "  {} - Disable cloud features",
        "orkee cloud disable".yellow()
    );
    println!("  {} - Sync all projects", "orkee cloud sync".yellow());
    println!(
        "  {} - Sync specific project",
        "orkee cloud sync --project <id>".yellow()
    );
    println!(
        "  {} - Restore from cloud",
        "orkee cloud restore --project <id>".yellow()
    );
    println!("  {} - List cloud projects", "orkee cloud list".yellow());
    println!();
    println!("Cloud sync requires authentication and an active subscription.");
    println!("Free tier includes 2 projects and 100MB storage.");
}
