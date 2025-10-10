use clap::{Parser, Subcommand};
use colored::*;
use std::process;
use std::path::PathBuf;

mod cli;

#[cfg(feature = "cloud")]
use cli::cloud::CloudCommands;
use cli::projects::ProjectsCommands;
use orkee_cli::dashboard::downloader::ensure_dashboard;
use orkee_cli::dashboard::DashboardMode;

#[derive(Subcommand)]
enum PreviewCommands {
    /// Stop all running preview servers
    StopAll,
}

#[derive(Parser)]
#[command(name = "orkee")]
#[command(about = "Orkee CLI - AI agent orchestration platform")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "cloud")]
#[derive(Subcommand)]
enum Commands {
    /// Start the dashboard (backend + frontend)
    Dashboard {
        #[arg(long, default_value = "0", help = "API server port (0 = auto-allocate)")]
        api_port: u16,
        #[arg(long, default_value = "0", help = "Dashboard UI port (0 = auto-allocate)")]
        ui_port: u16,
        #[arg(long, help = "Restart services (kill existing processes first)")]
        restart: bool,
        #[arg(long, help = "Use local development dashboard from packages/dashboard")]
        dev: bool,
    },
    /// Launch the terminal user interface
    Tui {
        /// Refresh interval in seconds
        #[arg(long, default_value = "20")]
        refresh_interval: u64,

        /// Theme
        #[arg(long, value_enum, default_value = "dark")]
        theme: TuiTheme,
    },
    /// Manage projects
    #[command(subcommand)]
    Projects(ProjectsCommands),
    /// Manage cloud sync
    #[command(subcommand)]
    Cloud(CloudCommands),
    /// Manage preview servers
    #[command(subcommand)]
    Preview(PreviewCommands),
}

#[cfg(not(feature = "cloud"))]
#[derive(Subcommand)]
enum Commands {
    /// Start the dashboard (backend + frontend)
    Dashboard {
        #[arg(long, default_value = "0", help = "API server port (0 = auto-allocate)")]
        api_port: u16,
        #[arg(long, default_value = "0", help = "Dashboard UI port (0 = auto-allocate)")]
        ui_port: u16,
        #[arg(long, help = "Restart services (kill existing processes first)")]
        restart: bool,
        #[arg(long, help = "Use local development dashboard from packages/dashboard")]
        dev: bool,
    },
    /// Launch the terminal user interface
    Tui {
        /// Refresh interval in seconds
        #[arg(long, default_value = "20")]
        refresh_interval: u64,

        /// Theme
        #[arg(long, value_enum, default_value = "dark")]
        theme: TuiTheme,
    },
    /// Manage projects
    #[command(subcommand)]
    Projects(ProjectsCommands),
    /// Manage preview servers
    #[command(subcommand)]
    Preview(PreviewCommands),
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum TuiTheme {
    Light,
    Dark,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match handle_command(cli.command).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            process::exit(1);
        }
    }
}

async fn handle_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Dashboard {
            api_port,
            ui_port,
            restart,
            dev,
        } => {
            // Determine ports: specified > environment > dynamic
            let final_api_port = if api_port != 0 {
                // User specified a port
                api_port
            } else if let Ok(port) = std::env::var("ORKEE_API_PORT")
                .and_then(|p| p.parse::<u16>().map_err(|_| std::env::VarError::NotPresent))
            {
                // Environment variable set
                port
            } else {
                // Dynamic allocation
                find_available_port(4001, 4100).unwrap_or(4001)
            };

            let final_ui_port = if ui_port != 0 {
                // User specified a port
                ui_port
            } else if let Ok(port) = std::env::var("ORKEE_UI_PORT")
                .and_then(|p| p.parse::<u16>().map_err(|_| std::env::VarError::NotPresent))
            {
                // Environment variable set
                port
            } else {
                // Dynamic allocation
                find_available_port(5173, 5273).unwrap_or(5173)
            };

            // Save port info for discovery when using dynamic ports
            if api_port == 0 || ui_port == 0 {
                save_port_info(final_api_port, final_ui_port)?;
            }

            if restart {
                restart_dashboard(final_api_port, final_ui_port, dev).await
            } else {
                start_full_dashboard(final_api_port, final_ui_port, dev).await
            }
        }
        Commands::Tui {
            refresh_interval,
            theme: _,
        } => start_tui(refresh_interval).await,
        Commands::Projects(projects_cmd) => {
            cli::projects::handle_projects_command(projects_cmd).await
        }
        #[cfg(feature = "cloud")]
        Commands::Cloud(cloud_cmd) => cli::cloud::handle_cloud_command(cloud_cmd)
            .await
            .map_err(|e| e.to_string().into()),
        Commands::Preview(preview_cmd) => handle_preview_command(preview_cmd).await,
    }
}

async fn handle_preview_command(
    command: PreviewCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        PreviewCommands::StopAll => stop_all_preview_servers().await,
    }
}

async fn stop_all_preview_servers() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🛑 Stopping all preview servers...".yellow().bold());

    // Get list of active servers
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:4001/api/preview/servers")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(
            "Failed to connect to Orkee server. Make sure the dashboard is running.".into(),
        );
    }

    let server_list: serde_json::Value = response.json().await?;
    let project_ids = server_list["data"]
        .as_array()
        .ok_or("Invalid response format")?;

    if project_ids.is_empty() {
        println!("{}", "✅ No preview servers are currently running".green());
        return Ok(());
    }

    println!(
        "{} Found {} running server(s)",
        "📋".cyan(),
        project_ids.len()
    );

    let mut stopped = 0;
    let mut failed = 0;

    for project_id in project_ids {
        let id = project_id.as_str().unwrap_or("unknown");
        print!("   Stopping server for project {}... ", id);

        let stop_response = client
            .post(format!(
                "http://localhost:4001/api/preview/servers/{}/stop",
                id
            ))
            .send()
            .await;

        match stop_response {
            Ok(resp) if resp.status().is_success() => {
                println!("{}", "✅".green());
                stopped += 1;
            }
            _ => {
                println!("{}", "❌".red());
                failed += 1;
            }
        }
    }

    if failed == 0 {
        println!(
            "{} Successfully stopped all {} preview servers!",
            "🎉".green(),
            stopped
        );
    } else {
        println!(
            "{} Stopped {} servers, {} failed",
            "⚠️".yellow(),
            stopped,
            failed
        );
    }

    Ok(())
}

async fn start_server_with_options(
    api_port: u16,
    cors_origin: String,
    dashboard_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🚀 Starting Orkee CLI server...".green().bold());
    println!(
        "{} http://localhost:{}",
        "📡 Server will run on".cyan(),
        api_port
    );
    println!("{} {}", "🔗 CORS origin:".cyan(), cors_origin);

    // Set environment variables for the server
    std::env::set_var("ORKEE_API_PORT", api_port.to_string());
    std::env::set_var("PORT", api_port.to_string()); // Backwards compatibility
    std::env::set_var("CORS_ORIGIN", cors_origin);

    // Call the server with optional dashboard path
    orkee_cli::run_server_with_options(dashboard_path).await
}

async fn start_tui(refresh_interval: u64) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{execute, terminal};

    println!("{}", "🎮 Starting Orkee TUI...".green().bold());
    println!("{} {}s", "⏱️ Refresh interval:".cyan(), refresh_interval);

    // Initialize TUI application
    let mut app = orkee_tui::App::new(refresh_interval);

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Run the application with proper cleanup
    let result = app.run(&mut terminal).await;

    // Always restore terminal, even if there was an error
    let cleanup_result = (|| -> Result<(), Box<dyn std::error::Error>> {
        terminal::disable_raw_mode()?;
        execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
        Ok(())
    })();

    // Report any cleanup errors
    if let Err(cleanup_error) = cleanup_result {
        eprintln!("Terminal cleanup error: {}", cleanup_error);
    }

    // Report application errors
    if let Err(e) = result {
        eprintln!("TUI application error: {}", e);
    }

    // Force process exit to ensure we don't hang
    std::process::exit(0);
}

async fn start_full_dashboard(
    api_port: u16,
    ui_port: u16,
    dev: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        "🚀 Starting Orkee Dashboard (Backend + Frontend)..."
            .green()
            .bold()
    );

    // Determine which dashboard to use first (moved up)
    let (dashboard_dir, dashboard_mode) = if dev || std::env::var("ORKEE_DEV_MODE").is_ok() {
        // Try to use local development dashboard
        // First check if ORKEE_DASHBOARD_PATH is set (explicit override)
        if let Ok(dashboard_path) = std::env::var("ORKEE_DASHBOARD_PATH") {
            let path = std::path::PathBuf::from(dashboard_path);
            if path.exists() && path.join("package.json").exists() {
                println!(
                    "{} Using dashboard from ORKEE_DASHBOARD_PATH: {}",
                    "🔧".cyan(),
                    path.display()
                );
                (path, DashboardMode::Source) // Local dev is always Source mode
            } else {
                println!(
                    "{} ORKEE_DASHBOARD_PATH invalid, falling back to auto-detection",
                    "⚠️".yellow()
                );
                let path = find_local_dashboard().await?;
                (path, DashboardMode::Source)
            }
        } else {
            let path = find_local_dashboard().await?;
            (path, DashboardMode::Source)
        }
    } else {
        // Use downloaded dashboard from ~/.orkee/dashboard
        let (path, mode) = ensure_dashboard(dev).await?;
        println!(
            "{} Using {} mode",
            "📦".cyan(),
            match mode {
                DashboardMode::Dist => "pre-built dashboard",
                DashboardMode::Source => "source dashboard with dev server",
            }
        );
        (path, mode)
    };

    // Auto-calculate CORS origin from UI port (for dev mode) or same origin (for dist mode)
    let cors_origin = if matches!(dashboard_mode, DashboardMode::Source) {
        format!("http://localhost:{}", ui_port)
    } else {
        // For dist mode, no CORS needed as it's same origin
        format!("http://localhost:{}", api_port)
    };

    // Start backend server in background
    let backend_handle = {
        let cors_origin_clone = cors_origin.clone();
        let dashboard_path_for_server = if matches!(dashboard_mode, DashboardMode::Dist) {
            Some(dashboard_dir.clone())
        } else {
            None
        };

        tokio::spawn(async move {
            if let Err(e) = start_server_with_options(api_port, cors_origin_clone, dashboard_path_for_server).await {
                eprintln!("{} Failed to start backend: {}", "Error:".red().bold(), e);
            }
        })
    };

    // Wait a moment for backend to start
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

async fn find_local_dashboard() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // Try to find dashboard by walking up directories (for monorepo)
    let mut current = std::env::current_dir()?;

    // Try current/packages/dashboard first
    let try_path = current.join("packages/dashboard");
    if try_path.exists() && try_path.join("package.json").exists() {
        println!(
            "{} Using local development dashboard from {}",
            "🔧".cyan(),
            try_path.display()
        );
        return Ok(try_path);
    }

    // Walk up parent directories to find monorepo root
    for _ in 0..5 {  // Increased from 3 to 5 levels
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
            let try_path = current.join("packages/dashboard");
            if try_path.exists() && try_path.join("package.json").exists() {
                println!(
                    "{} Using local development dashboard from {}",
                    "🔧".cyan(),
                    try_path.display()
                );
                return Ok(try_path);
            }
        } else {
            break;
        }
    }

    // Fallback to downloaded dashboard
    println!(
        "{} Local dashboard not found, falling back to downloaded version",
        "⚠️".yellow()
    );
    let (path, _mode) = ensure_dashboard(true).await?; // Use dev mode for source when local search fails
    Ok(path)
}

    // Start frontend based on mode
    match dashboard_mode {
        DashboardMode::Dist => {
            // For dist mode, serve everything from the API server (single port)
            println!("{}", "📦 Using single-port serving for production mode".cyan());
            println!("{}", "✅ Dashboard and API started!".green());
            println!("{} http://localhost:{}", "🌐 Access the dashboard at:".cyan(), api_port);
            println!("{} Running in production mode (pre-built assets)", "⚡".cyan());

            // Just wait for the backend since it's serving everything
            let _ = backend_handle.await;
        }
        DashboardMode::Source => {
            // Run dev server for source mode
            let frontend_result = std::process::Command::new("bun")
                .args(["run", "dev"])
                .current_dir(&dashboard_dir)
                .env("ORKEE_UI_PORT", ui_port.to_string())
                .env("ORKEE_API_PORT", api_port.to_string())
                .env("VITE_ORKEE_API_PORT", api_port.to_string())
                .spawn();

            match frontend_result {
                Ok(mut child) => {
                    println!("{}", "✅ Both backend and frontend started!".green());
                    println!("{} http://localhost:{}", "🔗 Backend API:".cyan(), api_port);
                    println!("{} http://localhost:{}", "🌐 Frontend UI:".cyan(), ui_port);
                    println!("{} Running in development mode (with hot reload)", "🔄".cyan());

                    // Wait for both processes
                    let _ = tokio::join!(
                        backend_handle,
                        tokio::task::spawn_blocking(move || {
                            let _ = child.wait();
                        })
                    );
                }
                Err(e) => {
                    eprintln!("{} Failed to start frontend: {}", "Error:".red().bold(), e);
                    eprintln!(
                        "{} Make sure bun is installed and dependencies are installed in {}",
                        "Tip:".yellow(),
                        dashboard_dir.display()
                    );
                }
            }
        }
    }

    Ok(())
}

async fn restart_dashboard(
    api_port: u16,
    ui_port: u16,
    dev: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        "🔄 Restarting all dashboard services...".yellow().bold()
    );

    // Kill existing processes on the ports
    kill_port(api_port).await?;
    kill_port(ui_port).await?;
    // Also kill common dev server ports if different
    if ui_port != 5173 {
        kill_port(5173).await?;
    }
    if ui_port != 5174 {
        kill_port(5174).await?;
    }

    println!("{}", "💀 Killed existing services".yellow());

    // Wait longer for ports to be freed and processes to fully terminate
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Start fresh
    start_full_dashboard(api_port, ui_port, dev).await
}

async fn kill_port(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("lsof")
        .args(["-ti", &format!(":{}", port)])
        .output();

    if let Ok(output) = output {
        if !output.stdout.is_empty() {
            let pid_string = String::from_utf8_lossy(&output.stdout);
            // Handle multiple PIDs (one per line)
            for pid_line in pid_string.lines() {
                let pid = pid_line.trim();
                if !pid.is_empty() {
                    let _ = std::process::Command::new("kill")
                        .args(["-9", pid])
                        .output();
                    println!("🔪 Killed process {} on port {}", pid, port);
                }
            }
        }
    }

    Ok(())
}

fn find_available_port(start: u16, end: u16) -> Option<u16> {
    for _ in 0..5 {
        // Try portpicker first for a random available port
        if let Some(port) = portpicker::pick_unused_port() {
            if port >= start && port <= end {
                return Some(port);
            }
        }
    }

    // Fallback: scan range for available port
    for port in start..=end {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Some(port);
        }
    }
    None
}

fn save_port_info(api_port: u16, ui_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // Create ~/.orkee directory if it doesn't exist
    let orkee_dir = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".orkee");
    fs::create_dir_all(&orkee_dir)?;

    // Write port info to JSON file
    let ports_file = orkee_dir.join("ports.json");
    let port_info = serde_json::json!({
        "api_port": api_port,
        "ui_port": ui_port,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    fs::write(&ports_file, serde_json::to_string_pretty(&port_info)?)?;
    println!("💾 Saved port configuration to {}", ports_file.display());

    Ok(())
}
