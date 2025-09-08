use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Text input buffer with cursor management and editing operations
#[derive(Debug, Clone)]
pub struct InputBuffer {
    /// The actual text content
    content: String,
    /// Cursor position as byte index in the content string
    cursor_position: usize,
    /// Preferred column for vertical navigation (used in multi-line scenarios)
    preferred_column: Option<u16>,
    /// Cache for display lines to avoid recalculation
    lines_cache: Option<Vec<String>>,
    /// Dirty flag to know when to recalculate cache
    cache_dirty: bool,
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl InputBuffer {
    /// Create a new empty input buffer
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            preferred_column: None,
            lines_cache: None,
            cache_dirty: true,
        }
    }

    /// Get the current text content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get the current cursor position (byte index)
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Get the length of the content in characters (not bytes)
    pub fn len(&self) -> usize {
        self.content.graphemes(true).count()
    }

    /// Clear all content and reset cursor
    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_position = 0;
        self.preferred_column = None;
        self.invalidate_cache();
    }

    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, ch: char) {
        self.content.insert(self.cursor_position, ch);
        self.cursor_position += ch.len_utf8();
        self.preferred_column = None;
        self.invalidate_cache();
    }

    /// Insert a string at the current cursor position
    pub fn insert_str(&mut self, s: &str) {
        self.content.insert_str(self.cursor_position, s);
        self.cursor_position += s.len();
        self.preferred_column = None;
        self.invalidate_cache();
    }

    /// Delete the character at the cursor position (Delete key behavior)
    pub fn delete_char(&mut self) -> bool {
        if self.cursor_position >= self.content.len() {
            return false;
        }

        // Find the next grapheme boundary
        let mut indices = self.content.grapheme_indices(true);
        if let Some((start, _)) = indices.find(|(idx, _)| *idx >= self.cursor_position) {
            if let Some((end, _)) = indices.next() {
                self.content.drain(start..end);
            } else {
                self.content.drain(start..);
            }
            self.invalidate_cache();
            true
        } else {
            false
        }
    }

    /// Delete the character before the cursor position (Backspace key behavior)
    pub fn backspace(&mut self) -> bool {
        if self.cursor_position == 0 {
            return false;
        }

        // Find the previous grapheme boundary
        let mut indices: Vec<_> = self
            .content
            .grapheme_indices(true)
            .take_while(|(idx, _)| *idx < self.cursor_position)
            .collect();

        if let Some((start, grapheme)) = indices.pop() {
            self.content.drain(start..start + grapheme.len());
            self.cursor_position = start;
            self.preferred_column = None;
            self.invalidate_cache();
            true
        } else {
            false
        }
    }

    /// Move cursor left by one grapheme
    pub fn move_left(&mut self) -> bool {
        if self.cursor_position == 0 {
            return false;
        }

        // Find the previous grapheme boundary
        let mut last_pos = 0;
        for (pos, _) in self.content.grapheme_indices(true) {
            if pos >= self.cursor_position {
                break;
            }
            last_pos = pos;
        }

        self.cursor_position = last_pos;
        self.preferred_column = None;
        true
    }

    /// Move cursor right by one grapheme
    pub fn move_right(&mut self) -> bool {
        if self.cursor_position >= self.content.len() {
            return false;
        }

        // Find the next grapheme boundary
        for (pos, grapheme) in self.content.grapheme_indices(true) {
            if pos >= self.cursor_position {
                self.cursor_position = pos + grapheme.len();
                self.preferred_column = None;
                return true;
            }
        }

        false
    }

    /// Move cursor to the beginning of the buffer
    pub fn move_to_start(&mut self) {
        self.cursor_position = 0;
        self.preferred_column = None;
    }

    /// Move cursor to the end of the buffer
    pub fn move_to_end(&mut self) {
        self.cursor_position = self.content.len();
        self.preferred_column = None;
    }

    /// Set cursor position to a specific byte position (clamped to valid range)
    pub fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position = position.min(self.content.len());
        self.preferred_column = None;
    }

    /// Move cursor to the beginning of the previous word
    pub fn move_word_left(&mut self) -> bool {
        if self.cursor_position == 0 {
            return false;
        }

        let text_before = &self.content[..self.cursor_position];
        let chars: Vec<char> = text_before.chars().collect();

        if chars.is_empty() {
            return false;
        }

        let mut pos = chars.len();

        // Skip trailing whitespace
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Skip the current word
        while pos > 0 && !chars[pos - 1].is_whitespace() && !chars[pos - 1].is_ascii_punctuation() {
            pos -= 1;
        }

        // Convert char position back to byte position
        let new_cursor_pos = if pos == 0 {
            0
        } else {
            chars[..pos].iter().collect::<String>().len()
        };

        self.cursor_position = new_cursor_pos;
        self.preferred_column = None;
        true
    }

    /// Move cursor to the beginning of the next word
    pub fn move_word_right(&mut self) -> bool {
        if self.cursor_position >= self.content.len() {
            return false;
        }

        let chars: Vec<char> = self.content.chars().collect();
        let mut pos = self.content[..self.cursor_position].chars().count();

        if pos >= chars.len() {
            return false;
        }

        // Skip current word (non-whitespace, non-punctuation)
        while pos < chars.len() && !chars[pos].is_whitespace() && !chars[pos].is_ascii_punctuation()
        {
            pos += 1;
        }

        // Skip whitespace and punctuation
        while pos < chars.len() && (chars[pos].is_whitespace() || chars[pos].is_ascii_punctuation())
        {
            pos += 1;
        }

        // Convert char position back to byte position
        let new_cursor_pos = if pos >= chars.len() {
            self.content.len()
        } else {
            chars[..pos].iter().collect::<String>().len()
        };

        self.cursor_position = new_cursor_pos;
        self.preferred_column = None;
        true
    }

    /// Get the cursor position in terms of display column (accounting for character width)
    pub fn cursor_display_column(&self) -> u16 {
        let text_before_cursor = &self.content[..self.cursor_position];
        text_before_cursor.width() as u16
    }

    /// Get display lines for rendering (handles wrapping)
    pub fn get_display_lines(&mut self, max_width: u16) -> &[String] {
        if self.cache_dirty || self.lines_cache.is_none() {
            self.recalculate_lines(max_width);
        }

        // Safe to use unwrap_or_default here since we just recalculated if needed
        self.lines_cache.as_deref()
            .unwrap_or(&[])
    }

    /// Find which line the cursor is on (0-indexed)
    pub fn cursor_line(&self, _max_width: u16) -> usize {
        // Simple approach: count newlines before cursor position
        let text_before_cursor = &self.content[..self.cursor_position];
        text_before_cursor.chars().filter(|&c| c == '\n').count()
    }

    /// Get the cursor position within its current line
    pub fn cursor_column_in_line(&self, _max_width: u16) -> u16 {
        // Simple approach: find the last newline before cursor, measure width after it
        let text_before_cursor = &self.content[..self.cursor_position];

        if let Some(last_newline_pos) = text_before_cursor.rfind('\n') {
            // Get text after the last newline
            let line_text = &text_before_cursor[last_newline_pos + 1..];
            line_text.width() as u16
        } else {
            // No newline before cursor, so we're on the first line
            text_before_cursor.width() as u16
        }
    }

    /// Private method to invalidate the lines cache
    fn invalidate_cache(&mut self) {
        self.cache_dirty = true;
    }

    /// Private method to recalculate display lines
    fn recalculate_lines(&mut self, max_width: u16) {
        let mut lines = Vec::new();

        if self.content.is_empty() {
            lines.push(String::new());
        } else {
            // Split on newlines, but preserve empty lines (unlike .lines() which drops them)
            let content_lines: Vec<&str> = if self.content.ends_with('\n') {
                // If content ends with newline, we need to add an empty line at the end
                let mut split_lines: Vec<&str> = self.content.split('\n').collect();
                split_lines.pop(); // Remove the last empty element that split creates
                split_lines.push(""); // Add back an empty line for the trailing newline
                split_lines
            } else {
                self.content.split('\n').collect()
            };

            for line in content_lines {
                if line.width() <= max_width as usize {
                    lines.push(line.to_string());
                } else {
                    // Basic word wrapping
                    let mut current_line = String::new();
                    for word in line.split_whitespace() {
                        if current_line.is_empty() {
                            current_line = word.to_string();
                        } else if (current_line.width() + 1 + word.width()) <= max_width as usize {
                            current_line.push(' ');
                            current_line.push_str(word);
                        } else {
                            lines.push(current_line);
                            current_line = word.to_string();
                        }
                    }
                    if !current_line.is_empty() {
                        lines.push(current_line);
                    }
                }
            }
        }

        self.lines_cache = Some(lines);
        self.cache_dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = InputBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.cursor_position(), 0);
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = InputBuffer::new();
        buffer.insert_char('H');
        buffer.insert_char('i');

        assert_eq!(buffer.content(), "Hi");
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.cursor_position(), 2);
    }

    #[test]
    fn test_backspace() {
        let mut buffer = InputBuffer::new();
        buffer.insert_str("Hello");

        assert!(buffer.backspace());
        assert_eq!(buffer.content(), "Hell");
        assert_eq!(buffer.cursor_position(), 4);

        // Test at beginning
        buffer.move_to_start();
        assert!(!buffer.backspace());
    }

    #[test]
    fn test_cursor_movement() {
        let mut buffer = InputBuffer::new();
        buffer.insert_str("Hello");

        assert!(buffer.move_left());
        assert_eq!(buffer.cursor_position(), 4);

        buffer.move_to_start();
        assert_eq!(buffer.cursor_position(), 0);
        assert!(!buffer.move_left());

        buffer.move_to_end();
        assert_eq!(buffer.cursor_position(), 5);
        assert!(!buffer.move_right());
    }

    #[test]
    fn test_unicode_handling() {
        let mut buffer = InputBuffer::new();
        buffer.insert_str("🦀rust");

        // The crab emoji is 4 bytes but 1 grapheme
        assert_eq!(buffer.len(), 5); // 1 crab + 4 letters
        assert_eq!(buffer.cursor_position(), 8); // 4 bytes for crab + 4 for rust

        buffer.move_left(); // Should move before 't'
        buffer.insert_char('!');
        assert_eq!(buffer.content(), "🦀rus!t");
    }

    #[test]
    fn test_word_movement() {
        let mut buffer = InputBuffer::new();
        buffer.insert_str("hello world test");

        buffer.move_word_left();
        assert_eq!(buffer.cursor_position(), 12); // Before "test"

        buffer.move_word_left();
        assert_eq!(buffer.cursor_position(), 6); // Before "world"

        buffer.move_word_right();
        assert_eq!(buffer.cursor_position(), 12); // Before "test"
    }

    #[test]
    fn test_multiline_input() {
        let mut buffer = InputBuffer::new();

        // Test basic newline insertion
        buffer.insert_str("Line 1");
        buffer.insert_char('\n');
        buffer.insert_str("Line 2");

        // Content should be "Line 1\nLine 2" with cursor at end
        assert_eq!(buffer.content(), "Line 1\nLine 2");

        // Move to different positions and test cursor positioning
        buffer.move_to_start();
        assert_eq!(buffer.cursor_line(80), 0);
        assert_eq!(buffer.cursor_column_in_line(80), 0);

        // Test that we can insert newlines with Shift+Enter-like functionality
        buffer.move_to_end();
        buffer.insert_char('\n');
        buffer.insert_str("Line 3");
        assert_eq!(buffer.content(), "Line 1\nLine 2\nLine 3");

        // Cursor should now be at end of Line 3
        let mut temp_buffer = buffer.clone();
        let lines = temp_buffer.get_display_lines(80);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
        assert_eq!(lines[2], "Line 3");
    }

    #[test]
    fn test_cursor_positioning_with_newlines() {
        let mut buffer = InputBuffer::new();

        // Test with just a newline at the start
        buffer.insert_char('\n');
        assert_eq!(buffer.content(), "\n");
        assert_eq!(buffer.cursor_position(), 1); // After the newline
        assert_eq!(buffer.cursor_line(80), 1); // Second line (0-indexed)
        assert_eq!(buffer.cursor_column_in_line(80), 0); // At start of second line

        // Add text after the newline
        buffer.insert_str("hello");
        assert_eq!(buffer.content(), "\nhello");
        assert_eq!(buffer.cursor_line(80), 1); // Still second line
        assert_eq!(buffer.cursor_column_in_line(80), 5); // After "hello"

        // Add another newline
        buffer.insert_char('\n');
        assert_eq!(buffer.content(), "\nhello\n");
        assert_eq!(buffer.cursor_line(80), 2); // Third line
        assert_eq!(buffer.cursor_column_in_line(80), 0); // At start of third line

        // Test cursor positioning in the middle
        buffer.set_cursor_position(3); // After "\nhe"
        assert_eq!(buffer.cursor_line(80), 1); // Second line
        assert_eq!(buffer.cursor_column_in_line(80), 2); // After "he"
    }
}
