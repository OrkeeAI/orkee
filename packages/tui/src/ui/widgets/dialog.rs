use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Result of user interaction with the confirmation dialog
#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    /// User confirmed the action
    Confirmed,
    /// User cancelled the action
    Cancelled,
    /// Dialog is still waiting for user input
    Pending,
}

/// Represents which button is currently focused
#[derive(Debug, Clone, PartialEq)]
pub enum DialogFocus {
    /// Cancel button is focused (default for safety)
    Cancel,
    /// Confirm/Delete button is focused
    Confirm,
}

/// Configuration for a confirmation dialog
#[derive(Debug, Clone)]
pub struct ConfirmationDialog {
    /// Main title of the dialog
    pub title: String,
    /// Main message to display
    pub message: String,
    /// Text for the confirmation button (e.g., "Delete", "Confirm")
    pub confirm_text: String,
    /// Text for the cancel button (e.g., "Cancel")
    pub cancel_text: String,
    /// Whether this is a dangerous action (affects styling)
    pub dangerous: bool,
    /// Which button is currently focused
    pub focus: DialogFocus,
    /// Additional details to show (optional)
    pub details: Option<String>,
}

impl ConfirmationDialog {
    /// Create a new confirmation dialog with safe defaults
    pub fn new(title: String, message: String) -> Self {
        Self {
            title,
            message,
            confirm_text: "Confirm".to_string(),
            cancel_text: "Cancel".to_string(),
            dangerous: false,
            focus: DialogFocus::Cancel, // Start with Cancel focused for safety
            details: None,
        }
    }

    /// Mark this dialog as dangerous (red styling for confirm button)
    pub fn dangerous(mut self) -> Self {
        self.dangerous = true;
        self
    }

    /// Set custom button text
    pub fn with_buttons(mut self, confirm_text: String, cancel_text: String) -> Self {
        self.confirm_text = confirm_text;
        self.cancel_text = cancel_text;
        self
    }

    /// Add additional details to display
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// Switch focus to the next button
    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            DialogFocus::Cancel => DialogFocus::Confirm,
            DialogFocus::Confirm => DialogFocus::Cancel,
        };
    }

    /// Handle key input and return the result
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> DialogResult {
        match key {
            crossterm::event::KeyCode::Tab => {
                self.next_focus();
                DialogResult::Pending
            }
            crossterm::event::KeyCode::BackTab => {
                self.next_focus(); // Same as Tab since we only have 2 buttons
                DialogResult::Pending
            }
            crossterm::event::KeyCode::Enter | crossterm::event::KeyCode::Char(' ') => {
                match self.focus {
                    DialogFocus::Cancel => DialogResult::Cancelled,
                    DialogFocus::Confirm => DialogResult::Confirmed,
                }
            }
            crossterm::event::KeyCode::Esc => DialogResult::Cancelled,
            _ => DialogResult::Pending,
        }
    }
}

/// Widget for rendering a confirmation dialog
pub struct ConfirmationDialogWidget<'a> {
    dialog: &'a ConfirmationDialog,
}

impl<'a> ConfirmationDialogWidget<'a> {
    /// Create a new confirmation dialog widget
    pub fn new(dialog: &'a ConfirmationDialog) -> Self {
        Self { dialog }
    }
}

impl<'a> Widget for ConfirmationDialogWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the background
        Clear.render(area, buf);

        // Calculate dialog size - aim for about 1/3 of screen width, auto height
        let dialog_width = (area.width / 3).max(50).min(area.width - 4);
        let min_height = 12; // Minimum height for the dialog

        // Calculate content height based on message and details
        let mut content_height = 4; // Title + borders + buttons
        content_height += (self.dialog.message.len() as u16 / (dialog_width - 4)) + 2; // Message
        if self.dialog.details.is_some() {
            content_height += 3; // Details section
        }
        let dialog_height = content_height.max(min_height).min(area.height - 2);

        // Center the dialog
        let dialog_area = Rect {
            x: (area.width.saturating_sub(dialog_width)) / 2,
            y: (area.height.saturating_sub(dialog_height)) / 2,
            width: dialog_width,
            height: dialog_height,
        };

        // Render main dialog block
        let title_style = if self.dialog.dangerous {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        };

        let block = Block::default()
            .title(self.dialog.title.clone())
            .title_style(title_style)
            .borders(Borders::ALL)
            .border_style(if self.dialog.dangerous {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Yellow)
            });

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        // Split the inner area for content and buttons
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Message content
                Constraint::Length(3), // Button area
            ])
            .split(inner);

        // Render message content
        let mut content_text = vec![
            Line::from(vec![
                Span::styled("⚠️  ", Style::default().fg(Color::Yellow)),
                Span::raw(&self.dialog.message),
            ]),
            Line::raw(""),
        ];

        // Add details if present
        if let Some(details) = &self.dialog.details {
            // Split details by newlines to create separate lines
            for detail_line in details.split('\n') {
                content_text.push(Line::raw(detail_line));
            }
            content_text.push(Line::raw(""));
        }

        // Add warning for dangerous actions
        if self.dialog.dangerous {
            content_text.push(Line::from(vec![Span::styled(
                "⚠️  This action cannot be undone!",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]));
            content_text.push(Line::raw(""));
        }

        let content = Paragraph::new(Text::from(content_text))
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: true });

        content.render(chunks[0], buf);

        // Render buttons
        let button_area = chunks[1];

        // Calculate button positions
        let total_button_width = self.dialog.cancel_text.len() + self.dialog.confirm_text.len() + 7; // buttons + spacing + brackets
        let start_x = (button_area.width.saturating_sub(total_button_width as u16)) / 2;

        let cancel_width = self.dialog.cancel_text.len() as u16 + 2; // text + brackets
        let confirm_width = self.dialog.confirm_text.len() as u16 + 2;
        let spacing = 3;

        // Render Cancel button
        let cancel_x = start_x;
        let cancel_style = if matches!(self.dialog.focus, DialogFocus::Cancel) {
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let cancel_area = Rect {
            x: button_area.x + cancel_x,
            y: button_area.y + 1,
            width: cancel_width,
            height: 1,
        };

        let cancel_text = format!("[{}]", self.dialog.cancel_text);
        let cancel_para = Paragraph::new(cancel_text).style(cancel_style);
        cancel_para.render(cancel_area, buf);

        // Render Confirm button
        let confirm_x = cancel_x + cancel_width + spacing;
        let confirm_style = if matches!(self.dialog.focus, DialogFocus::Confirm) {
            if self.dialog.dangerous {
                Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .bg(Color::Green)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            if self.dialog.dangerous {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            }
        };

        let confirm_area = Rect {
            x: button_area.x + confirm_x,
            y: button_area.y + 1,
            width: confirm_width,
            height: 1,
        };

        let confirm_text = format!("[{}]", self.dialog.confirm_text);
        let confirm_para = Paragraph::new(confirm_text).style(confirm_style);
        confirm_para.render(confirm_area, buf);

        // Render keyboard shortcuts
        let shortcuts = "Tab: Switch • Enter: Confirm • Esc: Cancel";
        let shortcuts_para = Paragraph::new(shortcuts)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);

        let shortcuts_area = Rect {
            x: button_area.x,
            y: button_area.y + 2,
            width: button_area.width,
            height: 1,
        };

        shortcuts_para.render(shortcuts_area, buf);
    }
}
