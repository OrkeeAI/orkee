use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod cli;

use cli::projects::ProjectsCommands;

#[derive(Parser)]
#[command(name = "orkee")]
#[command(about = "Orkee CLI - AI agent orchestration platform")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the dashboard server
    Dashboard {
        #[arg(short, long, default_value = "4001")]
        port: u16,
        #[arg(long, default_value = "http://localhost:5173")]
        cors_origin: String,
    },
    /// Launch the terminal user interface
    Tui {
        /// Server URL
        #[arg(long, default_value = "http://localhost:4001")]
        server_url: String,
        
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
        Ok(_) => {},
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            process::exit(1);
        }
    }
}

async fn handle_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Dashboard { port, cors_origin } => {
            start_server(port, cors_origin).await
        }
        Commands::Tui { server_url, refresh_interval, theme: _ } => {
            start_tui(server_url, refresh_interval).await
        }
        Commands::Projects(projects_cmd) => {
            cli::projects::handle_projects_command(projects_cmd).await
        }
    }
}

async fn start_server(port: u16, cors_origin: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🚀 Starting Orkee CLI server...".green().bold());
    println!("{} http://localhost:{}", "📡 Server will run on".cyan(), port);
    println!("{} {}", "🔗 CORS origin:".cyan(), cors_origin);

    // Set environment variables for the server
    std::env::set_var("PORT", port.to_string());
    std::env::set_var("CORS_ORIGIN", cors_origin);

    // Call the original main function from main.rs
    orkee_cli::run_server().await
}

async fn start_tui(server_url: String, refresh_interval: u64) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{execute, terminal};
    
    println!("{}", "🎮 Starting Orkee TUI...".green().bold());
    println!("{} {}", "🔗 Server URL:".cyan(), server_url);
    println!("{} {}s", "⏱️ Refresh interval:".cyan(), refresh_interval);
    
    // Initialize TUI application
    let mut app = orkee_tui::App::new(server_url, refresh_interval);
    
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;
    
    // Run the application
    let result = run_tui_app(&mut terminal, &mut app).await;
    
    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen
    )?;
    
    if let Err(e) = result {
        eprintln!("TUI application error: {}", e);
    }
    
    Ok(())
}

async fn run_tui_app<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>, 
    app: &mut orkee_tui::App
) -> Result<(), Box<dyn std::error::Error>> {
    use orkee_tui::events::{EventHandler, AppEvent};
    use crossterm::event::KeyCode;
    
    let mut event_handler = EventHandler::new(250); // 250ms tick rate
    
    // Initial health check and data loading
    match app.api_client.health_check().await {
        Ok(true) => {
            app.state.set_connection_status(orkee_tui::state::ConnectionStatus::Connected);
            // Load initial projects data
            if let Ok(projects_response) = app.api_client.get_projects().await {
                if let Some(data) = projects_response.get("data") {
                    if let Some(projects_array) = data.as_array() {
                        app.state.set_projects(projects_array.clone());
                    }
                }
            }
        }
        _ => {
            app.state.set_connection_status(orkee_tui::state::ConnectionStatus::Disconnected);
        }
    }
    
    loop {
        // Render the UI
        terminal.draw(|frame| {
            orkee_tui::ui::render(frame, &app.state);
        })?;
        
        // Handle events
        if let Some(event) = event_handler.next().await {
            match event {
                AppEvent::Key(key) => {
                    match key.code {
                        KeyCode::Char('q') => {
                            app.quit();
                            break;
                        }
                        KeyCode::Char('d') => {
                            app.state.current_screen = orkee_tui::state::Screen::Dashboard;
                        }
                        KeyCode::Char('p') => {
                            app.state.current_screen = orkee_tui::state::Screen::Projects;
                        }
                        KeyCode::Char('s') => {
                            app.state.current_screen = orkee_tui::state::Screen::Settings;
                        }
                        KeyCode::Tab => {
                            app.state.next_screen();
                        }
                        _ => {}
                    }
                }
                AppEvent::Tick => {
                    // Periodic tasks (could be used for auto-refresh)
                }
                AppEvent::Refresh => {
                    // Refresh data from API
                    match app.api_client.health_check().await {
                        Ok(true) => {
                            app.state.set_connection_status(orkee_tui::state::ConnectionStatus::Connected);
                            if let Ok(projects_response) = app.api_client.get_projects().await {
                                if let Some(data) = projects_response.get("data") {
                                    if let Some(projects_array) = data.as_array() {
                                        app.state.set_projects(projects_array.clone());
                                    }
                                }
                            }
                        }
                        _ => {
                            app.state.set_connection_status(orkee_tui::state::ConnectionStatus::Disconnected);
                        }
                    }
                }
                AppEvent::Quit => {
                    app.quit();
                    break;
                }
            }
        }
        
        if app.should_quit {
            break;
        }
    }
    
    Ok(())
}