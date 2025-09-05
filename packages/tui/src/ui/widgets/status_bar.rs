use crate::state::{AppState, Screen, FocusArea};
use crate::input::InputMode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};
use ratatui::layout::{Constraint, Direction, Layout};

/// Status bar widget that displays context-aware information
pub struct StatusBarWidget<'a> {
    state: &'a AppState,
}

impl<'a> StatusBarWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    /// Get the current mode indicator text and style (only when meaningful)
    fn get_mode_info(&self) -> Option<(String, Style)> {
        match self.state.input_mode() {
            InputMode::Normal => {
                // Only show focus areas on Chat screen where it's relevant
                match &self.state.current_screen {
                    &Screen::Chat => {
                        match self.state.focus_area() {
                            FocusArea::Chat => Some(("CHAT".to_string(), Style::default().fg(Color::Cyan))),
                            FocusArea::Input => Some(("INPUT".to_string(), Style::default().fg(Color::White))),
                        }
                    }
                    _ => {
                        // On other screens in normal mode, don't show any mode indicator
                        None
                    }
                }
            }
            InputMode::Command => Some(("COMMAND".to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            InputMode::Search => Some(("MENTION".to_string(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
            InputMode::History => Some(("HISTORY".to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))),
            InputMode::Edit => Some(("EDIT".to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            InputMode::Form => Some(("FORM".to_string(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
        }
    }

    /// Get navigation breadcrumb text
    fn get_navigation_breadcrumb(&self) -> String {
        match &self.state.current_screen {
            &Screen::Chat => " Chat".to_string(),
            &Screen::Dashboard => " Dashboard".to_string(),
            &Screen::Projects => {
                if self.state.is_form_mode() {
                    match &self.state.form_state {
                        Some(form_state) => {
                            match &form_state.form_mode {
                                crate::state::FormMode::Create => format!(" New Project (Step {}/{})", form_state.step, form_state.total_steps),
                                crate::state::FormMode::Edit(project_id) => {
                                    // Try to find project name for better breadcrumb
                                    if let Some(project) = self.state.projects.iter().find(|p| &p.id == project_id) {
                                        format!(" Edit: {} (Step {}/{})", project.name, form_state.step, form_state.total_steps)
                                    } else {
                                        format!(" Edit Project (Step {}/{})", form_state.step, form_state.total_steps)
                                    }
                                }
                            }
                        }
                        None => " Projects Form".to_string(),
                    }
                } else {
                    if self.state.projects.is_empty() {
                        " Projects (empty)".to_string()
                    } else if let Some(selected_idx) = self.state.selected_project {
                        format!(" Projects ({}/{})", selected_idx + 1, self.state.projects.len())
                    } else {
                        format!(" Projects ({})", self.state.projects.len())
                    }
                }
            }
            &Screen::ProjectDetail => {
                if let Some(project) = self.state.get_selected_project() {
                    format!(" {}", project.name)
                } else {
                    " Project Detail".to_string()
                }
            }
            &Screen::Settings => " Settings".to_string(),
        }
    }

    /// Get context-aware keyboard shortcuts
    fn get_shortcuts(&self) -> String {
        if self.state.is_showing_confirmation_dialog() {
            return "Tab: Switch • Enter: Confirm • Esc: Cancel".to_string();
        }

        if self.state.is_form_mode() {
            return match self.state.form_is_review_step() {
                true => "Enter: Submit • Esc: Cancel • ↑: Previous",
                false => "Enter/↓/Tab: Next • ↑: Previous • Esc: Cancel",
            }.to_string();
        }

        match (&self.state.current_screen, self.state.input_mode()) {
            (&Screen::Chat, InputMode::Command) => "↑↓: Navigate • Tab: Complete • Esc: Cancel".to_string(),
            (&Screen::Chat, InputMode::Search) => "↑↓: Navigate • Enter: Select • Esc: Cancel".to_string(),
            (&Screen::Chat, InputMode::History) => "↑↓: Navigate • Enter: Select • Esc: Cancel".to_string(),
            (&Screen::Chat, InputMode::Edit) => "Enter: Save • Esc: Cancel".to_string(),
            (&Screen::Chat, _) => {
                match self.state.focus_area() {
                    FocusArea::Chat => "↑↓: Scroll • Tab: Focus Input • q: Quit".to_string(),
                    FocusArea::Input => "Enter: Send • /: Commands • @: Mentions • Tab: Focus Chat".to_string(),
                }
            }
            (&Screen::Projects, _) => {
                if self.state.projects.is_empty() {
                    "n: New Project • Tab: Navigate • q: Quit".to_string()
                } else {
                    "↑↓: Navigate • Enter: Details • n: New • e: Edit • d: Delete • Tab: Switch Screen".to_string()
                }
            }
            (&Screen::ProjectDetail, _) => "e: Edit • d: Delete • Esc: Back to List • Tab: Navigate".to_string(),
            (&Screen::Dashboard, _) => "Tab: Navigate • q: Quit".to_string(),
            (&Screen::Settings, _) => "Tab: Navigate • q: Quit".to_string(),
        }
    }

    /// Get project context information (when relevant)
    fn get_project_context(&self) -> Option<String> {
        match &self.state.current_screen {
            &Screen::Projects | &Screen::ProjectDetail => {
                if let Some(project) = self.state.get_selected_project() {
                    // Build context string with GitHub info and path
                    let mut context_parts = Vec::new();
                    
                    // Add GitHub username/repo if available
                    if let Some(ref git_repo) = project.git_repository {
                        context_parts.push(format!("🔗 {}/{}", git_repo.owner, git_repo.repo));
                    }
                    
                    // Add project path
                    context_parts.push(format!("📁 {}", project.project_root));
                    
                    Some(context_parts.join(" • "))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get input history position indicator
    fn get_history_position(&self) -> Option<String> {
        if let Some((current, total)) = self.state.input_history_position() {
            Some(format!("History: {}/{}", current + 1, total))
        } else {
            None
        }
    }
}

impl<'a> Widget for StatusBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mode_info = self.get_mode_info();
        let breadcrumb = self.get_navigation_breadcrumb();
        let shortcuts = self.get_shortcuts();
        let project_context = self.get_project_context();
        let history_position = self.get_history_position();

        // Create layout for status bar sections based on whether we have mode info
        let chunks = if let Some((ref mode_text, _)) = mode_info {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(mode_text.len() as u16 + 2), // Mode indicator
                    Constraint::Min(20), // Breadcrumb and context (flexible)
                    Constraint::Length(shortcuts.len() as u16), // Shortcuts (right-aligned)
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(20), // Breadcrumb and context (flexible)
                    Constraint::Length(shortcuts.len() as u16), // Shortcuts (right-aligned)
                ])
                .split(area)
        };

        let mut current_chunk = 0;

        // Render mode indicator if present
        if let Some((mode_text, mode_style)) = mode_info {
            let mode_paragraph = Paragraph::new(format!(" {} ", mode_text))
                .style(mode_style)
                .block(Block::default());
            mode_paragraph.render(chunks[current_chunk], buf);
            current_chunk += 1;
        }

        // Build middle section content
        let mut middle_content = breadcrumb;

        // Add project context if available
        if let Some(context) = project_context {
            middle_content = format!("{} • {}", middle_content, context);
        }

        // Add history position if available
        if let Some(history) = history_position {
            middle_content = format!("{} • {}", middle_content, history);
        }

        // Render middle section (breadcrumb + context)
        let middle_paragraph = Paragraph::new(middle_content)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default());
        middle_paragraph.render(chunks[current_chunk], buf);
        current_chunk += 1;

        // Render shortcuts (right-aligned)
        let shortcuts_paragraph = Paragraph::new(shortcuts)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default());
        shortcuts_paragraph.render(chunks[current_chunk], buf);
    }
}