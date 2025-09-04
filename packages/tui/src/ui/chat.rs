use crate::state::AppState;
use crate::ui::widgets::{ChatWidget, InputWidget};
use ratatui::prelude::*;

/// Render the chat interface
pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    
    // Create layout with chat area and input area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),         // Chat area (takes most space)
            Constraint::Length(3),      // Input area (fixed height)
        ])
        .split(area);
    
    // Render chat messages
    let chat_widget = ChatWidget::new(state.messages())
        .scroll_offset(state.scroll_offset())
        .show_timestamps(false);
    
    frame.render_widget(chat_widget, chunks[0]);
    
    // Render input area with real input buffer
    let input_widget = InputWidget::new(state.input_buffer(), state.input_mode())
        .history_position(state.input_history_position())
        .placeholder("Type a message and press Enter...");
    
    frame.render_widget(input_widget, chunks[1]);
}