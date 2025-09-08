use crate::mention_popup::MentionPopup;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};

/// Widget for rendering the mention popup with fuzzy matching highlights
pub struct MentionPopupWidget<'a> {
    popup: &'a MentionPopup,
    max_rows: u16,
}

impl<'a> MentionPopupWidget<'a> {
    /// Create a new mention popup widget
    pub fn new(popup: &'a MentionPopup) -> Self {
        Self {
            popup,
            max_rows: 6, // Default max rows
        }
    }

    /// Set the maximum number of rows to display
    pub fn max_rows(mut self, max_rows: u16) -> Self {
        self.max_rows = max_rows;
        self
    }
}

impl<'a> Widget for MentionPopupWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Don't render if no results
        if !self.popup.has_results() {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Mentions ({} matches)", self.popup.result_count()))
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Color::Magenta));

        let inner = block.inner(area);
        block.render(area, buf);

        // Render filtered mentions with highlighting
        let matches = self.popup.filtered_matches();
        let display_count = (matches.len() as u16).min(self.max_rows).min(inner.height);

        for (row_idx, mention_match) in matches.iter().enumerate().take(display_count as usize) {
            let y = inner.y + row_idx as u16;

            // Determine if this row is selected
            let is_selected = row_idx == self.popup.selected_index();

            // Background style for selected item
            let bg_style = if is_selected {
                Style::default().bg(Color::Magenta)
            } else {
                Style::default()
            };

            // Clear the line with background color
            for x in inner.x..inner.x + inner.width {
                if x < buf.area().right() && y < buf.area().bottom() {
                    buf[(x, y)].set_style(bg_style);
                }
            }

            let mut x = inner.x;

            // Render icon for the item type (@ for projects, # for files, etc.)
            let icon = match mention_match.item.target_type {
                crate::mention_popup::MentionTarget::Projects => "@",
                crate::mention_popup::MentionTarget::Files => "#",
            };

            // Render the icon
            if x < inner.x + inner.width {
                buf[(x, y)]
                    .set_char(icon.chars().next().unwrap_or('@'))
                    .set_style(bg_style.fg(Color::Blue).add_modifier(Modifier::BOLD));
                x += 1;
            }

            // Add space after icon
            if x < inner.x + inner.width {
                buf[(x, y)].set_char(' ').set_style(bg_style);
                x += 1;
            }

            // Render the item name with fuzzy match highlighting
            let name = &mention_match.item.name;
            let name_chars: Vec<char> = name.chars().collect();

            for (char_idx, ch) in name_chars.iter().enumerate() {
                if x >= inner.x + inner.width {
                    break;
                }

                let char_style = if mention_match.match_indices.contains(&char_idx) {
                    // Highlighted match character
                    bg_style.fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    // Normal item name
                    bg_style.fg(Color::Green).add_modifier(Modifier::BOLD)
                };

                buf[(x, y)].set_char(*ch).set_style(char_style);
                x += 1;
            }

            // Add some spacing between name and path
            if x < inner.x + inner.width {
                buf[(x, y)].set_char(' ').set_style(bg_style);
                x += 1;
            }
            if x < inner.x + inner.width {
                buf[(x, y)]
                    .set_char('-')
                    .set_style(bg_style.fg(Color::Gray));
                x += 1;
            }
            if x < inner.x + inner.width {
                buf[(x, y)].set_char(' ').set_style(bg_style);
                x += 1;
            }

            // Render the path (truncated if needed)
            let path = &mention_match.item.path;
            let available_width = (inner.x + inner.width).saturating_sub(x);
            let truncated_path = if path.len() > available_width as usize {
                format!(
                    "...{}",
                    &path[path
                        .len()
                        .saturating_sub(available_width.saturating_sub(3) as usize)..]
                )
            } else {
                path.clone()
            };

            for ch in truncated_path.chars() {
                if x >= inner.x + inner.width {
                    break;
                }

                buf[(x, y)].set_char(ch).set_style(bg_style.fg(Color::Gray));
                x += 1;
            }

            // If there's a description and space remaining, show it
            if let Some(description) = &mention_match.item.description {
                let available_width = (inner.x + inner.width).saturating_sub(x);
                if available_width > 4 {
                    // Need at least space for " (x)"
                    // Add opening parenthesis
                    if x < inner.x + inner.width {
                        buf[(x, y)].set_char(' ').set_style(bg_style);
                        x += 1;
                    }
                    if x < inner.x + inner.width {
                        buf[(x, y)]
                            .set_char('(')
                            .set_style(bg_style.fg(Color::Gray));
                        x += 1;
                    }

                    // Render description (truncated)
                    let available_for_desc = available_width.saturating_sub(3); // Account for " ()"
                    let truncated_desc = if description.len() > available_for_desc as usize {
                        format!(
                            "{}...",
                            &description[..available_for_desc.saturating_sub(3) as usize]
                        )
                    } else {
                        description.clone()
                    };

                    for ch in truncated_desc.chars() {
                        if x >= inner.x + inner.width - 1 {
                            // Leave space for closing paren
                            break;
                        }
                        buf[(x, y)].set_char(ch).set_style(bg_style.fg(Color::Gray));
                        x += 1;
                    }

                    // Add closing parenthesis
                    if x < inner.x + inner.width {
                        buf[(x, y)]
                            .set_char(')')
                            .set_style(bg_style.fg(Color::Gray));
                    }
                }
            }
        }
    }
}

/// Calculate the area for the mention popup above the input area
pub fn calculate_mention_popup_area(terminal_area: Rect, mention_popup: &MentionPopup) -> Rect {
    let popup_height = (mention_popup.result_count() as u16 + 2).min(8); // +2 for borders, max 8 rows
    let popup_width = terminal_area.width.min(80); // Max 80 columns

    // Position above the input area (which is typically at the bottom)
    let popup_y = terminal_area.height.saturating_sub(4 + popup_height); // 4 for input area height
    let popup_x = (terminal_area.width.saturating_sub(popup_width)) / 2; // Center horizontally

    Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mention_popup::{MentionItem, MentionPopup, MentionTarget};

    fn create_test_item(name: &str, path: &str) -> MentionItem {
        MentionItem {
            id: name.to_string(),
            name: name.to_string(),
            path: path.to_string(),
            description: None,
            target_type: MentionTarget::Projects,
        }
    }

    #[test]
    fn test_widget_creation() {
        let items = vec![create_test_item("test", "/path")];
        let popup = MentionPopup::new(items, 0);
        let widget = MentionPopupWidget::new(&popup);

        // Should create without panicking
        assert!(widget.max_rows > 0);
    }

    #[test]
    fn test_max_rows_setting() {
        let items = vec![create_test_item("test", "/path")];
        let popup = MentionPopup::new(items, 0);
        let widget = MentionPopupWidget::new(&popup).max_rows(10);

        assert_eq!(widget.max_rows, 10);
    }

    #[test]
    fn test_popup_area_calculation() {
        let terminal_area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 30,
        };

        let items = vec![
            create_test_item("test1", "/path1"),
            create_test_item("test2", "/path2"),
        ];
        let popup = MentionPopup::new(items, 0);

        let popup_area = calculate_mention_popup_area(terminal_area, &popup);

        // Should be positioned above input area and centered
        assert!(popup_area.height >= 4); // At least 2 items + 2 borders
        assert!(popup_area.width <= 80);
        assert!(popup_area.y < terminal_area.height - 4);
    }
}
