pub mod widgets;
pub mod chat;
pub mod dashboard;
pub mod projects;

use crate::state::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

/// Main UI rendering function
pub fn render(frame: &mut Frame, state: &AppState) {
    match state.current_screen {
        crate::state::Screen::Chat => chat::render(frame, state),
        crate::state::Screen::Dashboard => dashboard::render(frame, state),
        crate::state::Screen::Projects => projects::render(frame, state),
        crate::state::Screen::Settings => {
            // TODO: Implement settings screen
            let block = Block::default()
                .title("Settings")
                .borders(Borders::ALL);
            frame.render_widget(block, frame.area());
        }
    }
}