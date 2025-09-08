use crate::command_popup::CommandPopup;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Widget for rendering the command popup with fuzzy matching highlights
pub struct CommandPopupWidget<'a> {
    popup: &'a CommandPopup,
    max_rows: u16,
}

impl<'a> CommandPopupWidget<'a> {
    /// Create a new command popup widget
    pub fn new(popup: &'a CommandPopup) -> Self {
        Self {
            popup,
            max_rows: 8, // Default max rows
        }
    }

    /// Set the maximum number of rows to display
    pub fn max_rows(mut self, max_rows: u16) -> Self {
        self.max_rows = max_rows;
        self
    }
}

impl<'a> Widget for CommandPopupWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Don't render if no results
        if !self.popup.has_results() {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Commands ({} matches)", self.popup.result_count()))
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        // Render filtered commands with highlighting
        let matches = self.popup.filtered_matches();
        let display_count = (matches.len() as u16).min(self.max_rows).min(inner.height);

        for (row_idx, cmd_match) in matches.iter().enumerate().take(display_count as usize) {
            let y = inner.y + row_idx as u16;

            // Determine if this row is selected
            let is_selected = row_idx == self.popup.selected_index();

            // Background style for selected item
            let bg_style = if is_selected {
                Style::default().bg(Color::Blue)
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

            // Render command usage (e.g., "/help", "/project <name>")
            let usage = &cmd_match.item.usage;
            let usage_chars: Vec<char> = usage.chars().collect();

            // Highlight matching characters in the command name part
            for (char_idx, ch) in usage_chars.iter().enumerate() {
                if x >= inner.x + inner.width {
                    break;
                }

                let char_style = if cmd_match.match_indices.contains(&char_idx) {
                    // Highlighted match
                    bg_style.fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    // Normal command text
                    bg_style.fg(Color::Green).add_modifier(Modifier::BOLD)
                };

                buf[(x, y)].set_char(*ch).set_style(char_style);
                x += 1;
            }

            // Add some spacing
            if x < inner.x + inner.width {
                buf[(x, y)].set_char(' ').set_style(bg_style);
                x += 1;
            }

            // Add separator
            if x < inner.x + inner.width {
                buf[(x, y)]
                    .set_char('-')
                    .set_style(bg_style.fg(Color::DarkGray));
                x += 1;
            }

            // Add another space
            if x < inner.x + inner.width {
                buf[(x, y)].set_char(' ').set_style(bg_style);
                x += 1;
            }

            // Add description (truncated to fit)
            let description = &cmd_match.item.description;
            let remaining_width = (inner.x + inner.width).saturating_sub(x);
            let desc_chars: Vec<char> =
                description.chars().take(remaining_width as usize).collect();

            for ch in desc_chars {
                if x >= inner.x + inner.width {
                    break;
                }

                buf[(x, y)].set_char(ch).set_style(bg_style.fg(Color::Gray));
                x += 1;
            }
        }

        // If there are more items than can be displayed, show an indicator
        if matches.len() > display_count as usize {
            let more_count = matches.len() - display_count as usize;
            let indicator = format!("... and {} more", more_count);
            let indicator_y = inner.y + display_count;

            if indicator_y < inner.y + inner.height {
                let mut x = inner.x;
                for ch in indicator.chars() {
                    if x >= inner.x + inner.width {
                        break;
                    }
                    buf[(x, indicator_y)].set_char(ch).set_style(
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    );
                    x += 1;
                }
            }
        }
    }
}

/// Helper widget for rendering command hints below the input
pub struct CommandHintWidget<'a> {
    popup: &'a CommandPopup,
}

impl<'a> CommandHintWidget<'a> {
    /// Create a new command hint widget
    pub fn new(popup: &'a CommandPopup) -> Self {
        Self { popup }
    }
}

impl<'a> Widget for CommandHintWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Show navigation hints
        let hint_text = if self.popup.has_results() {
            "↑↓ Navigate • Tab/Enter Complete • Esc Cancel"
        } else {
            "No matching commands"
        };

        let paragraph = Paragraph::new(hint_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_popup::CommandPopup;

    #[test]
    fn test_widget_creation() {
        let popup = CommandPopup::new();
        let widget = CommandPopupWidget::new(&popup);

        // Should be able to create widget
        assert_eq!(widget.max_rows, 8);
    }

    #[test]
    fn test_widget_with_custom_rows() {
        let popup = CommandPopup::new();
        let widget = CommandPopupWidget::new(&popup).max_rows(5);

        assert_eq!(widget.max_rows, 5);
    }

    #[test]
    fn test_hint_widget_creation() {
        let popup = CommandPopup::new();
        let _hint_widget = CommandHintWidget::new(&popup);

        // Should create without error
    }
}
