use crate::search_popup::{MatchedField, ProjectMatch, SearchMode, SearchPopup};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Parameters for text rendering with highlighting
struct RenderParams {
    start_x: u16,
    y: u16,
    max_x: u16,
    base_style: Style,
}

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
                Constraint::Length(4), // Search input area (2 lines for status/priority/tags)
                Constraint::Length(2), // Filters area
                Constraint::Min(5),    // Results area
                Constraint::Length(2), // Help text area
            ]
        } else {
            vec![
                Constraint::Length(4), // Search input area (2 lines for status/priority/tags)
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
            .title_style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
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
        match self.popup.search_mode() {
            SearchMode::Text => {
                self.render_text_search_input(area, buf);
            }
            SearchMode::Status => {
                self.render_status_selection(area, buf);
            }
            SearchMode::Priority => {
                self.render_priority_selection(area, buf);
            }
            SearchMode::Tags => {
                self.render_tags_input(area, buf);
            }
        }
    }

    /// Render text search input
    fn render_text_search_input(&self, area: Rect, buf: &mut Buffer) {
        let query = self.popup.search_query();
        let search_text = format!("Search: {}", query);
        let input_paragraph = Paragraph::new(search_text)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        input_paragraph.render(area, buf);

        // Render cursor
        let cursor_x = area.x + "Search: ".len() as u16 + query.len() as u16;
        if cursor_x < area.x + area.width && area.y < buf.area().bottom() {
            buf[(cursor_x, area.y)].set_style(Style::default().bg(Color::White).fg(Color::Black));
        }
    }

    /// Render status selection radio buttons
    fn render_status_selection(&self, area: Rect, buf: &mut Buffer) {
        let current_status = self.popup.get_status_filter();

        let line1 = "Status Filter: (1/a=Active  2/r=Archived  0/c=Clear)";
        let line2 = self.format_status_options(current_status);

        // Render first line (instructions)
        if area.height > 0 {
            let instr_paragraph = Paragraph::new(line1)
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: true });
            let instr_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 1,
            };
            instr_paragraph.render(instr_area, buf);
        }

        // Render second line (options)
        if area.height > 1 {
            let options_paragraph = Paragraph::new(line2)
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true });
            let options_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: 1,
            };
            options_paragraph.render(options_area, buf);
        }
    }

    /// Render priority selection radio buttons
    fn render_priority_selection(&self, area: Rect, buf: &mut Buffer) {
        let current_priority = self.popup.get_priority_filter();

        let line1 = "Priority Filter: (1/h=High  2/m=Medium  3/l=Low  0/c=Clear)";
        let line2 = self.format_priority_options(current_priority);

        // Render first line (instructions)
        if area.height > 0 {
            let instr_paragraph = Paragraph::new(line1)
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: true });
            let instr_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 1,
            };
            instr_paragraph.render(instr_area, buf);
        }

        // Render second line (options)
        if area.height > 1 {
            let options_paragraph = Paragraph::new(line2)
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true });
            let options_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: 1,
            };
            options_paragraph.render(options_area, buf);
        }
    }

    /// Render tags input with existing tag chips
    fn render_tags_input(&self, area: Rect, buf: &mut Buffer) {
        let query = self.popup.search_query();
        let tag_filters = self.popup.get_tag_filters();

        // Line 1: Existing tag filters as chips
        let chips_line = if tag_filters.is_empty() {
            "Tag Filters: (none)".to_string()
        } else {
            let chips = tag_filters
                .iter()
                .map(|tag| format!("[{} ×]", tag))
                .collect::<Vec<_>>()
                .join(" ");
            format!("Tag Filters: {}", chips)
        };

        // Line 2: Current input
        let input_line = format!("Add Tag: {}", query);

        // Render chips line
        if area.height > 0 {
            let chips_paragraph = Paragraph::new(chips_line)
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });
            let chips_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 1,
            };
            chips_paragraph.render(chips_area, buf);
        }

        // Render input line
        if area.height > 1 {
            let input_paragraph = Paragraph::new(input_line)
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true });
            let input_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: 1,
            };
            input_paragraph.render(input_area, buf);

            // Render cursor
            let cursor_x = area.x + "Add Tag: ".len() as u16 + query.len() as u16;
            if cursor_x < area.x + area.width && area.y + 1 < buf.area().bottom() {
                buf[(cursor_x, area.y + 1)]
                    .set_style(Style::default().bg(Color::White).fg(Color::Black));
            }
        }
    }

    /// Format status options with radio button indicators
    fn format_status_options(
        &self,
        current_status: &Option<orkee_projects::ProjectStatus>,
    ) -> String {
        let active_indicator =
            if matches!(current_status, Some(orkee_projects::ProjectStatus::Planning)) {
                "●"
            } else {
                "○"
            };
        let archived_indicator = if matches!(
            current_status,
            Some(orkee_projects::ProjectStatus::Archived)
        ) {
            "●"
        } else {
            "○"
        };
        let all_indicator = if current_status.is_none() {
            "●"
        } else {
            "○"
        };

        format!(
            "{} Active    {} Archived    {} All",
            active_indicator, archived_indicator, all_indicator
        )
    }

    /// Format priority options with radio button indicators
    fn format_priority_options(
        &self,
        current_priority: &Option<orkee_projects::Priority>,
    ) -> String {
        let high_indicator = if matches!(current_priority, Some(orkee_projects::Priority::High)) {
            "●"
        } else {
            "○"
        };
        let medium_indicator = if matches!(current_priority, Some(orkee_projects::Priority::Medium))
        {
            "●"
        } else {
            "○"
        };
        let low_indicator = if matches!(current_priority, Some(orkee_projects::Priority::Low)) {
            "●"
        } else {
            "○"
        };
        let all_indicator = if current_priority.is_none() {
            "●"
        } else {
            "○"
        };

        format!(
            "{} High    {} Medium    {} Low    {} All",
            high_indicator, medium_indicator, low_indicator, all_indicator
        )
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
            let no_results_text =
                if self.popup.search_query().is_empty() && !self.popup.has_active_filters() {
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
            let header_paragraph = Paragraph::new(header_text).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

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

        let display_count = (results.len() as u16)
            .min(self.max_rows)
            .min(results_area.height);

        for (row_idx, project_match) in results.iter().enumerate().take(display_count as usize) {
            if row_idx as u16 >= results_area.height {
                break;
            }

            let y = results_area.y + row_idx as u16;
            let is_selected = row_idx == self.popup.selected_index();

            self.render_project_result(
                project_match,
                is_selected,
                results_area.x,
                y,
                results_area.width,
                buf,
            );
        }
    }

    /// Render a single project result
    fn render_project_result(
        &self,
        project_match: &ProjectMatch,
        is_selected: bool,
        x: u16,
        y: u16,
        width: u16,
        buf: &mut Buffer,
    ) {
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
            RenderParams {
                start_x: current_x,
                y,
                max_x: x + width,
                base_style: bg_style,
            },
            buf,
        );

        current_x += project_match.project.name.len() as u16;

        // Status indicator
        let status_text = format!(
            " ({})",
            format!("{:?}", project_match.project.status).to_lowercase()
        );
        let status_color = match project_match.project.status {
            orkee_projects::ProjectStatus::Planning => Color::Cyan,
            orkee_projects::ProjectStatus::Building => Color::Blue,
            orkee_projects::ProjectStatus::Review => Color::Magenta,
            orkee_projects::ProjectStatus::Launched => Color::Green,
            orkee_projects::ProjectStatus::OnHold => Color::LightYellow,
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
        params: RenderParams,
        buf: &mut Buffer,
    ) {
        let chars: Vec<char> = text.chars().collect();
        let mut current_x = params.start_x;

        for (char_idx, ch) in chars.iter().enumerate() {
            if current_x >= params.max_x {
                break;
            }

            let char_style = if is_matched_field && match_indices.contains(&char_idx) {
                // Highlight matched characters
                params
                    .base_style
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                // Normal text color
                params.base_style.fg(Color::White)
            };

            if params.y < buf.area().bottom() {
                buf[(current_x, params.y)]
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
