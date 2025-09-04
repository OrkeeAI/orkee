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
    /// Start the HTTP server
    Server {
        #[arg(short, long, default_value = "4001")]
        port: u16,
        #[arg(long, default_value = "http://localhost:5173")]
        cors_origin: String,
    },
    /// Manage projects
    #[command(subcommand)]
    Projects(ProjectsCommands),
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
        Commands::Server { port, cors_origin } => {
            start_server(port, cors_origin).await
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