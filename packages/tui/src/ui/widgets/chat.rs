use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use crate::chat::{ChatMessage, MessageAuthor};

/// Chat widget for displaying message history
pub struct ChatWidget<'a> {
    messages: &'a [ChatMessage],
    scroll_offset: usize,
    show_timestamps: bool,
}

impl<'a> ChatWidget<'a> {
    /// Create a new chat widget
    pub fn new(messages: &'a [ChatMessage]) -> Self {
        Self {
            messages,
            scroll_offset: 0,
            show_timestamps: false,
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
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Chat")
            .border_style(Style::default().fg(Color::White));
        
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

/// Simple input widget for chat input
pub struct InputWidget<'a> {
    input: &'a str,
    cursor_pos: usize,
    placeholder: &'a str,
}

impl<'a> InputWidget<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            cursor_pos: input.len(),
            placeholder: "Type a message...",
        }
    }
    
    pub fn cursor_pos(mut self, pos: usize) -> Self {
        self.cursor_pos = pos.min(self.input.len());
        self
    }
    
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
}

impl<'a> Widget for InputWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Input")
            .border_style(Style::default().fg(Color::White));
        
        let inner = block.inner(area);
        
        let display_text = if self.input.is_empty() {
            self.placeholder
        } else {
            self.input
        };
        
        let style = if self.input.is_empty() {
            Style::default().fg(Color::Gray)
        } else {
            Style::default()
        };
        
        let paragraph = Paragraph::new(display_text)
            .style(style)
            .wrap(Wrap { trim: false });
        
        block.render(area, buf);
        paragraph.render(inner, buf);
        
        // Show cursor if there's input
        if !self.input.is_empty() && self.cursor_pos <= self.input.len() {
            let cursor_x = inner.x + self.cursor_pos as u16;
            if cursor_x < inner.x + inner.width {
                buf[(cursor_x, inner.y)].set_style(
                    Style::default().add_modifier(Modifier::REVERSED)
                );
            }
        }
    }
}