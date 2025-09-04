use orkee_projects::Project;
use crate::chat::{MessageHistory, ChatMessage};
use crate::input::{InputBuffer, InputHistory, InputMode};
use crate::command_popup::CommandPopup;
use crate::mention_popup::MentionPopup;
use std::time::{Duration, Instant};

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
    /// Track last escape key press for double-escape detection
    last_escape_time: Option<Instant>,
    /// Timeout for double-escape detection (500ms)
    escape_timeout: Duration,
    /// Track last Ctrl+C key press for double Ctrl+C quit detection
    last_ctrl_c_time: Option<Instant>,
    /// Timeout for double Ctrl+C detection (1000ms)
    ctrl_c_timeout: Duration,
    /// ID of message currently being edited
    editing_message_id: Option<String>,
    /// Current focus area (chat or input)
    focus_area: FocusArea,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    Projects,
    ProjectDetail,
    Settings,
    Chat,
}

/// Focus areas within the chat interface
#[derive(Debug, Clone, PartialEq)]
pub enum FocusArea {
    /// Focus on chat messages area (can scroll)
    Chat,
    /// Focus on input area (can type)
    Input,
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
            last_escape_time: None,
            escape_timeout: Duration::from_millis(500),
            last_ctrl_c_time: None,
            ctrl_c_timeout: Duration::from_millis(1000),
            editing_message_id: None,
            focus_area: FocusArea::Input, // Start with input focused
        };
        
        // Add welcome message
        state.message_history.add_system_message("Welcome to Orkee TUI! Type a message to get started.");
        state
    }
    
    pub fn set_projects(&mut self, projects: Vec<Project>) {
        self.projects = projects;
        // Reset selection if projects changed and current selection is invalid
        if let Some(selected) = self.selected_project {
            if selected >= self.projects.len() {
                self.selected_project = if self.projects.is_empty() { None } else { Some(0) };
            }
        }
    }
    
    /// Navigate to previous project in list
    pub fn select_previous_project(&mut self) -> bool {
        if self.projects.is_empty() {
            return false;
        }
        
        match self.selected_project {
            None => {
                self.selected_project = Some(self.projects.len() - 1);
                true
            }
            Some(0) => {
                self.selected_project = Some(self.projects.len() - 1);
                true
            }
            Some(index) => {
                self.selected_project = Some(index - 1);
                true
            }
        }
    }
    
    /// Navigate to next project in list
    pub fn select_next_project(&mut self) -> bool {
        if self.projects.is_empty() {
            return false;
        }
        
        match self.selected_project {
            None => {
                self.selected_project = Some(0);
                true
            }
            Some(index) if index + 1 >= self.projects.len() => {
                self.selected_project = Some(0);
                true
            }
            Some(index) => {
                self.selected_project = Some(index + 1);
                true
            }
        }
    }
    
    /// Get the currently selected project
    pub fn get_selected_project(&self) -> Option<&Project> {
        self.selected_project.and_then(|index| self.projects.get(index))
    }
    
    /// View details of the selected project
    pub fn view_selected_project_details(&mut self) -> bool {
        if self.selected_project.is_some() && !self.projects.is_empty() {
            self.current_screen = Screen::ProjectDetail;
            true
        } else {
            false
        }
    }
    
    /// Return to projects list from detail view
    pub fn return_to_projects_list(&mut self) {
        if self.current_screen == Screen::ProjectDetail {
            self.current_screen = Screen::Projects;
        }
    }
    
    pub fn next_screen(&mut self) {
        self.current_screen = match self.current_screen {
            Screen::Dashboard => Screen::Projects,
            Screen::Projects => Screen::Settings, 
            Screen::ProjectDetail => Screen::Projects, // Return to projects list
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
        
        if let Some(edit_id) = &self.editing_message_id.clone() {
            // Replace the existing message content
            if let Some(msg) = self.message_history.get_message_mut(&edit_id) {
                msg.content = content.clone();
                msg.mark_edited();
                // Update timestamp to current time
                msg.timestamp = chrono::Utc::now();
            }
            
            // Add a system message to confirm edit
            self.add_system_message("Message edited".to_string());
            
            // Clear edit mode
            self.editing_message_id = None;
            self.input_mode = InputMode::Normal;
        } else {
            // Normal message submission
            // Add to input history
            self.input_history.add(content.clone());
            
            // Add as user message to chat
            self.add_user_message(content);
            
            // Reset to normal mode
            self.input_mode = InputMode::Normal;
        }
        
        // Clear the input buffer
        self.input_buffer.clear();
        
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
        self.focus_input(); // Force focus to input when entering command mode
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
        self.focus_input(); // Force focus to input when entering mention mode
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
    
    // Message editing methods (Phase 5)
    
    /// Handle escape key press and detect double-escape
    pub fn handle_escape_key(&mut self) -> EscapeAction {
        let now = Instant::now();
        
        if let Some(last_time) = self.last_escape_time {
            if now.duration_since(last_time) < self.escape_timeout {
                // Double escape detected
                self.last_escape_time = None;
                return EscapeAction::EditPreviousMessage;
            }
        }
        
        self.last_escape_time = Some(now);
        EscapeAction::SingleEscape
    }

    /// Handle Ctrl+C key press, detecting double press for quit
    /// If input has text: first Ctrl+C clears it, then 2 more Ctrl+C presses quit
    /// If input is empty: 2 Ctrl+C presses quit  
    pub fn handle_ctrl_c_key(&mut self) -> CtrlCAction {
        let now = Instant::now();
        
        // Check if input buffer has content
        let input_is_empty = self.input_buffer.is_empty();
        
        if !input_is_empty {
            // Input has text - always clear it and reset timer
            self.last_ctrl_c_time = None;
            return CtrlCAction::ClearInput;
        }
        
        // Input is empty - track presses for quit detection
        if let Some(last_time) = self.last_ctrl_c_time {
            if now.duration_since(last_time) < self.ctrl_c_timeout {
                // Second Ctrl+C on empty input within timeout - quit
                self.last_ctrl_c_time = None;
                return CtrlCAction::QuitApplication;
            }
        }
        
        // First Ctrl+C on empty input - start timer
        self.last_ctrl_c_time = Some(now);
        CtrlCAction::ClearInput
    }
    
    /// Load the previous user message into the input buffer for editing
    pub fn load_previous_message_for_edit(&mut self) -> bool {
        if let Some(last_msg) = self.message_history.last_user_message() {
            // Store the message ID we're editing
            self.editing_message_id = Some(last_msg.id.clone());
            
            // Load content into input buffer
            self.input_buffer.clear();
            self.input_buffer.insert_str(&last_msg.content);
            self.input_buffer.move_to_end();
            
            // Set visual mode indicator and focus input
            self.input_mode = InputMode::Edit;
            self.focus_input(); // Force focus to input when entering edit mode
            
            true
        } else {
            false
        }
    }
    
    /// Cancel editing and restore normal input
    pub fn cancel_message_edit(&mut self) {
        self.editing_message_id = None;
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.last_escape_time = None; // Reset escape timing
    }
    
    /// Check if currently editing a message
    pub fn is_editing_message(&self) -> bool {
        self.editing_message_id.is_some()
    }
    
    /// Get the ID of the message being edited
    pub fn editing_message_id(&self) -> Option<&String> {
        self.editing_message_id.as_ref()
    }
    
    // Focus management methods
    
    /// Cycle focus between chat and input areas
    pub fn cycle_focus(&mut self) {
        self.focus_area = match self.focus_area {
            FocusArea::Chat => FocusArea::Input,
            FocusArea::Input => FocusArea::Chat,
        };
    }
    
    /// Get the current focus area
    pub fn focus_area(&self) -> &FocusArea {
        &self.focus_area
    }
    
    /// Check if chat area is focused
    pub fn is_chat_focused(&self) -> bool {
        self.focus_area == FocusArea::Chat
    }
    
    /// Check if input area is focused
    pub fn is_input_focused(&self) -> bool {
        self.focus_area == FocusArea::Input
    }
    
    /// Force focus to input area (used when entering input modes)
    pub fn focus_input(&mut self) {
        self.focus_area = FocusArea::Input;
    }
}

/// Action to take when escape key is pressed
#[derive(Debug, Clone, PartialEq)]
pub enum EscapeAction {
    /// Single escape - normal escape behavior
    SingleEscape,
    /// Double escape detected - edit previous message
    EditPreviousMessage,
}

/// Actions that can result from Ctrl+C key press
#[derive(Debug, Clone, PartialEq)]
pub enum CtrlCAction {
    /// Single Ctrl+C - clear input buffer
    ClearInput,
    /// Double Ctrl+C detected - quit application
    QuitApplication,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::MessageAuthor;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_single_escape_detection() {
        let mut state = AppState::new(20);
        
        let action = state.handle_escape_key();
        assert_eq!(action, EscapeAction::SingleEscape);
        assert!(state.last_escape_time.is_some());
    }

    #[test]
    fn test_double_escape_detection() {
        let mut state = AppState::new(20);
        
        // First escape
        let action1 = state.handle_escape_key();
        assert_eq!(action1, EscapeAction::SingleEscape);
        
        // Second escape within timeout
        let action2 = state.handle_escape_key();
        assert_eq!(action2, EscapeAction::EditPreviousMessage);
        assert!(state.last_escape_time.is_none()); // Should be reset
    }

    #[test]
    fn test_escape_timeout() {
        let mut state = AppState::new(20);
        state.escape_timeout = Duration::from_millis(10); // Very short timeout
        
        // First escape
        let action1 = state.handle_escape_key();
        assert_eq!(action1, EscapeAction::SingleEscape);
        
        // Wait for timeout
        thread::sleep(Duration::from_millis(20));
        
        // Second escape after timeout
        let action2 = state.handle_escape_key();
        assert_eq!(action2, EscapeAction::SingleEscape); // Should be single again
    }

    #[test]
    fn test_load_previous_message_for_edit() {
        let mut state = AppState::new(20);
        
        // Add a user message
        state.add_user_message("Test message".to_string());
        
        // Should successfully load message for edit
        assert!(state.load_previous_message_for_edit());
        assert_eq!(state.input_mode, InputMode::Edit);
        assert_eq!(state.input_buffer.content(), "Test message");
        assert!(state.is_editing_message());
    }

    #[test]
    fn test_load_previous_message_for_edit_no_messages() {
        let mut state = AppState::new(20);
        
        // Should fail to load message when no user messages exist
        assert!(!state.load_previous_message_for_edit());
        assert_eq!(state.input_mode, InputMode::Normal);
        assert!(!state.is_editing_message());
    }

    #[test]
    fn test_cancel_message_edit() {
        let mut state = AppState::new(20);
        
        // Add a message and enter edit mode
        state.add_user_message("Test message".to_string());
        state.load_previous_message_for_edit();
        
        assert!(state.is_editing_message());
        assert_eq!(state.input_mode, InputMode::Edit);
        
        // Cancel edit
        state.cancel_message_edit();
        
        assert!(!state.is_editing_message());
        assert_eq!(state.input_mode, InputMode::Normal);
        assert!(state.input_buffer.is_empty());
    }

    #[test]
    fn test_submit_edited_message() {
        let mut state = AppState::new(20);
        
        // Add original message
        let original_msg = state.add_user_message("Original message".to_string());
        let msg_id = original_msg.id.clone();
        
        // Enter edit mode
        state.load_previous_message_for_edit();
        
        // Change the message
        state.input_buffer.clear();
        state.input_buffer.insert_str("Edited message");
        
        // Submit the edit
        assert!(state.submit_current_input());
        
        // Verify message was updated
        let updated_msg = state.message_history.get_message(&msg_id).unwrap();
        assert_eq!(updated_msg.content, "Edited message");
        assert!(updated_msg.edited);
        
        // Verify we're out of edit mode
        assert!(!state.is_editing_message());
        assert_eq!(state.input_mode, InputMode::Normal);
        
        // Verify system confirmation message was added
        let last_msg = state.message_history.last_message().unwrap();
        assert_eq!(last_msg.content, "Message edited");
        assert_eq!(last_msg.author, MessageAuthor::System);
    }

    #[test]
    fn test_submit_normal_message_not_in_edit_mode() {
        let mut state = AppState::new(20);
        
        // Add normal message without being in edit mode
        state.input_buffer.insert_str("Normal message");
        
        let messages_before = state.message_history.len();
        assert!(state.submit_current_input());
        
        // Should add new message, not edit existing
        assert_eq!(state.message_history.len(), messages_before + 1);
        let last_msg = state.message_history.last_message().unwrap();
        assert_eq!(last_msg.content, "Normal message");
        assert!(!last_msg.edited);
    }

    #[test] 
    fn test_edit_mode_with_system_messages() {
        let mut state = AppState::new(20);
        
        // Add system message and user message
        state.add_system_message("System message".to_string());
        state.add_user_message("User message".to_string());
        
        // Should load the user message, not the system message
        assert!(state.load_previous_message_for_edit());
        assert_eq!(state.input_buffer.content(), "User message");
    }

    #[test]
    fn test_multiple_user_messages_edit_latest() {
        let mut state = AppState::new(20);
        
        // Add multiple user messages
        state.add_user_message("First message".to_string());
        state.add_user_message("Second message".to_string());
        state.add_user_message("Third message".to_string());
        
        // Should load the most recent user message
        assert!(state.load_previous_message_for_edit());
        assert_eq!(state.input_buffer.content(), "Third message");
    }

    #[test]
    fn test_focus_cycling() {
        let mut state = AppState::new(20);
        
        // Should start with input focused
        assert_eq!(state.focus_area(), &FocusArea::Input);
        assert!(state.is_input_focused());
        assert!(!state.is_chat_focused());
        
        // Cycle focus to chat
        state.cycle_focus();
        assert_eq!(state.focus_area(), &FocusArea::Chat);
        assert!(state.is_chat_focused());
        assert!(!state.is_input_focused());
        
        // Cycle back to input
        state.cycle_focus();
        assert_eq!(state.focus_area(), &FocusArea::Input);
        assert!(state.is_input_focused());
        assert!(!state.is_chat_focused());
    }

    #[test]
    fn test_special_modes_force_input_focus() {
        let mut state = AppState::new(20);
        
        // Start with chat focused
        state.cycle_focus(); // Now chat is focused
        assert!(state.is_chat_focused());
        
        // Entering command mode should focus input
        state.enter_command_mode();
        assert!(state.is_input_focused());
        
        // Reset and test mention mode
        state.exit_command_mode();
        state.cycle_focus(); // Focus chat again
        assert!(state.is_chat_focused());
        
        state.enter_mention_mode(0);
        assert!(state.is_input_focused());
        
        // Reset and test edit mode
        state.exit_mention_mode();
        state.add_user_message("Test message".to_string());
        state.cycle_focus(); // Focus chat
        assert!(state.is_chat_focused());
        
        state.load_previous_message_for_edit();
        assert!(state.is_input_focused());
    }

    #[test]
    fn test_input_focus_requirement_for_typing() {
        let mut state = AppState::new(20);
        
        // Start with input focused - typing should work normally
        assert!(state.is_input_focused());
        state.input_buffer.insert_char('h');
        assert_eq!(state.input_buffer.content(), "h");
        
        // Switch to chat focus
        state.cycle_focus();
        assert!(state.is_chat_focused());
        
        // When chat is focused, there should be a way to simulate this behavior
        // For now, we'll test the focus switching mechanism works
        
        // Verify that special modes always focus input
        state.enter_command_mode();
        assert!(state.is_input_focused()); // Command mode forces input focus
        
        state.exit_command_mode();
        state.cycle_focus(); // Focus chat again
        assert!(state.is_chat_focused());
        
        // Force focus back to input (simulating what happens when user types)
        state.focus_input();
        assert!(state.is_input_focused());
    }

    #[test]
    fn test_single_ctrl_c_clears_input() {
        let mut state = AppState::new(20);
        
        // Add some content to input buffer
        state.input_buffer.insert_char('h');
        state.input_buffer.insert_char('i');
        assert_eq!(state.input_buffer.content(), "hi");
        
        // First Ctrl+C should clear input and reset timer (not start quit sequence)
        let action = state.handle_ctrl_c_key();
        assert_eq!(action, CtrlCAction::ClearInput);
        assert!(state.last_ctrl_c_time.is_none()); // Timer reset when input had text
    }
    
    #[test]
    fn test_two_ctrl_c_quits_when_input_empty() {
        let mut state = AppState::new(20);
        
        // Start with empty input
        assert!(state.input_buffer.is_empty());
        
        // First Ctrl+C on empty input - starts quit sequence
        let action1 = state.handle_ctrl_c_key();
        assert_eq!(action1, CtrlCAction::ClearInput);
        assert!(state.last_ctrl_c_time.is_some());
        
        // Second Ctrl+C within timeout - should quit since input was empty
        let action2 = state.handle_ctrl_c_key();
        assert_eq!(action2, CtrlCAction::QuitApplication);
        assert!(state.last_ctrl_c_time.is_none()); // Should be reset
    }
    
    #[test]
    fn test_ctrl_c_with_text_requires_three_presses() {
        let mut state = AppState::new(20);
        
        // Add content to input
        state.input_buffer.insert_char('h');
        assert!(!state.input_buffer.is_empty());
        
        // First Ctrl+C - clears input and resets timer
        let action1 = state.handle_ctrl_c_key();
        assert_eq!(action1, CtrlCAction::ClearInput);
        assert!(state.last_ctrl_c_time.is_none()); // Timer should be reset
        
        // Now input is empty, simulate clearing it
        state.input_buffer.clear();
        assert!(state.input_buffer.is_empty());
        
        // Second Ctrl+C (first on empty input) - starts quit sequence
        let action2 = state.handle_ctrl_c_key();
        assert_eq!(action2, CtrlCAction::ClearInput);
        assert!(state.last_ctrl_c_time.is_some()); // Timer should be set
        
        // Third Ctrl+C (second on empty input) - should quit
        let action3 = state.handle_ctrl_c_key();
        assert_eq!(action3, CtrlCAction::QuitApplication);
        assert!(state.last_ctrl_c_time.is_none()); // Timer should be reset
    }

    #[test]
    fn test_ctrl_c_timeout() {
        let mut state = AppState::new(20);
        state.ctrl_c_timeout = Duration::from_millis(10); // Very short timeout
        
        // Start with empty input
        assert!(state.input_buffer.is_empty());
        
        // First Ctrl+C
        let action1 = state.handle_ctrl_c_key();
        assert_eq!(action1, CtrlCAction::ClearInput);
        
        // Wait longer than timeout
        std::thread::sleep(Duration::from_millis(15));
        
        // Second Ctrl+C after timeout - should not quit
        let action2 = state.handle_ctrl_c_key();
        assert_eq!(action2, CtrlCAction::ClearInput); // Should be single again
    }
}