use orkee_projects::{Project, ProjectCreateInput, ProjectUpdateInput, ProjectStatus, Priority, create_project, update_project, delete_project};
use crate::chat::{MessageHistory, ChatMessage};
use crate::input::{InputBuffer, InputHistory, InputMode};
use crate::command_popup::CommandPopup;
use crate::mention_popup::MentionPopup;
use crate::ui::widgets::{FormWidget, FormField, FormStep};
use crate::ui::widgets::form::FieldValue;
use crate::ui::widgets::dialog::{ConfirmationDialog, DialogResult};
use tui_input::Input;
use tui_textarea::TextArea;
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
    /// Form state for project creation/editing
    pub form_state: Option<FormState>,
    /// Confirmation dialog for destructive actions
    pub confirmation_dialog: Option<ConfirmationDialog>,
    /// Pending action waiting for confirmation
    pub pending_action: Option<PendingAction>,
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

/// Form state for project creation and editing
#[derive(Debug)]
pub struct FormState {
    pub form: FormWidget,
    pub step: usize,
    pub total_steps: usize,
    pub can_submit: bool,
    pub form_mode: FormMode,
}

/// Mode for form operation
#[derive(Debug, Clone, PartialEq)]
pub enum FormMode {
    /// Creating a new project
    Create,
    /// Editing an existing project by ID
    Edit(String),
}

/// Actions that require user confirmation
#[derive(Debug, Clone, PartialEq)]
pub enum PendingAction {
    /// Delete a project by ID
    DeleteProject(String),
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
            form_state: None,
            confirmation_dialog: None,
            pending_action: None,
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

    // Form management methods

    /// Start creating a new project with the form
    pub fn start_project_creation(&mut self) {
        let mut form = FormWidget::new("Create New Project".to_string());

        // Step 1: Basic Information
        let mut step1 = FormStep::new("Basic Information".to_string());
        step1.add_field(
            FormField::text("name".to_string(), "Project Name".to_string(), true)
                .with_validator(Self::validate_project_name_static)
        );
        step1.add_field(
            FormField::path("path".to_string(), "Project Path".to_string(), true)
                .with_validator(Self::validate_project_path_static)
        );
        step1.add_field(
            FormField::multiline_text("description".to_string(), "Description".to_string(), false)
        );
        form.add_step(step1);

        // Step 2: Configuration (Status, Priority, Tags)
        let mut step2 = FormStep::new("Configuration".to_string());
        step2.add_field(
            FormField::selection("status".to_string(), "Status".to_string(), 
                vec!["active".to_string(), "archived".to_string()], false)
        );
        step2.add_field(
            FormField::selection("priority".to_string(), "Priority".to_string(),
                vec!["high".to_string(), "medium".to_string(), "low".to_string()], false)
        );
        step2.add_field(
            FormField::tags("tags".to_string(), "Tags".to_string(), false)
        );
        form.add_step(step2);

        // Step 3: Scripts
        let mut step3 = FormStep::new("Scripts".to_string());
        step3.add_field(
            FormField::text("setup_script".to_string(), "Setup Script".to_string(), false)
                .with_placeholder("npm install".to_string())
                .with_help_text("Command to run when setting up the project".to_string())
        );
        step3.add_field(
            FormField::text("dev_script".to_string(), "Development Script".to_string(), false)
                .with_placeholder("npm run dev".to_string())
                .with_help_text("Command to start development server".to_string())
        );
        step3.add_field(
            FormField::text("cleanup_script".to_string(), "Cleanup Script".to_string(), false)
                .with_placeholder("npm run clean".to_string())
                .with_help_text("Command to clean up project artifacts".to_string())
        );
        form.add_step(step3);

        // Step 4: Review & Confirmation
        let step4 = FormStep::new("Review & Confirm".to_string());
        // No fields needed - this step will show a summary of all previous steps
        form.add_step(step4);

        self.form_state = Some(FormState {
            form,
            step: 1,
            total_steps: 4,
            can_submit: false,
            form_mode: FormMode::Create,
        });

        self.input_mode = InputMode::Form;
        self.current_screen = Screen::Projects; // Show form on projects screen
        self.focus_input();
    }

    /// Start editing an existing project with the form
    pub fn start_project_edit(&mut self, project_id: String) {
        // Find the project to edit
        if let Some(project) = self.projects.iter().find(|p| p.id == project_id) {
            let form = self.create_edit_form(project);
            
            self.form_state = Some(FormState {
                form,
                step: 1,
                total_steps: 4,
                can_submit: false,
                form_mode: FormMode::Edit(project_id),
            });

            self.input_mode = InputMode::Form;
            self.current_screen = Screen::Projects; // Show form on projects screen
            self.focus_input();
        }
    }

    /// Create a pre-populated form for editing an existing project
    fn create_edit_form(&self, project: &Project) -> FormWidget {
        let mut form = FormWidget::new_for_edit(format!("Edit Project: {}", project.name));

        // Step 1: Basic Information
        let mut step1 = FormStep::new("Basic Information".to_string());
        
        // Pre-populate name field
        let name_input = Input::default().with_value(project.name.clone());
        let name_field = FormField {
            name: "name".to_string(),
            label: "Project Name".to_string(),
            field_value: FieldValue::SingleLine(name_input),
            field_type: crate::ui::widgets::FieldType::Text,
            required: true,
            validator: Some(Self::validate_project_name_static),
            placeholder: None,
            help_text: Some("Enter/↓/Tab to continue, ↑ to go back".to_string()),
        };
        step1.add_field(name_field);
        
        // Pre-populate path field
        let path_input = Input::default().with_value(project.project_root.clone());
        let path_field = FormField {
            name: "path".to_string(),
            label: "Project Path".to_string(),
            field_value: FieldValue::SingleLine(path_input),
            field_type: crate::ui::widgets::FieldType::Path,
            required: true,
            validator: Some(Self::validate_project_path_static),
            placeholder: Some("/path/to/project".to_string()),
            help_text: Some("Enter the full path to the project directory • Enter/↓/Tab to continue, ↑ to go back".to_string()),
        };
        step1.add_field(path_field);
        
        // Pre-populate description field
        let mut desc_textarea = if let Some(ref description) = project.description {
            TextArea::new(vec![description.clone()])
        } else {
            TextArea::default()
        };
        desc_textarea.set_cursor_line_style(ratatui::style::Style::default());
        let desc_field = FormField {
            name: "description".to_string(),
            label: "Description".to_string(),
            field_value: FieldValue::MultiLine(desc_textarea),
            field_type: crate::ui::widgets::FieldType::MultilineText,
            required: false,
            validator: None,
            placeholder: Some("Enter description...".to_string()),
            help_text: Some("Use Shift+Enter for new lines, Enter/↓/Tab to continue, ↑ to go back".to_string()),
        };
        step1.add_field(desc_field);
        form.add_step(step1);

        // Step 2: Configuration (Status, Priority, Tags)
        let mut step2 = FormStep::new("Configuration".to_string());
        
        // Pre-populate status field
        let status_options = vec!["active".to_string(), "archived".to_string()];
        let status_selected = match project.status {
            ProjectStatus::Active => 0,
            ProjectStatus::Archived => 1,
        };
        let mut status_field = FormField::selection("status".to_string(), "Status".to_string(), status_options, false);
        if let FieldValue::Selection { selected, .. } = &mut status_field.field_value {
            *selected = status_selected;
        }
        step2.add_field(status_field);
        
        // Pre-populate priority field
        let priority_options = vec!["high".to_string(), "medium".to_string(), "low".to_string()];
        let priority_selected = match project.priority {
            Priority::High => 0,
            Priority::Medium => 1,
            Priority::Low => 2,
        };
        let mut priority_field = FormField::selection("priority".to_string(), "Priority".to_string(), priority_options, false);
        if let FieldValue::Selection { ref mut selected, .. } = &mut priority_field.field_value {
            *selected = priority_selected;
        }
        step2.add_field(priority_field);
        
        // Pre-populate tags field
        let tags_string = if let Some(ref tags) = project.tags {
            tags.join(", ")
        } else {
            String::new()
        };
        let tags_input = Input::default().with_value(tags_string);
        let tags_field = FormField {
            name: "tags".to_string(),
            label: "Tags".to_string(),
            field_value: FieldValue::SingleLine(tags_input),
            field_type: crate::ui::widgets::FieldType::Tags,
            required: false,
            validator: None,
            placeholder: Some("tag1, tag2, tag3".to_string()),
            help_text: Some("Separate tags with commas • Enter/↓/Tab to continue, ↑ to go back".to_string()),
        };
        step2.add_field(tags_field);
        form.add_step(step2);

        // Step 3: Scripts
        let mut step3 = FormStep::new("Scripts".to_string());
        
        // Pre-populate setup script
        let setup_value = project.setup_script.as_deref().unwrap_or("");
        let setup_input = Input::default().with_value(setup_value.to_string());
        let setup_field = FormField {
            name: "setup_script".to_string(),
            label: "Setup Script".to_string(),
            field_value: FieldValue::SingleLine(setup_input),
            field_type: crate::ui::widgets::FieldType::Text,
            required: false,
            validator: None,
            placeholder: Some("npm install".to_string()),
            help_text: Some("Command to run when setting up the project".to_string()),
        };
        step3.add_field(setup_field);
        
        // Pre-populate dev script
        let dev_value = project.dev_script.as_deref().unwrap_or("");
        let dev_input = Input::default().with_value(dev_value.to_string());
        let dev_field = FormField {
            name: "dev_script".to_string(),
            label: "Development Script".to_string(),
            field_value: FieldValue::SingleLine(dev_input),
            field_type: crate::ui::widgets::FieldType::Text,
            required: false,
            validator: None,
            placeholder: Some("npm run dev".to_string()),
            help_text: Some("Command to start development server".to_string()),
        };
        step3.add_field(dev_field);
        
        // Pre-populate cleanup script
        let cleanup_value = project.cleanup_script.as_deref().unwrap_or("");
        let cleanup_input = Input::default().with_value(cleanup_value.to_string());
        let cleanup_field = FormField {
            name: "cleanup_script".to_string(),
            label: "Cleanup Script".to_string(),
            field_value: FieldValue::SingleLine(cleanup_input),
            field_type: crate::ui::widgets::FieldType::Text,
            required: false,
            validator: None,
            placeholder: Some("npm run clean".to_string()),
            help_text: Some("Command to clean up project artifacts".to_string()),
        };
        step3.add_field(cleanup_field);
        form.add_step(step3);

        // Step 4: Review & Confirmation
        let step4 = FormStep::new("Review & Confirm".to_string());
        form.add_step(step4);

        form
    }

    /// Get mutable reference to current form
    pub fn form_mut(&mut self) -> Option<&mut FormWidget> {
        self.form_state.as_mut().map(|fs| &mut fs.form)
    }

    /// Get reference to current form
    pub fn form(&self) -> Option<&FormWidget> {
        self.form_state.as_ref().map(|fs| &fs.form)
    }

    /// Check if currently in form mode
    pub fn is_form_mode(&self) -> bool {
        self.input_mode == InputMode::Form
    }

    /// Navigate to next field in form, or next step if at end of current step
    pub fn form_next_field(&mut self) -> bool {
        if let Some(ref mut form_state) = self.form_state {
            // Try to move to next field within current step first
            if form_state.form.next_field() {
                true
            } else {
                // At end of current step - try to move to next step
                self.form_next_step()
            }
        } else {
            false
        }
    }

    /// Navigate to previous field in form, or previous step if at start of current step  
    pub fn form_previous_field(&mut self) -> bool {
        if let Some(ref mut form_state) = self.form_state {
            // Try to move to previous field within current step first
            if form_state.form.previous_field() {
                true
            } else {
                // At start of current step - try to move to previous step
                if form_state.form.previous_step() {
                    // Move to last field of the previous step
                    if let Some(fields) = form_state.form.current_step_fields() {
                        if !fields.is_empty() {
                            form_state.form.current_field = fields.len() - 1;
                        }
                    }
                    true
                } else {
                    false
                }
            }
        } else {
            false
        }
    }

    /// Navigate to next step in form
    pub fn form_next_step(&mut self) -> bool {
        if let Some(ref mut form_state) = self.form_state {
            // Validate current step before moving to next
            if form_state.form.validate_current_step() {
                form_state.form.next_step()
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Navigate to previous step in form
    pub fn form_previous_step(&mut self) -> bool {
        if let Some(ref mut form_state) = self.form_state {
            form_state.form.previous_step()
        } else {
            false
        }
    }

    /// This method is no longer needed - input is handled directly by widgets

    /// Validate current form field
    pub fn form_validate_current_field(&mut self) -> bool {
        if let Some(ref mut form_state) = self.form_state {
            let is_valid = form_state.form.validate_current_field();
            // Update can_submit status based on current step completion
            form_state.can_submit = form_state.form.is_current_step_complete();
            is_valid
        } else {
            false
        }
    }

    /// Static validator for project name (for use in forms)
    fn validate_project_name_static(name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Project name cannot be empty".to_string());
        }
        if name.len() > 100 {
            return Err("Project name cannot exceed 100 characters".to_string());
        }
        // TODO: Add uniqueness check when we have access to existing projects
        Ok(())
    }

    /// Static validator for project path (for use in forms)
    fn validate_project_path_static(path: &str) -> Result<(), String> {
        if path.trim().is_empty() {
            return Err("Project path cannot be empty".to_string());
        }
        
        let path_obj = std::path::Path::new(path.trim());
        if !path_obj.exists() {
            return Err("Path does not exist".to_string());
        }
        if !path_obj.is_dir() {
            return Err("Path must be a directory".to_string());
        }
        
        Ok(())
    }

    /// Check if form can be submitted  
    pub fn form_can_submit(&self) -> bool {
        if let Some(ref form_state) = self.form_state {
            // For multi-step forms, form can be submitted only if:
            // 1. We're on the review step (last step)
            // 2. All previous steps are complete and valid
            form_state.form.is_review_step() && form_state.form.is_all_steps_complete()
        } else {
            false
        }
    }
    
    /// Check if currently on review step
    pub fn form_is_review_step(&self) -> bool {
        if let Some(ref form_state) = self.form_state {
            form_state.form.is_review_step()
        } else {
            false
        }
    }

    /// Check if current form field is multiline
    pub fn form_current_field_is_multiline(&self) -> bool {
        if let Some(ref form_state) = self.form_state {
            if let Some(field) = form_state.form.current_field() {
                // Explicitly check field name and type for description field
                let is_desc = field.name == "description";
                let is_multiline_type = matches!(field.field_type, crate::ui::widgets::FieldType::MultilineText);
                
                // Check if this field is multiline based on name or type
                
                return is_desc || is_multiline_type;
            }
        }
        false
    }

    /// Cancel form and return to projects list
    pub fn cancel_form(&mut self) {
        self.form_state = None;
        self.input_mode = InputMode::Normal;
        self.current_screen = Screen::Projects;
    }

    /// Convert form data to ProjectCreateInput
    fn form_to_project_create_input(&self) -> Option<ProjectCreateInput> {
        if let Some(ref form_state) = self.form_state {
            if matches!(form_state.form_mode, FormMode::Create) {
                let form = &form_state.form;
                
                // Extract values from form fields
                let mut name = None;
                let mut project_root = None;
                let mut description = None;
                let mut status = None;
                let mut priority = None;
                let mut tags = None;
                let mut setup_script = None;
                let mut dev_script = None;
                let mut cleanup_script = None;
                
                // Iterate through all steps and fields
                for step in &form.steps {
                    for field in &step.fields {
                        match field.name.as_str() {
                            "name" => name = Some(field.field_value.value()),
                            "path" => project_root = Some(field.field_value.value()),
                            "description" => {
                                let field_value = field.field_value.value();
                                if !field_value.trim().is_empty() {
                                    description = Some(field_value);
                                }
                            }
                            "status" => {
                                let status_value = field.field_value.value();
                                if !status_value.trim().is_empty() {
                                    status = match status_value.trim().to_lowercase().as_str() {
                                        "active" => Some(ProjectStatus::Active),
                                        "archived" => Some(ProjectStatus::Archived),
                                        _ => None,
                                    };
                                }
                            }
                            "priority" => {
                                let priority_value = field.field_value.value();
                                if !priority_value.trim().is_empty() {
                                    priority = match priority_value.trim().to_lowercase().as_str() {
                                        "high" => Some(Priority::High),
                                        "medium" => Some(Priority::Medium),
                                        "low" => Some(Priority::Low),
                                        _ => None,
                                    };
                                }
                            }
                            "tags" => {
                                let tags_value = field.field_value.value();
                                if !tags_value.trim().is_empty() {
                                    let tag_list: Vec<String> = tags_value
                                        .split(',')
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    if !tag_list.is_empty() {
                                        tags = Some(tag_list);
                                    }
                                }
                            }
                            "setup_script" => {
                                let script_value = field.field_value.value();
                                if !script_value.trim().is_empty() {
                                    setup_script = Some(script_value);
                                }
                            }
                            "dev_script" => {
                                let script_value = field.field_value.value();
                                if !script_value.trim().is_empty() {
                                    dev_script = Some(script_value);
                                }
                            }
                            "cleanup_script" => {
                                let script_value = field.field_value.value();
                                if !script_value.trim().is_empty() {
                                    cleanup_script = Some(script_value);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                if let (Some(name), Some(project_root)) = (name, project_root) {
                    return Some(ProjectCreateInput {
                        name: name.trim().to_string(),
                        project_root: project_root.trim().to_string(),
                        description,
                        status,
                        priority,
                        setup_script,
                        dev_script,
                        cleanup_script,
                        tags,
                        rank: None,
                        task_source: None,
                        manual_tasks: None,
                        mcp_servers: None,
                    });
                }
            }
        }
        None
    }

    /// Convert form data to ProjectUpdateInput for editing
    fn form_to_project_update_input(&self) -> Option<ProjectUpdateInput> {
        if let Some(ref form_state) = self.form_state {
            if let FormMode::Edit(_) = form_state.form_mode {
                let form = &form_state.form;
                
                // Extract values from form fields
                let mut name = None;
                let mut project_root = None;
                let mut description = None;
                let mut status = None;
                let mut priority = None;
                let mut tags = None;
                let mut setup_script = None;
                let mut dev_script = None;
                let mut cleanup_script = None;
                
                // Iterate through all steps and fields
                for step in &form.steps {
                    for field in &step.fields {
                        match field.name.as_str() {
                            "name" => {
                                let field_value = field.field_value.value();
                                if !field_value.trim().is_empty() {
                                    name = Some(field_value.trim().to_string());
                                }
                            }
                            "path" => {
                                let field_value = field.field_value.value();
                                if !field_value.trim().is_empty() {
                                    project_root = Some(field_value.trim().to_string());
                                }
                            }
                            "description" => {
                                let field_value = field.field_value.value();
                                // Include description even if empty (allows clearing)
                                description = Some(field_value);
                            }
                            "status" => {
                                let status_value = field.field_value.value();
                                if !status_value.trim().is_empty() {
                                    status = match status_value.trim().to_lowercase().as_str() {
                                        "active" => Some(ProjectStatus::Active),
                                        "archived" => Some(ProjectStatus::Archived),
                                        _ => None,
                                    };
                                }
                            }
                            "priority" => {
                                let priority_value = field.field_value.value();
                                if !priority_value.trim().is_empty() {
                                    priority = match priority_value.trim().to_lowercase().as_str() {
                                        "high" => Some(Priority::High),
                                        "medium" => Some(Priority::Medium),
                                        "low" => Some(Priority::Low),
                                        _ => None,
                                    };
                                }
                            }
                            "tags" => {
                                let tags_value = field.field_value.value();
                                if !tags_value.trim().is_empty() {
                                    let tag_list: Vec<String> = tags_value
                                        .split(',')
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    tags = Some(tag_list);
                                } else {
                                    // Empty tags field means clear all tags
                                    tags = Some(Vec::new());
                                }
                            }
                            "setup_script" => {
                                let script_value = field.field_value.value();
                                // Include script even if empty (allows clearing)
                                setup_script = Some(if script_value.trim().is_empty() {
                                    String::new()
                                } else {
                                    script_value
                                });
                            }
                            "dev_script" => {
                                let script_value = field.field_value.value();
                                // Include script even if empty (allows clearing)
                                dev_script = Some(if script_value.trim().is_empty() {
                                    String::new()
                                } else {
                                    script_value
                                });
                            }
                            "cleanup_script" => {
                                let script_value = field.field_value.value();
                                // Include script even if empty (allows clearing)
                                cleanup_script = Some(if script_value.trim().is_empty() {
                                    String::new()
                                } else {
                                    script_value
                                });
                            }
                            _ => {}
                        }
                    }
                }
                
                return Some(ProjectUpdateInput {
                    name,
                    project_root,
                    description,
                    status,
                    priority,
                    setup_script,
                    dev_script,
                    cleanup_script,
                    tags,
                    rank: None,
                    task_source: None,
                    manual_tasks: None,
                    mcp_servers: None,
                });
            }
        }
        None
    }

    /// Submit the form and create or update project
    pub async fn submit_form(&mut self) -> Result<(), String> {
        if let Some(ref form_state) = self.form_state {
            match &form_state.form_mode {
                FormMode::Create => {
                    // Handle project creation
                    if let Some(project_input) = self.form_to_project_create_input() {
                        match create_project(project_input).await {
                            Ok(project) => {
                                // Project created successfully
                                self.add_system_message(format!("✅ **Project Created Successfully**\n\n📁 **{}** has been created at `{}`", project.name, project.project_root));
                                
                                // Refresh projects list
                                if let Ok(projects) = orkee_projects::get_all_projects().await {
                                    self.set_projects(projects);
                                }
                                
                                // Cancel form and return to projects list
                                self.cancel_form();
                                
                                Ok(())
                            }
                            Err(e) => {
                                let error_msg = format!("❌ **Failed to Create Project**\n\n{}", e);
                                Err(error_msg)
                            }
                        }
                    } else {
                        Err("❌ **Invalid Form Data**\n\nPlease fill in all required fields.".to_string())
                    }
                }
                FormMode::Edit(project_id) => {
                    // Handle project update
                    if let Some(project_update) = self.form_to_project_update_input() {
                        match update_project(&project_id, project_update).await {
                            Ok(project) => {
                                // Project updated successfully
                                self.add_system_message(format!("✅ **Project Updated Successfully**\n\n📁 **{}** has been updated", project.name));
                                
                                // Refresh projects list
                                if let Ok(projects) = orkee_projects::get_all_projects().await {
                                    self.set_projects(projects);
                                }
                                
                                // Cancel form and return to projects list
                                self.cancel_form();
                                
                                Ok(())
                            }
                            Err(e) => {
                                let error_msg = format!("❌ **Failed to Update Project**\n\n{}", e);
                                Err(error_msg)
                            }
                        }
                    } else {
                        Err("❌ **Invalid Form Data**\n\nPlease check your input and try again.".to_string())
                    }
                }
            }
        } else {
            Err("❌ **No Form Data**\n\nForm is not initialized.".to_string())
        }
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

impl AppState {
    /// Dialog handling methods
    
    /// Show a delete confirmation dialog for the given project ID
    pub fn show_delete_confirmation(&mut self, project_id: String) {
        if let Some(project) = self.projects.iter().find(|p| p.id == project_id) {
            let dialog = ConfirmationDialog::new(
                "Delete Project".to_string(),
                "Are you sure you want to delete this project?".to_string(),
            )
            .dangerous()
            .with_buttons("Delete".to_string(), "Cancel".to_string())
            .with_details(format!(
                "Name: \"{}\"\nPath: {}",
                project.name, project.project_root
            ));

            self.confirmation_dialog = Some(dialog);
            self.pending_action = Some(PendingAction::DeleteProject(project_id));
        }
    }

    /// Handle key input for the confirmation dialog
    pub fn handle_dialog_key(&mut self, key: crossterm::event::KeyCode) -> Option<DialogResult> {
        if let Some(dialog) = &mut self.confirmation_dialog {
            let result = dialog.handle_key(key);
            match result {
                DialogResult::Confirmed | DialogResult::Cancelled => {
                    Some(result)
                }
                DialogResult::Pending => Some(result),
            }
        } else {
            None
        }
    }

    /// Cancel the current confirmation dialog
    pub fn cancel_confirmation_dialog(&mut self) {
        self.confirmation_dialog = None;
        self.pending_action = None;
    }

    /// Check if a confirmation dialog is currently shown
    pub fn is_showing_confirmation_dialog(&self) -> bool {
        self.confirmation_dialog.is_some()
    }

    /// Confirm the pending action and execute it
    pub async fn confirm_pending_action(&mut self) -> Result<(), String> {
        if let Some(action) = self.pending_action.take() {
            self.confirmation_dialog = None;
            
            match action {
                PendingAction::DeleteProject(project_id) => {
                    self.delete_project_by_id(project_id).await
                }
            }
        } else {
            Err("No pending action to confirm".to_string())
        }
    }

    /// Delete a project by ID
    pub async fn delete_project_by_id(&mut self, project_id: String) -> Result<(), String> {
        // Find the project name for the success message
        let project_name = self.projects.iter()
            .find(|p| p.id == project_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Call the delete function from the projects library
        match delete_project(&project_id).await {
            Ok(true) => {
                // Project was deleted successfully
                // Refresh the projects list
                match orkee_projects::get_all_projects().await {
                    Ok(updated_projects) => {
                        self.set_projects(updated_projects);
                        self.add_system_message(format!("✅ **Project Deleted**\n\nSuccessfully deleted project \"{}\".", project_name));
                        
                        // Clear selection if the deleted project was selected
                        if let Some(selected_idx) = self.selected_project {
                            if selected_idx >= self.projects.len() {
                                // If the selected index is now out of bounds, clear or adjust selection
                                self.selected_project = if self.projects.is_empty() {
                                    None
                                } else {
                                    Some(self.projects.len().saturating_sub(1))
                                };
                            }
                        }
                        
                        Ok(())
                    }
                    Err(e) => {
                        Err(format!("Project deleted but failed to refresh list: {}", e))
                    }
                }
            }
            Ok(false) => {
                // Project was not found (already deleted?)
                Err(format!("Project \"{}\" was not found (may have been already deleted)", project_name))
            }
            Err(e) => {
                // Delete operation failed
                Err(format!("Failed to delete project \"{}\": {}", project_name, e))
            }
        }
    }
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