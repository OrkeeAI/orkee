use crate::state::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Render the dashboard screen
pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    
    let block = Block::default()
        .title("Orkee Dashboard")
        .borders(Borders::ALL);
    
    let content = format!(
        "Projects: {}\n\nPress 'p' for Projects, 's' for Settings, 'q' to quit",
        state.projects.len()
    );
    
    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}