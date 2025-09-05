pub mod widgets;
pub mod chat;
pub mod dashboard;
pub mod projects;

use crate::state::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};
use widgets::ConfirmationDialogWidget;

/// Main UI rendering function
pub fn render(frame: &mut Frame, state: &AppState) {
    // Render the main screen content
    match state.current_screen {
        crate::state::Screen::Chat => chat::render(frame, state),
        crate::state::Screen::Dashboard => dashboard::render(frame, state),
        crate::state::Screen::Projects => {
            if state.is_form_mode() {
                projects::render_form(frame, state);
            } else {
                projects::render(frame, state);
            }
        }
        crate::state::Screen::ProjectDetail => projects::render_detail(frame, state),
        crate::state::Screen::Settings => {
            // TODO: Implement settings screen
            let block = Block::default()
                .title("Settings")
                .borders(Borders::ALL);
            frame.render_widget(block, frame.area());
        }
    }

    // Render confirmation dialog on top if one is active
    if let Some(dialog) = &state.confirmation_dialog {
        let dialog_widget = ConfirmationDialogWidget::new(dialog);
        frame.render_widget(dialog_widget, frame.area());
    }
}