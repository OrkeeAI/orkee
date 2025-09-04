use orkee_projects::Project;
use crate::chat::{MessageHistory, ChatMessage};
use crate::input::{InputBuffer, InputHistory, InputMode};

/// Application state management
#[derive(Debug, Clone)]
pub struct AppState {
    pub projects: Vec<Project>,
    pub selected_project: Option<usize>,
    pub current_screen: Screen,
    pub refresh_interval: u64,
    pub message_history: MessageHistory,
    pub scroll_offset: usize,
    pub input_buffer: InputBuffer,
    pub input_history: InputHistory,
    pub input_mode: InputMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    Projects,
    Settings,
    Chat,
}

impl AppState {
    pub fn new(refresh_interval: u64) -> Self {
        let mut state = Self {
            projects: Vec::new(),
            selected_project: None,
            current_screen: Screen::Chat,
            refresh_interval,
            message_history: MessageHistory::new(),
            scroll_offset: 0,
            input_buffer: InputBuffer::new(),
            input_history: InputHistory::new(),
            input_mode: InputMode::Normal,
        };
        
        // Add welcome message
        state.message_history.add_system_message("Welcome to Orkee TUI! Type a message to get started.");
        state
    }
    
    pub fn set_projects(&mut self, projects: Vec<Project>) {
        self.projects = projects;
    }
    
    pub fn next_screen(&mut self) {
        self.current_screen = match self.current_screen {
            Screen::Dashboard => Screen::Projects,
            Screen::Projects => Screen::Settings, 
            Screen::Settings => Screen::Chat,
            Screen::Chat => Screen::Dashboard,
        }
    }
    
    /// Add a user message to the chat
    pub fn add_user_message(&mut self, content: String) -> &ChatMessage {
        self.message_history.add_user_message(content)
    }
    
    /// Add a system message to the chat
    pub fn add_system_message(&mut self, content: String) -> &ChatMessage {
        self.message_history.add_system_message(content)
    }
    
    /// Add an assistant message to the chat
    pub fn add_assistant_message(&mut self, content: String) -> &ChatMessage {
        self.message_history.add_assistant_message(content)
    }
    
    /// Get all messages for display
    pub fn messages(&self) -> &[ChatMessage] {
        self.message_history.messages()
    }
    
    /// Scroll up in the message history
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }
    
    /// Scroll down in the message history
    pub fn scroll_down(&mut self) {
        let max_offset = self.message_history.len().saturating_sub(1);
        if self.scroll_offset < max_offset {
            self.scroll_offset += 1;
        }
    }
    
    /// Reset scroll to bottom (most recent messages)
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }
    
    /// Get the current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }
    
    /// Submit the current input as a user message and clear the buffer
    pub fn submit_current_input(&mut self) -> bool {
        let content = self.input_buffer.content().trim().to_string();
        
        if content.is_empty() {
            return false;
        }
        
        // Add to input history
        self.input_history.add(content.clone());
        
        // Add as user message to chat
        self.add_user_message(content);
        
        // Clear the input buffer
        self.input_buffer.clear();
        
        // Reset to normal mode
        self.input_mode = InputMode::Normal;
        
        // Auto-scroll to bottom to show new message
        self.scroll_to_bottom();
        
        true
    }
    
    /// Start navigating input history (Up arrow pressed)
    pub fn navigate_history_previous(&mut self) -> bool {
        match self.input_mode {
            InputMode::History => {
                // Already navigating, go to previous entry
                if let Some(entry) = self.input_history.navigate_previous() {
                    self.input_buffer.clear();
                    self.input_buffer.insert_str(entry);
                    true
                } else {
                    false
                }
            }
            InputMode::Normal if self.input_buffer.is_empty() => {
                // Start history navigation
                if let Some(entry) = self.input_history.start_navigation(String::new()) {
                    self.input_mode = InputMode::History;
                    self.input_buffer.clear();
                    self.input_buffer.insert_str(entry);
                    true
                } else {
                    false
                }
            }
            _ => false, // Can't start history navigation with content in buffer
        }
    }
    
    /// Navigate to next entry in input history (Down arrow pressed)
    pub fn navigate_history_next(&mut self) -> bool {
        if self.input_mode != InputMode::History {
            return false;
        }
        
        if let Some(entry) = self.input_history.navigate_next() {
            self.input_buffer.clear();
            self.input_buffer.insert_str(entry);
            true
        } else {
            // Reached end of history, restore temp buffer or clear
            if let Some(temp) = self.input_history.stop_navigation() {
                self.input_buffer.clear();
                self.input_buffer.insert_str(&temp);
            } else {
                self.input_buffer.clear();
            }
            self.input_mode = InputMode::Normal;
            true
        }
    }
    
    /// Cancel history navigation (Escape pressed)
    pub fn cancel_history_navigation(&mut self) -> bool {
        if self.input_mode != InputMode::History {
            return false;
        }
        
        // Restore original content
        if let Some(temp) = self.input_history.stop_navigation() {
            self.input_buffer.clear();
            self.input_buffer.insert_str(&temp);
        } else {
            self.input_buffer.clear();
        }
        
        self.input_mode = InputMode::Normal;
        true
    }
    
    /// Get a reference to the input buffer
    pub fn input_buffer(&self) -> &InputBuffer {
        &self.input_buffer
    }
    
    /// Get a mutable reference to the input buffer
    pub fn input_buffer_mut(&mut self) -> &mut InputBuffer {
        &mut self.input_buffer
    }
    
    /// Get the current input mode
    pub fn input_mode(&self) -> &InputMode {
        &self.input_mode
    }
    
    /// Check if we should show input history position indicator
    pub fn input_history_position(&self) -> Option<(usize, usize)> {
        if self.input_mode == InputMode::History {
            self.input_history.current_position()
        } else {
            None
        }
    }
}