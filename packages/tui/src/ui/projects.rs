use crate::state::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

/// Render the projects screen
pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    
    let block = Block::default()
        .title("Projects")
        .borders(Borders::ALL);
    
    if state.projects.is_empty() {
        let paragraph = ratatui::widgets::Paragraph::new("No projects found.\n\nPress 'd' for Dashboard, 's' for Settings, 'q' to quit")
            .block(block);
        frame.render_widget(paragraph, area);
    } else {
        let items: Vec<ListItem> = state.projects
            .iter()
            .enumerate()
            .map(|(i, project)| {
                let name = &project.name;
                let status = format!("{:?}", project.status).to_lowercase();
                    
                let content = format!("{} ({})", name, status);
                if Some(i) == state.selected_project {
                    ListItem::new(content).style(Style::default().bg(Color::Blue))
                } else {
                    ListItem::new(content)
                }
            })
            .collect();
            
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::Blue));
            
        frame.render_widget(list, area);
    }
}