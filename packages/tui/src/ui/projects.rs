use crate::state::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::layout::{Layout, Direction, Constraint};

/// Render the projects screen
pub fn render(frame: &mut Frame, state: &AppState) {
    render_with_area(frame, state, frame.area());
}

/// Render the projects screen with specific area
pub fn render_with_area(frame: &mut Frame, state: &AppState, area: Rect) {
    
    if state.projects.is_empty() {
        let block = Block::default()
            .title("Projects - No projects found")
            .title_style(Style::default().fg(Color::Yellow))
            .borders(Borders::ALL);
            
        let help_text = "No projects found.\n\nKeyboard shortcuts:\n• 'n' - Create new project\n• 'd' - Dashboard\n• 's' - Settings\n• 'q' - Quit";
        let paragraph = Paragraph::new(help_text)
            .block(block)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(paragraph, area);
    } else {
        let title = format!("Projects ({}) - ↑↓ Navigate • Enter Details • Esc Back to Chat • n New • e Edit • d Delete", state.projects.len());
        let block = Block::default()
            .title(title)
            .title_style(Style::default().fg(Color::Green))
            .borders(Borders::ALL);
            
        let items: Vec<ListItem> = state.projects
            .iter()
            .enumerate()
            .map(|(_i, project)| {
                let name = &project.name;
                let status = format!("{:?}", project.status).to_lowercase();
                let status_color = match status.as_str() {
                    "active" => Color::Green,
                    "inactive" => Color::Gray,
                    "archived" => Color::Yellow,
                    _ => Color::White,
                };
                
                // Create a more detailed display
                let mut lines = vec![
                    Line::from(vec![
                        Span::styled(format!("📁 {}", name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::raw("  "),
                        Span::styled(format!("({})", status), Style::default().fg(status_color)),
                    ])
                ];
                
                // Add project path in a subtle way
                if let Some(path_part) = project.project_root.split('/').last() {
                    if path_part != name {
                        lines.push(Line::from(vec![
                            Span::raw("   "),
                            Span::styled(format!("📂 {}", path_part), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                        ]));
                    }
                }
                
                // Add description if present
                if let Some(description) = &project.description {
                    if !description.is_empty() {
                        let desc_preview = if description.len() > 50 {
                            format!("{}...", &description[..47])
                        } else {
                            description.clone()
                        };
                        lines.push(Line::from(vec![
                            Span::raw("   "),
                            Span::styled(desc_preview, Style::default().fg(Color::Gray)),
                        ]));
                    }
                }
                
                ListItem::new(Text::from(lines))
            })
            .collect();
            
        let mut list_state = ListState::default();
        list_state.select(state.selected_project);
        
        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol(">> ");
            
        frame.render_stateful_widget(list, area, &mut list_state);
    }
}

/// Render the project detail screen
pub fn render_detail(frame: &mut Frame, state: &AppState) {
    render_detail_with_area(frame, state, frame.area());
}

/// Render the project detail screen with specific area
pub fn render_detail_with_area(frame: &mut Frame, state: &AppState, area: Rect) {
    
    if let Some(project) = state.get_selected_project() {
        let title = format!("Project Details: {} - Esc Back • e Edit • d Delete", project.name);
        let block = Block::default()
            .title(title)
            .title_style(Style::default().fg(Color::Cyan))
            .borders(Borders::ALL);
            
        // Format project details
        let mut details = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(project.name.clone(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(project.id.clone(), Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Path: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(project.project_root.clone(), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:?}", project.status), match format!("{:?}", project.status).to_lowercase().as_str() {
                    "active" => Style::default().fg(Color::Green),
                    "inactive" => Style::default().fg(Color::Gray),
                    "archived" => Style::default().fg(Color::Yellow),
                    _ => Style::default().fg(Color::White),
                }),
            ]),
            Line::from(vec![
                Span::styled("Priority: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:?}", project.priority), Style::default().fg(Color::White)),
            ]),
            Line::raw(""),
        ];
        
        // Add description if present
        if let Some(description) = &project.description {
            details.push(Line::from(vec![
                Span::styled("Description:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            details.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(description.clone(), Style::default().fg(Color::White)),
            ]));
            details.push(Line::raw(""));
        }
        
        // Add tags if present
        if let Some(tags) = &project.tags {
            if !tags.is_empty() {
                details.push(Line::from(vec![
                    Span::styled("Tags: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(tags.join(", "), Style::default().fg(Color::Magenta)),
                ]));
                details.push(Line::raw(""));
            }
        }
        
        // Add scripts if present
        if project.setup_script.is_some() || project.dev_script.is_some() || project.cleanup_script.is_some() {
            details.push(Line::from(vec![
                Span::styled("Scripts:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            
            if let Some(setup) = &project.setup_script {
                details.push(Line::from(vec![
                    Span::raw("  Setup: "),
                    Span::styled(setup.clone(), Style::default().fg(Color::Cyan)),
                ]));
            }
            
            if let Some(dev) = &project.dev_script {
                details.push(Line::from(vec![
                    Span::raw("  Dev: "),
                    Span::styled(dev.clone(), Style::default().fg(Color::Cyan)),
                ]));
            }
            
            if let Some(cleanup) = &project.cleanup_script {
                details.push(Line::from(vec![
                    Span::raw("  Cleanup: "),
                    Span::styled(cleanup.clone(), Style::default().fg(Color::Cyan)),
                ]));
            }
            
            details.push(Line::raw(""));
        }
        
        // Add git info if available
        if let Some(git_info) = &project.git_repository {
            details.push(Line::from(vec![
                Span::styled("Git Repository:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            details.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(git_info.url.clone(), Style::default().fg(Color::Blue)),
            ]));
            details.push(Line::from(vec![
                Span::raw("  Branch: "),
                Span::styled(git_info.branch.as_deref().unwrap_or("unknown").to_string(), Style::default().fg(Color::Green)),
            ]));
            details.push(Line::raw(""));
        }
        
        // Add timestamps
        details.push(Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(project.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(), Style::default().fg(Color::Gray)),
        ]));
        details.push(Line::from(vec![
            Span::styled("Updated: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(project.updated_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(), Style::default().fg(Color::Gray)),
        ]));
        
        let text = Text::from(details);
        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });
            
        frame.render_widget(paragraph, area);
    } else {
        // No project selected - this shouldn't happen, but handle it gracefully
        let block = Block::default()
            .title("Project Details - No Project Selected")
            .title_style(Style::default().fg(Color::Red))
            .borders(Borders::ALL);
            
        let paragraph = Paragraph::new("No project selected. Press Esc to return to projects list.")
            .block(block)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(paragraph, area);
    }
}

/// Render the project creation form
pub fn render_form(frame: &mut Frame, state: &AppState) {
    render_form_with_area(frame, state, frame.area());
}

/// Render the project creation form with specific area
pub fn render_form_with_area(frame: &mut Frame, state: &AppState, area: Rect) {
    
    if let Some(form) = state.form() {
        // Check if there are recent system messages to display (errors or success)
        let recent_system_message = state.message_history.messages()
            .iter()
            .rev()
            .take(3) // Check last 3 messages
            .find(|msg| (msg.content.contains("❌") || msg.content.contains("✅")) && msg.author == crate::chat::MessageAuthor::System)
            .map(|msg| (msg.content.clone(), msg.content.contains("❌")));
        
        if let Some((message, is_error)) = recent_system_message {
            // Split area to show form and notification message
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(10), // Form area
                    Constraint::Length(6), // Message area
                ])
                .split(area);
            
            // Render form in top area
            form.render(frame, chunks[0]);
            
            // Render notification message in bottom area
            let (title, border_color) = if is_error {
                ("⚠️ Error", Color::Red)
            } else {
                ("✅ Success", Color::Green)
            };
            
            let message_block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color));
            
            let message_paragraph = Paragraph::new(message)
                .block(message_block)
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true });
            
            frame.render_widget(message_paragraph, chunks[1]);
        } else {
            // No notification message - render form normally
            form.render(frame, area);
        }
    } else {
        // Fallback to regular projects view if no form
        render(frame, state);
    }
}