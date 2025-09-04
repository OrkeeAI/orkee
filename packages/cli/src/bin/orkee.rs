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
        Commands::Tui { refresh_interval, theme: _ } => {
            start_tui(refresh_interval).await
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
        execute!(
            terminal.backend_mut(),
            terminal::LeaveAlternateScreen
        )?;
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

