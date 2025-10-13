use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use std::process;

mod cli;

#[cfg(feature = "cloud")]
use cli::cloud::CloudCommands;
use cli::projects::ProjectsCommands;
use orkee_cli::dashboard::downloader::ensure_dashboard;
use orkee_cli::dashboard::DashboardMode;

/// Maximum number of parent directories to search when looking for monorepo root
const MAX_PARENT_SEARCH_DEPTH: usize = 5;

/// Number of retries when attempting to find an available port
const PORT_PICKER_RETRIES: usize = 5;

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
        #[arg(
            long,
            default_value = "0",
            help = "API server port (0 = auto-allocate)"
        )]
        api_port: u16,
        #[arg(
            long,
            default_value = "0",
            help = "Dashboard UI port (0 = auto-allocate)"
        )]
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
        #[arg(
            long,
            default_value = "0",
            help = "API server port (0 = auto-allocate)"
        )]
        api_port: u16,
        #[arg(
            long,
            default_value = "0",
            help = "Dashboard UI port (0 = auto-allocate)"
        )]
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
                if let Err(e) = save_port_info(final_api_port, final_ui_port) {
                    eprintln!("{} Failed to save port configuration: {}", "⚠️".yellow(), e);
                    eprintln!(
                        "{} Port discovery may not work. Consider setting ORKEE_API_PORT and ORKEE_UI_PORT environment variables.",
                        "ℹ️".cyan()
                    );
                }
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

    // Discover the actual API port being used
    let api_port = discover_api_port()?;

    // Get list of active servers
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/api/preview/servers", api_port))
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
                "http://localhost:{}/api/preview/servers/{}/stop",
                api_port, id
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
            if let Err(e) =
                start_server_with_options(api_port, cors_origin_clone, dashboard_path_for_server)
                    .await
            {
                eprintln!("{} Failed to start backend: {}", "Error:".red().bold(), e);
            }
        })
    };

    // Wait for backend to be ready with health check retry loop
    wait_for_backend_ready(api_port).await?;

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
        for _ in 0..MAX_PARENT_SEARCH_DEPTH {
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
            println!(
                "{}",
                "📦 Using single-port serving for production mode".cyan()
            );
            println!("{}", "✅ Dashboard and API started!".green());
            println!(
                "{} http://localhost:{}",
                "🌐 Access the dashboard at:".cyan(),
                api_port
            );
            println!(
                "{} Running in production mode (pre-built assets)",
                "⚡".cyan()
            );

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
                    println!(
                        "{} Running in development mode (with hot reload)",
                        "🔄".cyan()
                    );

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

    // Wait for ports to actually be freed rather than using a fixed delay
    wait_for_port_available(api_port).await?;
    wait_for_port_available(ui_port).await?;

    // Start fresh
    start_full_dashboard(api_port, ui_port, dev).await
}

async fn kill_port(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        let output = std::process::Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output();

        if let Ok(output) = output {
            if !output.stdout.is_empty() {
                let pid_string = String::from_utf8_lossy(&output.stdout);
                // Handle multiple PIDs (one per line)
                for pid_line in pid_string.lines() {
                    let pid_str = pid_line.trim();
                    // Validate PID is numeric before passing to shell command
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        let _ = std::process::Command::new("kill")
                            .args(["-9", &pid.to_string()])
                            .output();
                        println!("🔪 Killed process {} on port {}", pid, port);
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    {
        // Use netstat to find PIDs using the port
        let output = std::process::Command::new("netstat")
            .args(["-ano"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);

            // Parse netstat output to find PIDs for the port
            for line in output_str.lines() {
                // Look for lines containing the port (format: "  TCP    0.0.0.0:PORT    0.0.0.0:0    LISTENING    PID")
                if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
                    // Extract PID (last column)
                    if let Some(pid_str) = line.split_whitespace().last() {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            // Use taskkill to terminate the process
                            let kill_result = std::process::Command::new("taskkill")
                                .args(["/F", "/PID", &pid.to_string()])
                                .output();

                            match kill_result {
                                Ok(kill_output) if kill_output.status.success() => {
                                    println!("🔪 Killed process {} on port {}", pid, port);
                                }
                                Ok(kill_output) => {
                                    eprintln!(
                                        "Failed to kill process {}: {}",
                                        pid,
                                        String::from_utf8_lossy(&kill_output.stderr)
                                    );
                                }
                                Err(e) => {
                                    eprintln!("Failed to execute taskkill for PID {}: {}", pid, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn find_available_port(start: u16, end: u16) -> Option<u16> {
    for _ in 0..PORT_PICKER_RETRIES {
        // Try portpicker first for a random available port
        if let Some(port) = portpicker::pick_unused_port() {
            if port >= start && port <= end {
                return Some(port);
            }
        }
    }

    // Fallback: scan range for available port
    (start..=end).find(|&port| std::net::TcpListener::bind(("127.0.0.1", port)).is_ok())
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

fn discover_api_port() -> Result<u16, Box<dyn std::error::Error>> {
    // Priority 1: Check environment variable
    if let Ok(port_str) = std::env::var("ORKEE_API_PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            return Ok(port);
        }
    }

    // Priority 2: Read from saved port info file
    let home_dir_result = dirs::home_dir();
    if let Some(home_dir) = home_dir_result {
        let ports_file = home_dir.join(".orkee").join("ports.json");
        if let Ok(contents) = std::fs::read_to_string(&ports_file) {
            if let Ok(port_info) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(api_port) = port_info["api_port"].as_u64() {
                    return Ok(api_port as u16);
                }
            }
        }
    } else {
        eprintln!(
            "{} Could not determine home directory for port discovery",
            "⚠️".yellow()
        );
    }

    // Priority 3: Try to detect running server by checking common ports
    let common_ports = vec![4001, 4000, 4002, 3000, 8000, 8080, 9000];
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    for port in common_ports {
        // Try to connect to the port with a short timeout
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        if TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok() {
            eprintln!(
                "{} Found service running on port {} - assuming it's Orkee server",
                "🔍".cyan(),
                port
            );
            return Ok(port);
        }
    }

    // Priority 4: Fall back to default port with better error message
    Err(
        "Could not discover API port. Please ensure the Orkee server is running.\n\
        You can specify the port using:\n\
        - ORKEE_API_PORT environment variable\n\
        - Or start the server with: orkee dashboard --api-port <PORT>"
            .to_string()
            .into(),
    )
}

async fn wait_for_port_available(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let max_retries = 20;
    let retry_interval = tokio::time::Duration::from_millis(100);

    for _ in 0..max_retries {
        // Check if port is available by trying to bind to it
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        tokio::time::sleep(retry_interval).await;
    }

    Err(format!(
        "Port {} did not become available after {} seconds",
        port,
        (max_retries as f64) * retry_interval.as_secs_f64()
    )
    .into())
}

async fn wait_for_backend_ready(api_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let health_url = format!("http://localhost:{}/api/health", api_port);
    let max_retries = 30;
    let retry_interval = tokio::time::Duration::from_millis(500);

    print!("⏳ Waiting for backend to be ready");
    std::io::Write::flush(&mut std::io::stdout())?;

    for attempt in 1..=max_retries {
        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => {
                println!(" ✅");
                return Ok(());
            }
            _ => {
                // Backend not ready yet, wait and retry
                print!(".");
                std::io::Write::flush(&mut std::io::stdout())?;
                tokio::time::sleep(retry_interval).await;
            }
        }

        // On last attempt, return error
        if attempt == max_retries {
            println!(" ❌");
            return Err(format!(
                "Backend failed to become ready after {} seconds. Check logs for errors.",
                (max_retries as f64) * retry_interval.as_secs_f64()
            )
            .into());
        }
    }

    Ok(())
}
