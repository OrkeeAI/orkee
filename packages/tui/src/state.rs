use orkee_projects::Project;
use crate::chat::{MessageHistory, ChatMessage};
use crate::input::{InputBuffer, InputHistory, InputMode};
use crate::command_popup::CommandPopup;
use crate::mention_popup::MentionPopup;

/// Application state management
#[derive(Debug)]
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
    pub command_popup: Option<CommandPopup>,
    pub mention_popup: Option<MentionPopup>,
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
            command_popup: None,
            mention_popup: None,
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
    
    /// Enter command mode and show command popup
    pub fn enter_command_mode(&mut self) {
        self.input_mode = InputMode::Command;
        let mut popup = CommandPopup::new();
        
        // Get the command text (everything after the '/')
        let input_content = self.input_buffer.content();
        let command_text = input_content.strip_prefix('/').unwrap_or("");
        popup.update_filter(command_text);
        
        self.command_popup = Some(popup);
    }
    
    /// Exit command mode and hide command popup
    pub fn exit_command_mode(&mut self) {
        if self.input_mode == InputMode::Command {
            self.input_mode = InputMode::Normal;
            self.command_popup = None;
        }
    }
    
    /// Check if currently in command mode
    pub fn is_command_mode(&self) -> bool {
        self.input_mode == InputMode::Command
    }
    
    /// Update command popup filter when typing in command mode
    pub fn update_command_filter(&mut self) {
        if let Some(ref mut popup) = self.command_popup {
            let input_content = self.input_buffer.content();
            let command_text = input_content.strip_prefix('/').unwrap_or("");
            popup.update_filter(command_text);
        }
    }
    
    /// Navigate command popup up
    pub fn command_popup_up(&mut self) -> bool {
        if let Some(ref mut popup) = self.command_popup {
            popup.move_up();
            true
        } else {
            false
        }
    }
    
    /// Navigate command popup down  
    pub fn command_popup_down(&mut self) -> bool {
        if let Some(ref mut popup) = self.command_popup {
            popup.move_down();
            true
        } else {
            false
        }
    }
    
    /// Complete the currently selected command
    pub fn complete_selected_command(&mut self) -> Option<String> {
        if let Some(ref popup) = self.command_popup {
            if let Some(item) = popup.selected_item() {
                let usage = item.usage.clone();
                
                // Clear input buffer and insert the full command usage
                self.input_buffer.clear();
                self.input_buffer.insert_str(&usage);
                
                // If command doesn't require args, we can exit command mode immediately
                if !item.command.requires_args() {
                    self.exit_command_mode();
                    return Some(usage);
                } else {
                    // Keep in command mode for argument entry, but hide popup
                    self.command_popup = None;
                }
                
                return Some(usage);
            }
        }
        None
    }
    
    /// Get reference to command popup for UI rendering
    pub fn command_popup(&self) -> Option<&CommandPopup> {
        self.command_popup.as_ref()
    }
    
    // Mention popup methods
    
    /// Enter mention mode and show mention popup
    pub fn enter_mention_mode(&mut self, mention_start_position: usize) {
        self.input_mode = InputMode::Search;
        let popup = MentionPopup::from_projects(&self.projects, mention_start_position);
        self.mention_popup = Some(popup);
    }
    
    /// Exit mention mode and hide mention popup
    pub fn exit_mention_mode(&mut self) {
        if self.input_mode == InputMode::Search {
            self.input_mode = InputMode::Normal;
            self.mention_popup = None;
        }
    }
    
    /// Check if currently in mention mode
    pub fn is_mention_mode(&self) -> bool {
        self.input_mode == InputMode::Search
    }
    
    /// Update mention popup filter when typing in mention mode
    pub fn update_mention_filter(&mut self) {
        if let Some(ref mut popup) = self.mention_popup {
            let input_content = self.input_buffer.content();
            let mention_start = popup.mention_start_position();
            
            // Extract the text after @ for filtering
            if mention_start < input_content.len() {
                let mention_text = &input_content[mention_start + 1..]; // +1 to skip @
                popup.update_filter(mention_text);
            } else {
                popup.update_filter("");
            }
        }
    }
    
    /// Navigate mention popup up
    pub fn mention_popup_up(&mut self) -> bool {
        if let Some(ref mut popup) = self.mention_popup {
            popup.move_up();
            true
        } else {
            false
        }
    }
    
    /// Navigate mention popup down
    pub fn mention_popup_down(&mut self) -> bool {
        if let Some(ref mut popup) = self.mention_popup {
            popup.move_down();
            true
        } else {
            false
        }
    }
    
    /// Complete the currently selected mention
    pub fn complete_selected_mention(&mut self) -> Option<String> {
        if let Some(ref popup) = self.mention_popup {
            if let Some(item) = popup.selected_item() {
                let insertion_text = item.insertion_text();
                let current_cursor = self.input_buffer.cursor_position();
                let (start, end) = popup.replacement_range(current_cursor);
                
                // Replace the @ and following text with the selected item
                let content = self.input_buffer.content().to_string();
                let before = &content[..start];
                let after = &content[end..];
                let new_content = format!("{}{}{}", before, insertion_text, after);
                let new_cursor_pos = before.len() + insertion_text.len();
                
                // Update the buffer
                self.input_buffer.clear();
                self.input_buffer.insert_str(&new_content);
                
                // Position cursor after the inserted text
                self.input_buffer.set_cursor_position(new_cursor_pos);
                
                // Exit mention mode
                self.exit_mention_mode();
                
                return Some(insertion_text);
            }
        }
        None
    }
    
    /// Get reference to mention popup for UI rendering
    pub fn mention_popup(&self) -> Option<&MentionPopup> {
        self.mention_popup.as_ref()
    }
    
    /// Check if the character at the given position should trigger mention mode
    /// Returns true if @ is preceded by whitespace or is at the start
    pub fn should_trigger_mention(&self, position: usize) -> bool {
        let content = self.input_buffer.content();
        
        if position == 0 {
            return true; // @ at start of input
        }
        
        // Check if previous character is whitespace
        if let Some(prev_char) = content.chars().nth(position - 1) {
            prev_char.is_whitespace()
        } else {
            false
        }
    }
}