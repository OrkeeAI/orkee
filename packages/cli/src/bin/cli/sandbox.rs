// ABOUTME: CLI commands for sandbox image management (build, push, list, configure)
// ABOUTME: Wraps Docker CLI commands for building and pushing custom sandbox images

use anyhow::{Context, Result};
use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand)]
pub enum SandboxCommands {
    /// Build a custom sandbox Docker image
    Build {
        /// Name of the image (default: orkee-sandbox)
        #[arg(long)]
        name: Option<String>,

        /// Tag for the image (default: latest)
        #[arg(long)]
        tag: Option<String>,

        /// Path to Dockerfile (default: Dockerfile)
        #[arg(long)]
        dockerfile: Option<String>,
    },

    /// Push sandbox image to Docker Hub
    Push {
        /// Full image name (e.g., username/orkee-sandbox:v1.0)
        image: String,
    },

    /// List all Orkee sandbox images
    Images,

    /// Manage sandbox configuration
    #[command(subcommand)]
    Config(SandboxConfigCommands),
}

#[derive(Subcommand)]
pub enum SandboxConfigCommands {
    /// Show current sandbox configuration
    Show,

    /// Set default sandbox image
    SetImage {
        /// Image name (e.g., username/orkee-sandbox:v1.0)
        image: String,
    },
}

impl SandboxCommands {
    pub async fn execute(&self) {
        match self {
            SandboxCommands::Build {
                name,
                tag,
                dockerfile,
            } => {
                if let Err(e) = build_command(name.clone(), tag.clone(), dockerfile.clone()).await
                {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            SandboxCommands::Push { image } => {
                if let Err(e) = push_command(image.clone()).await {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            SandboxCommands::Images => {
                if let Err(e) = images_command().await {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            SandboxCommands::Config(config_cmd) => match config_cmd {
                SandboxConfigCommands::Show => {
                    if let Err(e) = config_show_command().await {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
                SandboxConfigCommands::SetImage { image } => {
                    if let Err(e) = config_set_image_command(image.clone()).await {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            },
        }
    }
}

pub async fn build_command(
    name: Option<String>,
    tag: Option<String>,
    dockerfile: Option<String>,
) -> Result<()> {
    use inquire::Text;
    use orkee_projects::DbState;

    // Initialize database
    let db = DbState::init().await?;
    let mut settings = db.sandbox_settings.get_sandbox_settings().await?;

    // Try to get Docker username from settings, docker info, or prompt
    let docker_username = if let Some(username) = &settings.docker_username {
        username.clone()
    } else {
        match get_docker_username() {
            Ok(username) => {
                // Save to settings for future use
                settings.docker_username = Some(username.clone());
                db.sandbox_settings
                    .update_sandbox_settings(&settings, Some("cli"))
                    .await?;
                println!("✅ Saved Docker Hub username to settings");
                username
            }
            Err(_) => {
                println!("⚠️  Could not detect Docker Hub username.");
                println!("   Please enter your Docker Hub username to tag the image:");
                let username = Text::new("Docker Hub username:")
                    .prompt()
                    .context("Failed to read username input")?;

                // Save to settings for future use
                settings.docker_username = Some(username.clone());
                db.sandbox_settings
                    .update_sandbox_settings(&settings, Some("cli"))
                    .await?;
                println!("✅ Saved Docker Hub username to settings");

                username
            }
        }
    };

    let image_name = name.unwrap_or_else(|| "orkee-sandbox".to_string());
    let image_tag = tag.unwrap_or_else(|| "latest".to_string());
    let full_image = format!("{}/{}:{}", docker_username, image_name, image_tag);

    // Determine build context and Dockerfile path
    // If using default, set both to packages/sandbox/docker/
    let (dockerfile_path, build_context) = match dockerfile {
        Some(path) => (path, ".".to_string()),
        None => (
            "packages/sandbox/docker/Dockerfile".to_string(),
            "packages/sandbox/docker".to_string(),
        ),
    };

    println!("🐋 Building sandbox image: {}", full_image);
    println!("📄 Using Dockerfile: {}", dockerfile_path);

    // Build the Docker image
    let status = Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg(&full_image)
        .arg("-f")
        .arg(&dockerfile_path)
        .arg("--label")
        .arg("orkee.sandbox=true")
        .arg(build_context)
        .status()
        .context("Failed to execute docker build command")?;

    if !status.success() {
        anyhow::bail!("Docker build failed with exit code: {:?}", status.code());
    }

    println!("✅ Successfully built image: {}", full_image);
    println!("\nNext steps:");
    println!("  1. Push to Docker Hub: orkee sandbox push {}", full_image);
    println!(
        "  2. Set as default: orkee sandbox config set-image {}",
        full_image
    );

    Ok(())
}

pub async fn push_command(image: String) -> Result<()> {
    // Check if user is logged in
    if !is_docker_logged_in()? {
        println!("⚠️  You are not logged in to Docker Hub.");
        println!("   Please run: orkee auth login docker");
        anyhow::bail!("Docker authentication required");
    }

    println!("🐋 Pushing image to Docker Hub: {}", image);

    // Push the Docker image
    let status = Command::new("docker")
        .arg("push")
        .arg(&image)
        .status()
        .context("Failed to execute docker push command")?;

    if !status.success() {
        anyhow::bail!("Docker push failed with exit code: {:?}", status.code());
    }

    println!("✅ Successfully pushed image: {}", image);
    println!("\nYour image is now available on Docker Hub!");
    println!("Set it as default: orkee sandbox config set-image {}", image);

    Ok(())
}

pub async fn images_command() -> Result<()> {
    println!("🐋 Orkee Sandbox Images\n");

    // List Docker images with orkee.sandbox label
    let output = Command::new("docker")
        .arg("images")
        .arg("--filter")
        .arg("label=orkee.sandbox=true")
        .arg("--format")
        .arg("table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}")
        .output()
        .context("Failed to execute docker images command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Docker images failed with exit code: {:?}",
            output.status.code()
        );
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    if output_str.trim().is_empty() {
        println!("No Orkee sandbox images found.");
        println!("\nBuild your first sandbox image:");
        println!("  orkee sandbox build --name my-sandbox --tag v1.0");
    } else {
        println!("{}", output_str);
    }

    Ok(())
}

pub async fn config_show_command() -> Result<()> {
    use orkee_projects::DbState;

    let db = DbState::init().await?;
    let settings = db.sandbox_settings.get_sandbox_settings().await?;

    println!("📦 Sandbox Configuration\n");
    println!("Enabled:         {}", settings.enabled);
    println!("Provider:        {}", settings.default_provider);
    println!("Image:           {}", settings.default_image);
    println!("Docker Username: {}", settings.docker_username.as_deref().unwrap_or("(not set)"));
    println!("Max Concurrent:  {} local, {} cloud",
        settings.max_concurrent_local,
        settings.max_concurrent_cloud);
    println!("Max Resources:   {} CPU, {} GB RAM, {} GB Disk",
        settings.max_cpu_cores_per_sandbox,
        settings.max_memory_gb_per_sandbox,
        settings.max_disk_gb_per_sandbox);
    println!("Auto-stop idle:  {} minutes", settings.auto_stop_idle_minutes);
    println!("Max runtime:     {} hours", settings.max_runtime_hours);

    Ok(())
}

pub async fn config_set_image_command(image: String) -> Result<()> {
    use orkee_projects::DbState;

    let db = DbState::init().await?;

    // Get current settings
    let mut settings = db.sandbox_settings.get_sandbox_settings().await?;

    // Update image (it's a String, not Option<String>)
    settings.default_image = image.clone();

    // Save
    db.sandbox_settings
        .update_sandbox_settings(&settings, Some("cli"))
        .await?;

    println!("✅ Default sandbox image set to: {}", image);

    Ok(())
}

// Helper function to check if user is logged in to Docker
fn is_docker_logged_in() -> Result<bool> {
    let output = Command::new("docker")
        .arg("info")
        .arg("--format")
        .arg("{{.Username}}")
        .output()
        .context("Failed to execute docker info command")?;

    if !output.status.success() {
        return Ok(false);
    }

    let username = String::from_utf8_lossy(&output.stdout);
    Ok(!username.trim().is_empty())
}

// Helper function to get Docker Hub username
// Returns Ok(username) if detected, Err if not available
fn get_docker_username() -> Result<String> {
    // Try to read from Docker config file first
    if let Ok(home) = std::env::var("HOME") {
        let config_path = format!("{}/.docker/config.json", home);
        if let Ok(_content) = std::fs::read_to_string(config_path) {
            // Try to parse and extract username from config
            // This is a simple approach - Docker config doesn't always have the username
            // so we'll fall back to prompting
            // TODO: Parse JSON config if needed
        }
    }

    // Try docker info command (may not work on all Docker versions)
    let output = Command::new("docker")
        .arg("info")
        .output();

    if let Ok(output) = output {
        let info_str = String::from_utf8_lossy(&output.stdout);
        // Try to find username in output
        for line in info_str.lines() {
            if line.trim().starts_with("Username:") {
                if let Some(username) = line.split(':').nth(1) {
                    let username = username.trim();
                    if !username.is_empty() {
                        return Ok(username.to_string());
                    }
                }
            }
        }
    }

    // Could not detect username
    anyhow::bail!("Could not detect Docker Hub username")
}
