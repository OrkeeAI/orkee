use crate::search_popup::{SearchPopup, ProjectMatch, SearchMode, MatchedField};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
    layout::{Layout, Direction, Constraint},
};

/// Widget for rendering the project search popup
pub struct SearchPopupWidget<'a> {
    popup: &'a SearchPopup,
    max_rows: u16,
    show_help: bool,
}

impl<'a> SearchPopupWidget<'a> {
    /// Create a new search popup widget
    pub fn new(popup: &'a SearchPopup) -> Self {
        Self {
            popup,
            max_rows: 10, // Default max rows for search results
            show_help: true,
        }
    }
    
    /// Set the maximum number of result rows to display
    pub fn max_rows(mut self, max_rows: u16) -> Self {
        self.max_rows = max_rows;
        self
    }

    /// Set whether to show help text at bottom
    pub fn show_help(mut self, show_help: bool) -> Self {
        self.show_help = show_help;
        self
    }
}

impl<'a> Widget for SearchPopupWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate layout areas
        let layout_constraints = if self.show_help {
            vec![
                Constraint::Length(3), // Search input area
                Constraint::Length(2), // Filters area  
                Constraint::Min(5),    // Results area
                Constraint::Length(2), // Help text area
            ]
        } else {
            vec![
                Constraint::Length(3), // Search input area
                Constraint::Length(2), // Filters area
                Constraint::Min(5),    // Results area
            ]
        };

        let _main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(layout_constraints.clone())
            .split(area);

        // Render main border
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title("🔍 Search Projects")
            .title_style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Color::Blue));
        
        // Get inner area before rendering
        let inner_area = main_block.inner(area);
        main_block.render(area, buf);
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(layout_constraints)
            .split(inner_area);

        // 1. Render search input area
        self.render_search_input(areas[0], buf);
        
        // 2. Render active filters
        self.render_filters(areas[1], buf);
        
        // 3. Render search results
        self.render_results(areas[2], buf);
        
        // 4. Render help text if enabled
        if self.show_help && areas.len() > 3 {
            self.render_help(areas[3], buf);
        }
    }
}

impl<'a> SearchPopupWidget<'a> {
    /// Render the search input field
    fn render_search_input(&self, area: Rect, buf: &mut Buffer) {
        let query = self.popup.search_query();
        let mode_text = match self.popup.search_mode() {
            SearchMode::Text => "",
            SearchMode::Status => " [Status Mode]",
            SearchMode::Priority => " [Priority Mode]",
            SearchMode::Tags => " [Tags Mode]",
        };

        let search_text = format!("Search: {}{}", query, mode_text);
        let input_paragraph = Paragraph::new(search_text)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        input_paragraph.render(area, buf);

        // Render cursor if in text mode
        if matches!(self.popup.search_mode(), SearchMode::Text) {
            let cursor_x = area.x + "Search: ".len() as u16 + query.len() as u16;
            if cursor_x < area.x + area.width && area.y < buf.area().bottom() {
                buf[(cursor_x, area.y)].set_style(
                    Style::default().bg(Color::White).fg(Color::Black)
                );
            }
        }
    }

    /// Render active filters
    fn render_filters(&self, area: Rect, buf: &mut Buffer) {
        let active_filters = self.popup.active_filters();
        
        let filter_text = if active_filters.is_empty() {
            "Filters: None".to_string()
        } else {
            format!("Filters: {}", active_filters.join(" | "))
        };

        let filter_color = if self.popup.has_active_filters() {
            Color::Yellow
        } else {
            Color::Gray
        };

        let filters_paragraph = Paragraph::new(filter_text)
            .style(Style::default().fg(filter_color))
            .wrap(Wrap { trim: true });

        filters_paragraph.render(area, buf);
    }

    /// Render search results
    fn render_results(&self, area: Rect, buf: &mut Buffer) {
        let results = self.popup.visible_results();
        
        if results.is_empty() {
            let no_results_text = if self.popup.search_query().is_empty() && !self.popup.has_active_filters() {
                "Enter search text or apply filters..."
            } else {
                "No matching projects found."
            };
            
            let no_results = Paragraph::new(no_results_text)
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: true });
            
            no_results.render(area, buf);
            return;
        }

        // Render results header
        let results_count = self.popup.filtered_results().len();
        let header_text = format!("Results ({} projects):", results_count);
        
        if area.height > 0 {
            let header_paragraph = Paragraph::new(header_text)
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
            
            let header_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 1,
            };
            header_paragraph.render(header_area, buf);
        }

        // Render individual results
        let results_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height.saturating_sub(1),
        };

        let display_count = (results.len() as u16).min(self.max_rows).min(results_area.height);
        
        for (row_idx, project_match) in results.iter().enumerate().take(display_count as usize) {
            if row_idx as u16 >= results_area.height {
                break;
            }
            
            let y = results_area.y + row_idx as u16;
            let is_selected = row_idx == self.popup.selected_index();
            
            self.render_project_result(project_match, is_selected, results_area.x, y, results_area.width, buf);
        }
    }

    /// Render a single project result
    fn render_project_result(&self, project_match: &ProjectMatch, is_selected: bool, x: u16, y: u16, width: u16, buf: &mut Buffer) {
        // Background style for selected item
        let bg_style = if is_selected {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };
        
        // Clear the line with background color
        for i in 0..width {
            if x + i < buf.area().right() && y < buf.area().bottom() {
                buf[(x + i, y)].set_style(bg_style);
            }
        }

        let mut current_x = x;
        
        // Selection indicator
        let indicator = if is_selected { ">> " } else { "   " };
        for ch in indicator.chars() {
            if current_x < x + width && y < buf.area().bottom() {
                buf[(current_x, y)]
                    .set_char(ch)
                    .set_style(bg_style.add_modifier(Modifier::BOLD));
                current_x += 1;
            }
        }

        // Project icon
        if current_x < x + width {
            buf[(current_x, y)]
                .set_char('📁')
                .set_style(bg_style.fg(Color::Cyan));
            current_x += 1;
        }

        // Space after icon
        if current_x < x + width {
            buf[(current_x, y)].set_char(' ').set_style(bg_style);
            current_x += 1;
        }

        // Project name with highlighting
        self.render_text_with_highlighting(
            &project_match.project.name,
            &project_match.match_indices,
            matches!(project_match.matched_field, MatchedField::Name),
            current_x,
            y,
            x + width,
            bg_style,
            buf,
        );
        
        current_x += project_match.project.name.len() as u16;

        // Status indicator
        let status_text = format!(" ({})", format!("{:?}", project_match.project.status).to_lowercase());
        let status_color = match project_match.project.status {
            orkee_projects::ProjectStatus::Active => Color::Green,
            orkee_projects::ProjectStatus::Archived => Color::Yellow,
        };

        for ch in status_text.chars() {
            if current_x < x + width && y < buf.area().bottom() {
                buf[(current_x, y)]
                    .set_char(ch)
                    .set_style(bg_style.fg(status_color));
                current_x += 1;
            } else {
                break;
            }
        }

        // Show matched field info for non-name matches
        if !matches!(project_match.matched_field, MatchedField::Name) {
            let field_info = match &project_match.matched_field {
                MatchedField::Path => " [path]",
                MatchedField::Description => " [desc]",
                MatchedField::Tag(_tag) => " [tag]",
                MatchedField::Name => "",
            };
            
            for ch in field_info.chars() {
                if current_x < x + width && y < buf.area().bottom() {
                    buf[(current_x, y)]
                        .set_char(ch)
                        .set_style(bg_style.fg(Color::Gray));
                    current_x += 1;
                } else {
                    break;
                }
            }
        }
    }

    /// Render text with fuzzy match highlighting
    fn render_text_with_highlighting(
        &self,
        text: &str,
        match_indices: &[usize],
        is_matched_field: bool,
        start_x: u16,
        y: u16,
        max_x: u16,
        base_style: Style,
        buf: &mut Buffer,
    ) {
        let chars: Vec<char> = text.chars().collect();
        let mut current_x = start_x;
        
        for (char_idx, ch) in chars.iter().enumerate() {
            if current_x >= max_x {
                break;
            }
            
            let char_style = if is_matched_field && match_indices.contains(&char_idx) {
                // Highlight matched characters
                base_style.fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                // Normal text color
                base_style.fg(Color::White)
            };
            
            if y < buf.area().bottom() {
                buf[(current_x, y)]
                    .set_char(*ch)
                    .set_style(char_style);
            }
            
            current_x += 1;
        }
    }

    /// Render help text
    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let help_text = "↑↓ Navigate • Enter Select • Tab Filters • Shift+F Close • Esc Cancel";
        
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: true });
        
        help_paragraph.render(area, buf);
    }
}

/// Calculate the popup area for the search widget (centered overlay)
pub fn calculate_search_popup_area(full_area: Rect) -> Rect {
    let popup_width = std::cmp::min(80, full_area.width.saturating_sub(4));
    let popup_height = std::cmp::min(20, full_area.height.saturating_sub(4));
    
    let x = full_area.width.saturating_sub(popup_width) / 2;
    let y = full_area.height.saturating_sub(popup_height) / 2;
    
    Rect {
        x: full_area.x + x,
        y: full_area.y + y,
        width: popup_width,
        height: popup_height,
    }
}