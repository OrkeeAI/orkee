pub mod widgets;
pub mod chat;
pub mod dashboard;
pub mod projects;

use crate::state::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};
use ratatui::layout::{Constraint, Direction, Layout};
use widgets::{ConfirmationDialogWidget, StatusBarWidget};

/// Main UI rendering function
pub fn render(frame: &mut Frame, state: &AppState) {
    // Create layout with status bar at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3), // Main content area (flexible)
            Constraint::Length(1), // Status bar (fixed height)
        ])
        .split(frame.area());

    let main_area = chunks[0];
    let status_area = chunks[1];

    // Render the main screen content
    match state.current_screen {
        crate::state::Screen::Chat => chat::render_with_area(frame, state, main_area),
        crate::state::Screen::Dashboard => dashboard::render_with_area(frame, state, main_area),
        crate::state::Screen::Projects => {
            if state.is_form_mode() {
                projects::render_form_with_area(frame, state, main_area);
            } else {
                projects::render_with_area(frame, state, main_area);
            }
        }
        crate::state::Screen::ProjectDetail => projects::render_detail_with_area(frame, state, main_area),
        crate::state::Screen::Settings => {
            // TODO: Implement settings screen
            let block = Block::default()
                .title("Settings")
                .borders(Borders::ALL);
            frame.render_widget(block, main_area);
        }
    }

    // Render status bar at bottom
    let status_bar = StatusBarWidget::new(state);
    frame.render_widget(status_bar, status_area);

    // Render confirmation dialog on top if one is active
    if let Some(dialog) = &state.confirmation_dialog {
        let dialog_widget = ConfirmationDialogWidget::new(dialog);
        frame.render_widget(dialog_widget, frame.area());
    }
}