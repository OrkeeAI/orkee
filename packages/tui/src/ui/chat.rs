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
    
    // Render input area (placeholder for now)
    let input_widget = InputWidget::new("")
        .placeholder("Type a message... (Input handling coming in Phase 2)");
    
    frame.render_widget(input_widget, chunks[1]);
}