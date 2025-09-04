//! Orkee TUI - Terminal User Interface for Orkee
//! 
//! This library provides a terminal-based user interface for the Orkee
//! AI agent orchestration platform, built with ratatui.

pub mod app;
pub mod chat;
pub mod command_popup;
pub mod events;
pub mod input;
pub mod slash_command;
pub mod state;
pub mod ui;

pub use app::App;
pub use state::AppState;