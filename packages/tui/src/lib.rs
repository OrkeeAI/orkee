//! Orkee TUI - Terminal User Interface for Orkee
//!
//! This library provides a terminal-based user interface for the Orkee
//! AI agent orchestration platform, built with ratatui.

#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::type_complexity)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::option_as_ref_deref)]
#![allow(clippy::unnecessary_lazy_evaluations)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::unused_enumerate_index)]

pub mod app;
pub mod chat;
pub mod command_popup;
pub mod events;
pub mod input;
pub mod mention_popup;
pub mod search_popup;
pub mod slash_command;
pub mod state;
pub mod ui;

pub use app::App;
pub use state::AppState;
