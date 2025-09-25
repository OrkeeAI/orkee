use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod cli;
mod dashboard_downloader;

#[cfg(feature = "cloud")]
use cli::cloud::CloudCommands;
use cli::projects::ProjectsCommands;
use dashboard_downloader::ensure_dashboard;

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
        #[arg(long, default_value = "4001", help = "API server port")]
        api_port: u16,
        #[arg(long, default_value = "5173", help = "Dashboard UI port")]
        ui_port: u16,
        #[arg(long, help = "Restart services (kill existing processes first)")]
        restart: bool,
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
        #[arg(long, default_value = "4001", help = "API server port")]
        api_port: u16,
        #[arg(long, default_value = "5173", help = "Dashboard UI port")]
        ui_port: u16,
        #[arg(long, help = "Restart services (kill existing processes first)")]
        restart: bool,
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
        } => {
            // Check environment variables if not explicitly set via CLI
            let final_api_port = if api_port == 4001 {
                std::env::var("ORKEE_API_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(api_port)
            } else {
                api_port
            };

            let final_ui_port = if ui_port == 5173 {
                std::env::var("ORKEE_UI_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(ui_port)
            } else {
                ui_port
            };

            if restart {
                restart_dashboard(final_api_port, final_ui_port).await
            } else {
                start_full_dashboard(final_api_port, final_ui_port).await
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

async fn start_server(
    api_port: u16,
    cors_origin: String,
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

    // Call the original main function from main.rs
    orkee_cli::run_server().await
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
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        "🚀 Starting Orkee Dashboard (Backend + Frontend)..."
            .green()
            .bold()
    );

    // Auto-calculate CORS origin from UI port
    let cors_origin = format!("http://localhost:{}", ui_port);

    // Start backend server in background
    let backend_handle = {
        let cors_origin_clone = cors_origin.clone();
        tokio::spawn(async move {
            if let Err(e) = start_server(api_port, cors_origin_clone).await {
                eprintln!("{} Failed to start backend: {}", "Error:".red().bold(), e);
            }
        })
    };

    // Wait a moment for backend to start
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Check if we should use downloaded dashboard or dev server
    let dashboard_path = std::path::PathBuf::from("../dashboard");
    let package_json = dashboard_path.join("package.json");

    if package_json.exists() {
        // Development mode - use pnpm dev
        println!(
            "{}",
            "🖥️  Starting frontend dashboard (development mode)...".cyan()
        );
        let frontend_result = std::process::Command::new("pnpm")
            .args(["dev"])
            .current_dir("../dashboard")
            .env("ORKEE_UI_PORT", ui_port.to_string())
            .env("VITE_ORKEE_API_PORT", api_port.to_string())
            .spawn();

        match frontend_result {
            Ok(mut child) => {
                println!("{}", "✅ Both backend and frontend started!".green());
                println!("{} http://localhost:{}", "🔗 Backend API:".cyan(), api_port);
                println!("{} http://localhost:{}", "🌐 Frontend UI:".cyan(), ui_port);

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
                    "{} Make sure you're in the packages/cli directory and pnpm is installed",
                    "Tip:".yellow()
                );
            }
        }
    } else {
        // Production mode - download dashboard source and run dev server
        println!(
            "{}",
            "📦 Dashboard source not found, downloading...".yellow()
        );

        // Ensure dashboard is downloaded
        let dashboard_dir = ensure_dashboard().await?;

        // Run pnpm dev from the downloaded dashboard
        println!(
            "{}",
            "🖥️  Starting frontend dashboard from downloaded source...".cyan()
        );
        let frontend_result = std::process::Command::new("pnpm")
            .args(["dev"])
            .current_dir(&dashboard_dir)
            .env("ORKEE_UI_PORT", ui_port.to_string())
            .env("VITE_ORKEE_API_PORT", api_port.to_string())
            .spawn();

        match frontend_result {
            Ok(mut child) => {
                println!("{}", "✅ Both backend and frontend started!".green());
                println!("{} http://localhost:{}", "🔗 Backend API:".cyan(), api_port);
                println!("{} http://localhost:{}", "🌐 Frontend UI:".cyan(), ui_port);

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
                    "{} Make sure pnpm is installed and dependencies are installed in {}",
                    "Tip:".yellow(),
                    dashboard_dir.display()
                );
            }
        }
    }

    Ok(())
}

async fn restart_dashboard(api_port: u16, ui_port: u16) -> Result<(), Box<dyn std::error::Error>> {
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
    start_full_dashboard(api_port, ui_port).await
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
