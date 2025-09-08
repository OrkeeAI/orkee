/// Manages input history for up/down arrow navigation
#[derive(Debug, Clone)]
pub struct InputHistory {
    /// Store of previous input strings
    history: Vec<String>,
    /// Current position in history (None = not navigating)
    current_index: Option<usize>,
    /// Temporarily store the current input when starting history navigation
    temp_buffer: Option<String>,
    /// Maximum number of history entries to keep
    max_entries: usize,
}

impl Default for InputHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHistory {
    /// Create a new input history with default capacity
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// Create a new input history with specified capacity
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            history: Vec::new(),
            current_index: None,
            temp_buffer: None,
            max_entries,
        }
    }

    /// Add a new entry to history (called when user submits input)
    pub fn add(&mut self, entry: String) {
        // Don't add empty strings or duplicates of the last entry
        if entry.is_empty() || self.history.last() == Some(&entry) {
            return;
        }

        self.history.push(entry);

        // Maintain capacity limit
        if self.history.len() > self.max_entries {
            self.history.remove(0);
        }

        // Reset navigation state
        self.current_index = None;
        self.temp_buffer = None;
    }

    /// Start navigating history (called on first Up arrow)
    /// Returns the most recent entry if available
    pub fn start_navigation(&mut self, current_input: String) -> Option<&String> {
        if self.history.is_empty() {
            return None;
        }

        // Store current input for potential restoration
        self.temp_buffer = Some(current_input);

        // Start at the most recent entry
        self.current_index = Some(self.history.len() - 1);

        self.history.last()
    }

    /// Navigate to previous entry (older, Up arrow)
    pub fn navigate_previous(&mut self) -> Option<&String> {
        match self.current_index {
            Some(index) if index > 0 => {
                self.current_index = Some(index - 1);
                self.history.get(index - 1)
            }
            Some(0) => {
                // Already at oldest entry
                self.history.first()
            }
            Some(_) => {
                // Any other index values (shouldn't happen, but handle safely)
                None
            }
            None => {
                // Not currently navigating, this shouldn't happen
                None
            }
        }
    }

    /// Navigate to next entry (newer, Down arrow)
    pub fn navigate_next(&mut self) -> Option<&String> {
        match self.current_index {
            Some(index) if index < self.history.len().saturating_sub(1) => {
                self.current_index = Some(index + 1);
                self.history.get(index + 1)
            }
            Some(index) if index == self.history.len().saturating_sub(1) => {
                // At newest entry, return to temp buffer
                self.current_index = None;
                None // Caller should restore temp buffer
            }
            Some(_) => {
                // Any other index values (edge cases)
                None
            }
            None => {
                // Not currently navigating
                None
            }
        }
    }

    /// Stop navigating and return the temporarily stored buffer
    pub fn stop_navigation(&mut self) -> Option<String> {
        self.current_index = None;
        self.temp_buffer.take()
    }

    /// Get the currently stored temporary buffer without stopping navigation
    pub fn get_temp_buffer(&self) -> Option<&String> {
        self.temp_buffer.as_ref()
    }

    /// Check if currently navigating history
    pub fn is_navigating(&self) -> bool {
        self.current_index.is_some()
    }

    /// Get current history position (for UI display)
    pub fn current_position(&self) -> Option<(usize, usize)> {
        self.current_index.map(|idx| (idx + 1, self.history.len()))
    }

    /// Get all history entries (for debugging/display)
    pub fn entries(&self) -> &[String] {
        &self.history
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.history.clear();
        self.current_index = None;
        self.temp_buffer = None;
    }

    /// Get the number of entries in history
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_entries() {
        let mut history = InputHistory::new();

        history.add("first command".to_string());
        history.add("second command".to_string());
        history.add("third command".to_string());

        assert_eq!(history.len(), 3);
        assert_eq!(history.entries()[0], "first command");
        assert_eq!(history.entries()[2], "third command");
    }

    #[test]
    fn test_no_duplicates() {
        let mut history = InputHistory::new();

        history.add("same command".to_string());
        history.add("same command".to_string());

        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_navigation() {
        let mut history = InputHistory::new();

        history.add("first".to_string());
        history.add("second".to_string());
        history.add("third".to_string());

        // Start navigation with current input
        let entry = history.start_navigation("current input".to_string());
        assert_eq!(entry, Some(&"third".to_string()));
        assert!(history.is_navigating());

        // Go back
        let entry = history.navigate_previous();
        assert_eq!(entry, Some(&"second".to_string()));

        let entry = history.navigate_previous();
        assert_eq!(entry, Some(&"first".to_string()));

        // Go forward
        let entry = history.navigate_next();
        assert_eq!(entry, Some(&"second".to_string()));

        let entry = history.navigate_next();
        assert_eq!(entry, Some(&"third".to_string()));

        // Go past newest (should stop navigation)
        let entry = history.navigate_next();
        assert_eq!(entry, None);
        assert!(!history.is_navigating());

        // Should be able to restore temp buffer
        let temp = history.stop_navigation();
        assert_eq!(temp, Some("current input".to_string()));
    }

    #[test]
    fn test_capacity_limit() {
        let mut history = InputHistory::with_capacity(2);

        history.add("first".to_string());
        history.add("second".to_string());
        history.add("third".to_string());

        assert_eq!(history.len(), 2);
        assert_eq!(history.entries()[0], "second");
        assert_eq!(history.entries()[1], "third");
    }

    #[test]
    fn test_empty_history_navigation() {
        let mut history = InputHistory::new();

        let entry = history.start_navigation("test".to_string());
        assert_eq!(entry, None);
        assert!(!history.is_navigating());
    }
}
