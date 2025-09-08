#!/usr/bin/env -S cargo run --bin orkee-mcp --

use anyhow::Result;
use clap::Parser;
use serde_json::{json, Value};
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::io::{self, BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

mod mcp;
mod tools;

use mcp::*;
use tools::{tools_call, tools_list};

#[derive(Parser)]
#[command(name = "orkee-mcp")]
#[command(about = "Orkee MCP Server - Model Context Protocol server for project management")]
#[command(version)]
struct Cli {
    #[arg(long, help = "Enable MCP server mode (default behavior)")]
    mcp: bool,
    #[arg(long, help = "Display available tools")]
    tools: bool,
    #[arg(long, help = "Display available resources")]
    resources: bool,
    #[arg(long, help = "Display available prompts")]
    prompts: bool,
}

async fn handle_rpc_request(method: &str, params: Option<Value>) -> Result<Value> {
    match method {
        "initialize" => {
            let request = if let Some(p) = params {
                Some(serde_json::from_value(p)?)
            } else {
                None
            };
            let result = initialize(request).await?;
            Ok(serde_json::to_value(result)?)
        }
        "ping" => {
            let result = ping(params).await?;
            Ok(result)
        }
        "logging/setLevel" => {
            let request = if let Some(p) = params {
                Some(serde_json::from_value(p)?)
            } else {
                None
            };
            let result = logging_set_level(request).await?;
            Ok(result)
        }
        "resources/list" => {
            let result = resources_list(params).await?;
            Ok(result)
        }
        "resources/read" => {
            let result = resources_read(params).await?;
            Ok(result)
        }
        "prompts/list" => {
            let result = prompts_list(params).await?;
            Ok(result)
        }
        "prompts/get" => {
            let result = prompts_get(params).await?;
            Ok(result)
        }
        "tools/list" => {
            let request = if let Some(p) = params {
                Some(serde_json::from_value(p)?)
            } else {
                None
            };
            let result = tools_list(request).await?;
            Ok(serde_json::to_value(result)?)
        }
        "tools/call" => {
            let request = if let Some(p) = params {
                Some(serde_json::from_value(p)?)
            } else {
                None
            };
            let result = tools_call(request).await?;
            Ok(serde_json::to_value(result)?)
        }
        _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.tools {
        println!("Available tools:");
        println!("- projects: List, get, or search Orkee projects");
        println!("- project_manage: Create, update, or delete Orkee projects");
        return Ok(());
    }

    if cli.resources {
        println!("Available resources:");
        println!("- project://[id]: Access to specific project data");
        return Ok(());
    }

    if cli.prompts {
        println!("Available prompts:");
        println!("- project_analysis: Analyze project structure and suggest improvements");
        return Ok(());
    }

    // Default behavior is to start MCP server unless showing capabilities

    // Set up signal handling for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGINT]).unwrap();
        for _ in signals.forever() {
            r.store(false, Ordering::SeqCst);
            break;
        }
    });

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);

    for line_result in reader.lines() {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }

        // Log incoming request to stderr for debugging
        eprintln!("Received: {}", line);

        let request: Value = serde_json::from_str(&line)?;

        if let Some(method) = request.get("method").and_then(|m| m.as_str()) {
            match method {
                "notifications/initialized"
                | "notifications/cancelled"
                | "notifications/progress" => {
                    // Notifications don't require responses
                    continue;
                }
                method_name => {
                    let params = request.get("params").cloned();

                    match handle_rpc_request(method_name, params).await {
                        Ok(result) => {
                            let response_json = json!({
                                "jsonrpc": "2.0",
                                "id": request.get("id"),
                                "result": result
                            });
                            println!("{}", serde_json::to_string(&response_json)?);
                        }
                        Err(error) => {
                            let error_response = json!({
                                "jsonrpc": "2.0",
                                "id": request.get("id"),
                                "error": {
                                    "code": -32603,
                                    "message": error.to_string()
                                }
                            });
                            println!("{}", serde_json::to_string(&error_response)?);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
