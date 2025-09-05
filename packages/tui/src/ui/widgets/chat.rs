use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use crate::chat::{ChatMessage, MessageAuthor};
use crate::input::{InputBuffer, InputMode};

/// Chat widget for displaying message history
pub struct ChatWidget<'a> {
    messages: &'a [ChatMessage],
    scroll_offset: usize,
    show_timestamps: bool,
    focused: bool,
}

impl<'a> ChatWidget<'a> {
    /// Create a new chat widget
    pub fn new(messages: &'a [ChatMessage]) -> Self {
        Self {
            messages,
            scroll_offset: 0,
            show_timestamps: false,
            focused: false,
        }
    }
    
    /// Set the scroll offset for the chat
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }
    
    /// Enable or disable timestamp display
    pub fn show_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }
    
    /// Set whether this widget is focused
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
    
    /// Format a message for display
    fn format_message<'b>(&self, message: &'b ChatMessage) -> Vec<Line<'b>> {
        let mut lines = Vec::new();
        
        // Create author line with styling
        let author_style = match message.author {
            MessageAuthor::User => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            MessageAuthor::System => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            MessageAuthor::Assistant => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        };
        
        let mut author_line = vec![
            Span::styled(message.author_label(), author_style),
        ];
        
        if message.edited {
            author_line.push(Span::styled(" (edited)", Style::default().fg(Color::Gray)));
        }
        
        if self.show_timestamps {
            let timestamp = message.timestamp.format("%H:%M:%S");
            author_line.push(Span::styled(
                format!(" [{}]", timestamp),
                Style::default().fg(Color::Gray)
            ));
        }
        
        lines.push(Line::from(author_line));
        
        // Add message content with proper wrapping
        let content_lines = message.content.lines();
        for content_line in content_lines {
            if content_line.trim().is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(Span::raw(content_line)));
            }
        }
        
        // Add spacing between messages
        lines.push(Line::from(""));
        
        lines
    }
}

impl<'a> Widget for ChatWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (title, border_color) = if self.focused {
            ("Messages (↑/↓ to scroll, Tab to switch)", Color::Yellow)
        } else {
            ("Messages (Tab to focus)", Color::Gray)
        };
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(border_color));
        
        let inner = block.inner(area);
        
        // Collect all formatted message lines
        let mut all_lines = Vec::new();
        for message in self.messages.iter().rev() {
            let mut message_lines = self.format_message(message);
            message_lines.reverse(); // Since we're iterating in reverse
            all_lines.extend(message_lines);
        }
        all_lines.reverse(); // Get back to chronological order
        
        // Apply scroll offset
        let visible_lines: Vec<_> = all_lines
            .into_iter()
            .skip(self.scroll_offset)
            .collect();
        
        // Create paragraph with the lines
        let paragraph = Paragraph::new(visible_lines)
            .wrap(Wrap { trim: false })
            .scroll((0, 0));
        
        // Render the block first
        block.render(area, buf);
        
        // Then render the content
        paragraph.render(inner, buf);
    }
}

/// Enhanced input widget that works with InputBuffer
pub struct InputWidget<'a> {
    input_buffer: &'a InputBuffer,
    input_mode: &'a InputMode,
    history_position: Option<(usize, usize)>,
    placeholder: &'a str,
    focused: bool,
}

impl<'a> InputWidget<'a> {
    pub fn new(input_buffer: &'a InputBuffer, input_mode: &'a InputMode) -> Self {
        Self {
            input_buffer,
            input_mode,
            history_position: None,
            placeholder: "Type a message...",
            focused: false,
        }
    }
    
    pub fn history_position(mut self, position: Option<(usize, usize)>) -> Self {
        self.history_position = position;
        self
    }
    
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
    
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl<'a> Widget for InputWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create title based on input mode and focus state  
        let title = match (self.input_mode, self.history_position, self.focused) {
            (InputMode::History, Some((current, total)), _) => {
                format!("Input [History {}/{}]", current, total)
            }
            (InputMode::Command, None, _) => "Input [Command Mode]".to_string(),
            (InputMode::Search, None, _) => "Input [Search Mode]".to_string(),
            (InputMode::Edit, None, _) => "Input [Editing Message - Enter to save, Esc to cancel]".to_string(),
            (InputMode::Normal, None, _) => "".to_string(), // No title for normal mode
            (_, _, _) => "".to_string(), // No title by default
        };
        
        let border_color = match (self.input_mode, self.focused) {
            (InputMode::History, _) => Color::Yellow,
            (InputMode::Command, _) => Color::Green,
            (InputMode::Search, _) => Color::Blue,
            (InputMode::Edit, _) => Color::Magenta,
            (InputMode::Form, _) => Color::Cyan,
            (InputMode::Normal, true) => Color::White,
            (InputMode::Normal, false) => Color::Gray,
        };
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(border_color));
        
        let inner = block.inner(area);
        
        // Determine what to display
        let (display_text, text_style, should_show_cursor) = if self.input_buffer.is_empty() {
            (self.placeholder, Style::default().fg(Color::Gray), true)
        } else {
            (self.input_buffer.content(), Style::default().fg(Color::White), true)
        };
        
        let paragraph = Paragraph::new(display_text)
            .style(text_style)
            .wrap(Wrap { trim: false });
        
        // Render the block and content
        block.render(area, buf);
        paragraph.render(inner, buf);
        
        // Render cursor if appropriate
        if should_show_cursor {
            self.render_cursor(inner, buf);
        }
        
        // Show help text in bottom right when focused
        if self.focused && matches!(self.input_mode, InputMode::Normal) {
            let help_text = "Enter to send | Shift+Enter for newline | Ctrl+D to quit";
            let help_x = area.x + area.width.saturating_sub(help_text.len() as u16 + 1);
            let help_y = area.y + area.height - 1;
            
            if help_x > area.x && help_y >= area.y {
                for (i, ch) in help_text.chars().enumerate() {
                    let x = help_x + i as u16;
                    if x < area.x + area.width {
                        buf[(x, help_y)].set_char(ch).set_style(Style::default().fg(border_color));
                    }
                }
            }
        }
    }
}

impl<'a> InputWidget<'a> {
    /// Render the cursor at the correct position (supports multiline)
    fn render_cursor(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        
        // Get multiline cursor position
        let cursor_line = self.input_buffer.cursor_line(area.width);
        let cursor_col = self.input_buffer.cursor_column_in_line(area.width);
        
        // Calculate actual screen position
        let cursor_x = area.x + cursor_col.min(area.width.saturating_sub(1));
        let cursor_y = area.y + (cursor_line as u16).min(area.height.saturating_sub(1));
        
        // Only render cursor if it's within the display area
        if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
            let cell = &mut buf[(cursor_x, cursor_y)];
            
            // Always use reversed style for cursor to preserve the underlying character
            // This way we don't overwrite text with a block character
            cell.set_style(
                cell.style().add_modifier(Modifier::REVERSED)
            );
        }
    }
}