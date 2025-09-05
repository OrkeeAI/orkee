use crate::state::{AppState, EscapeAction, CtrlCAction, Screen};
use crate::events::{EventHandler, AppEvent};
use crate::ui;
use crate::slash_command::SlashCommand;
use crate::input::InputMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use orkee_projects::get_all_projects;
use ratatui::{backend::CrosstermBackend, Terminal};

/// Main TUI application struct
pub struct App {
    pub state: AppState,
    pub should_quit: bool,
    event_sender: Option<tokio::sync::mpsc::UnboundedSender<AppEvent>>,
}

impl App {
    pub fn new(refresh_interval: u64) -> Self {
        Self {
            state: AppState::new(refresh_interval),
            should_quit: false,
            event_sender: None,
        }
    }
    
    /// Load projects from local storage
    pub async fn load_projects(&mut self) -> Result<()> {
        match get_all_projects().await {
            Ok(projects) => {
                self.state.set_projects(projects);
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to load projects: {}", e))
        }
    }
    
    pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        let mut event_handler = EventHandler::new(250); // 250ms tick rate
        
        // Store the event sender for quit functionality
        self.event_sender = Some(event_handler.sender().clone());
        
        // Load projects on startup
        if let Err(e) = self.load_projects().await {
            self.state.add_system_message(format!("Warning: Failed to load projects: {}", e));
        }
        
        // Main event loop
        while !self.should_quit {
            // Render the UI
            terminal.draw(|frame| {
                ui::render(frame, &self.state);
            })?;
            
            // Handle events
            if let Some(event) = event_handler.next().await {
                let should_redraw = match event {
                    AppEvent::Key(key_event) => {
                        
                        if key_event.kind == KeyEventKind::Press {
                            self.handle_key_event(key_event).await?;
                            true // Redraw immediately after key events
                        } else {
                            false
                        }
                    }
                    AppEvent::Tick => {
                        // Handle periodic tasks
                        false // Tick doesn't need immediate redraw
                    }
                    AppEvent::Refresh => {
                        // Handle refresh requests
                        if let Err(e) = self.load_projects().await {
                            self.state.add_system_message(format!("Failed to refresh projects: {}", e));
                        }
                        true // Redraw after refresh
                    }
                    AppEvent::Quit => {
                        self.quit();
                        false
                    }
                };
                
                // Immediate redraw for input events
                if should_redraw {
                    terminal.draw(|frame| {
                        ui::render(frame, &self.state);
                    })?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle keyboard input
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let key = key_event.code;
        let modifiers = key_event.modifiers;
        
        // DEBUG: Trace all key events to see if Enter is being processed
        if self.state.is_form_mode() {
            self.state.add_system_message(format!("🔧 DEBUG: Key event - Code: {:?}, Modifiers: {:?}", key, modifiers));
        }
        
        // Handle dialog keys first (highest priority when dialog is shown)
        if self.state.is_showing_confirmation_dialog() {
            if let Some(result) = self.state.handle_dialog_key(key) {
                match result {
                    crate::ui::widgets::DialogResult::Confirmed => {
                        // User confirmed the action - execute it
                        match self.state.confirm_pending_action().await {
                            Ok(_) => {
                                // Action completed successfully - dialog is already closed
                            }
                            Err(error_msg) => {
                                // Action failed - show error and close dialog
                                self.state.cancel_confirmation_dialog();
                                self.state.add_system_message(format!("❌ **Error**\n\n{}", error_msg));
                            }
                        }
                    }
                    crate::ui::widgets::DialogResult::Cancelled => {
                        // User cancelled - just close the dialog
                        self.state.cancel_confirmation_dialog();
                    }
                    crate::ui::widgets::DialogResult::Pending => {
                        // Dialog is still waiting for input, do nothing
                    }
                }
            }
            // When dialog is shown, don't process any other keys
            return Ok(());
        }
        
        // Handle Ctrl+C (clear input on first press, quit on double press)
        if let KeyCode::Char('c') = key {
            if modifiers.contains(KeyModifiers::CONTROL) {
                let action = self.state.handle_ctrl_c_key();
                match action {
                    CtrlCAction::ClearInput => {
                        // Clear input buffer but keep focus
                        self.state.input_buffer_mut().clear();
                        
                        // Exit any special modes but stay in input
                        if self.state.is_mention_mode() {
                            self.state.exit_mention_mode();
                        } else if self.state.is_command_mode() {
                            self.state.exit_command_mode();
                        } else if self.state.is_editing_message() {
                            self.state.cancel_message_edit();
                        }
                        
                        // Cancel history navigation if active
                        self.state.cancel_history_navigation();
                    }
                    CtrlCAction::QuitApplication => {
                        // Quit the application
                        self.quit();
                    }
                }
                return Ok(());
            }
        }

        // Handle Ctrl+D for immediate quit
        if let KeyCode::Char('d') = key {
            if modifiers.contains(KeyModifiers::CONTROL) {
                self.quit();
                return Ok(());
            }
        }
        
        // Handle input-related keys when in input modes or with modifiers
        match key {
            // Text input keys
            KeyCode::Char(c) => {
                // Handle navigation shortcuts only when NOT in special modes
                if !self.state.is_command_mode() && !self.state.is_mention_mode() && 
                   !self.state.is_form_mode() && self.state.current_screen != crate::state::Screen::Chat {
                    match (c, &self.state.current_screen) {
                        ('d', &crate::state::Screen::Projects | &crate::state::Screen::ProjectDetail) => {
                            // Delete project on projects screen
                            if let Some(project) = self.state.get_selected_project() {
                                // Show delete confirmation dialog
                                let project_id = project.id.clone();
                                self.state.show_delete_confirmation(project_id);
                            } else {
                                self.state.add_system_message("❌ **No project selected**\n\nNavigate to projects screen and select a project first.".to_string());
                            }
                            return Ok(());
                        }
                        ('n', &crate::state::Screen::Projects) => {
                            // Start project creation form
                            self.state.start_project_creation();
                            return Ok(());
                        }
                        ('n', _) => {
                            self.state.current_screen = crate::state::Screen::Projects;
                            self.state.add_system_message("📁 **Switch to Projects Screen**\n\nPress 'n' again from the projects screen to create a new project.".to_string());
                            return Ok(());
                        }
                        ('e', &crate::state::Screen::Projects | &crate::state::Screen::ProjectDetail) => {
                            if let Some(project) = self.state.get_selected_project() {
                                // Start project editing form
                                let project_id = project.id.clone();
                                self.state.start_project_edit(project_id);
                            } else {
                                self.state.add_system_message("❌ **No project selected**\n\nNavigate to projects screen and select a project first.".to_string());
                            }
                            return Ok(());
                        }
                        ('q', _) => {
                            // Allow quit from any screen
                            self.quit();
                            return Ok(());
                        }
                        _ => {
                            // Not a navigation shortcut on this screen, continue with normal text input
                        }
                    }
                }
                
                // Allow quit from chat screen too (but only if input is empty and not in form mode)
                if c == 'q' && !self.state.is_command_mode() && !self.state.is_mention_mode() && 
                   !self.state.is_form_mode() && self.state.input_buffer().is_empty() && 
                   self.state.current_screen == crate::state::Screen::Chat {
                    self.quit();
                    return Ok(());
                }
                
                // Handle Ctrl+J (Shift+Enter in many terminals) specially
                if c == 'j' && modifiers.contains(KeyModifiers::CONTROL) {
                    if self.state.is_form_mode() {
                        // Form mode - handle Shift+Enter for multiline fields
                        if self.state.form_current_field_is_multiline() {
                            if let Some(form) = self.state.form_mut() {
                                form.insert_newline();
                            }
                        }
                        return Ok(());
                    } else if self.state.is_input_focused() {
                        // Chat input - add newline
                        self.state.input_buffer_mut().insert_char('\n');
                        return Ok(());
                    }
                }
                
                // Handle Shift+C (cancel form) - BEFORE routing to form widget
                if c == 'C' && modifiers.contains(KeyModifiers::SHIFT) {
                    if self.state.is_form_mode() {
                        // Cancel form and return to projects list
                        self.state.cancel_form();
                        return Ok(());
                    }
                }
                
                // Handle Shift+F (open search popup) - Project search
                if c == 'F' && modifiers.contains(KeyModifiers::SHIFT) {
                    if matches!(self.state.current_screen, Screen::Projects | Screen::ProjectDetail) {
                        if self.state.is_search_mode() {
                            // Close search if already open
                            self.state.close_search();
                            self.state.input_mode = InputMode::Normal;
                        } else {
                            // Open search popup
                            self.state.open_project_search();
                            self.state.input_mode = InputMode::ProjectSearch;
                        }
                        return Ok(());
                    }
                }
                
                // Global shortcuts are now handled above in the navigation logic
                
                // Only process text input if input area is focused (true Codex behavior)
                // EXCEPT in form mode where we handle input directly
                if !self.state.is_input_focused() && !self.state.is_form_mode() {
                    // If chat is focused, ignore typing - only Tab switches focus
                    return Ok(());
                }
                
                // Handle input based on mode
                if self.state.is_form_mode() {
                    // For review step, don't route keys to form widget since there are no fields
                    if !self.state.form_is_review_step() {
                        // Form mode - route input to form widget only if not on review step
                        let crossterm_event = crossterm::event::KeyEvent::new(key, modifiers);
                        let event = crossterm::event::Event::Key(crossterm_event);
                        if let Some(form) = self.state.form_mut() {
                            form.handle_input(&event);
                        }
                    }
                    // Note: Review step has no input fields, so we don't send keys to form widget
                } else if self.state.input_mode == InputMode::ProjectSearch {
                    // Project search mode - handle input based on current search mode
                    if let Some(ref mut search_popup) = self.state.search_popup {
                        match search_popup.search_mode() {
                            crate::search_popup::SearchMode::Text => {
                                // Text mode - normal character input
                                search_popup.handle_char(c);
                            }
                            crate::search_popup::SearchMode::Status => {
                                // Status filter mode - handle status selection keys
                                match c {
                                    '1' | 'a' => {
                                        search_popup.toggle_status_filter(Some(orkee_projects::ProjectStatus::Active));
                                    }
                                    '2' | 'r' => {
                                        search_popup.toggle_status_filter(Some(orkee_projects::ProjectStatus::Archived));
                                    }
                                    '0' | 'c' => {
                                        search_popup.toggle_status_filter(None);
                                    }
                                    ' ' => {
                                        // Space key - cycle through status options
                                        let current_status = search_popup.get_status_filter();
                                        let next_status = match current_status {
                                            None => Some(orkee_projects::ProjectStatus::Active),
                                            Some(orkee_projects::ProjectStatus::Active) => Some(orkee_projects::ProjectStatus::Archived),
                                            Some(orkee_projects::ProjectStatus::Archived) => None,
                                        };
                                        search_popup.toggle_status_filter(next_status);
                                    }
                                    _ => {
                                        // Other keys - ignore in status mode
                                    }
                                }
                            }
                            crate::search_popup::SearchMode::Priority => {
                                // Priority filter mode - handle priority selection keys
                                match c {
                                    '1' | 'h' => {
                                        search_popup.toggle_priority_filter(Some(orkee_projects::Priority::High));
                                    }
                                    '2' | 'm' => {
                                        search_popup.toggle_priority_filter(Some(orkee_projects::Priority::Medium));
                                    }
                                    '3' | 'l' => {
                                        search_popup.toggle_priority_filter(Some(orkee_projects::Priority::Low));
                                    }
                                    '0' | 'c' => {
                                        search_popup.toggle_priority_filter(None);
                                    }
                                    ' ' => {
                                        // Space key - cycle through priority options
                                        let current_priority = search_popup.get_priority_filter();
                                        let next_priority = match current_priority {
                                            None => Some(orkee_projects::Priority::High),
                                            Some(orkee_projects::Priority::High) => Some(orkee_projects::Priority::Medium),
                                            Some(orkee_projects::Priority::Medium) => Some(orkee_projects::Priority::Low),
                                            Some(orkee_projects::Priority::Low) => None,
                                        };
                                        search_popup.toggle_priority_filter(next_priority);
                                    }
                                    _ => {
                                        // Other keys - ignore in priority mode
                                    }
                                }
                            }
                            crate::search_popup::SearchMode::Tags => {
                                // Tags mode - normal character input for tag entry
                                search_popup.handle_char(c);
                            }
                        }
                        // Text input and tag input use debounced updates (handled in invalidate_cache)
                        // Filter changes use immediate updates
                        match search_popup.search_mode() {
                            crate::search_popup::SearchMode::Status | 
                            crate::search_popup::SearchMode::Priority => {
                                // Filter changes - immediate update
                                search_popup.force_search_update(&self.state.projects);
                            }
                            _ => {
                                // Text/tag input - debounced update already requested
                                // Check if enough time has passed for debounced update
                                if search_popup.should_update_search() {
                                    self.state.update_search();
                                }
                            }
                        }
                    }
                } else {
                    // Normal mode - add character to input buffer
                    self.state.input_buffer_mut().insert_char(c);
                }
                
                // Check if we just typed '/' and should enter command mode (use original char, not transformed)
                if c == '/' && self.state.input_buffer().content() == "/" && !self.state.is_command_mode() {
                    self.state.enter_command_mode();
                } else if self.state.is_command_mode() {
                    // Update command filter as we type
                    self.state.update_command_filter();
                }
                
                // Check if we just typed '@' and should enter mention mode (use original char, not transformed)
                if c == '@' && !self.state.is_mention_mode() && !self.state.is_command_mode() {
                    let cursor_pos = self.state.input_buffer().cursor_position();
                    let char_pos = cursor_pos - c.len_utf8(); // Position where @ was inserted
                    
                    // Only trigger mention mode if @ is at start or preceded by whitespace
                    if self.state.should_trigger_mention(char_pos) {
                        self.state.enter_mention_mode(char_pos);
                    }
                } else if self.state.is_mention_mode() {
                    // Update mention filter as we type
                    self.state.update_mention_filter();
                }
            }
            
            // Input editing keys
            KeyCode::Backspace => {
                // Only process if input area is focused
                // EXCEPT in form mode where we handle input directly
                if !self.state.is_input_focused() && !self.state.is_form_mode() {
                    // If chat is focused, ignore backspace - only Tab switches focus
                    return Ok(());
                }
                
                if self.state.is_form_mode() {
                    // Form mode - route backspace to form widget
                    let crossterm_event = crossterm::event::KeyEvent::new(key, modifiers);
                    let event = crossterm::event::Event::Key(crossterm_event);
                    if let Some(form) = self.state.form_mut() {
                        form.handle_input(&event);
                    }
                } else if self.state.input_mode == InputMode::ProjectSearch {
                    // Project search mode - handle backspace based on current search mode
                    if let Some(ref mut search_popup) = self.state.search_popup {
                        match search_popup.search_mode() {
                            crate::search_popup::SearchMode::Text => {
                                // Text mode - normal backspace handling
                                search_popup.handle_backspace();
                            }
                            crate::search_popup::SearchMode::Status => {
                                // Status mode - clear current status filter
                                search_popup.toggle_status_filter(None);
                            }
                            crate::search_popup::SearchMode::Priority => {
                                // Priority mode - clear current priority filter
                                search_popup.toggle_priority_filter(None);
                            }
                            crate::search_popup::SearchMode::Tags => {
                                // Tags mode - handle backspace in tag input or remove last tag
                                if search_popup.search_query().is_empty() {
                                    // If input is empty, remove last tag filter
                                    let tag_filters = search_popup.get_tag_filters().clone();
                                    if let Some(last_tag) = tag_filters.last() {
                                        let last_tag = last_tag.clone();
                                        search_popup.remove_tag_filter(&last_tag);
                                    }
                                } else {
                                    // If input is not empty, normal backspace
                                    search_popup.handle_backspace();
                                }
                            }
                        }
                        // Handle debounced vs immediate updates for backspace
                        match search_popup.search_mode() {
                            crate::search_popup::SearchMode::Status | 
                            crate::search_popup::SearchMode::Priority => {
                                // Filter changes - immediate update
                                search_popup.force_search_update(&self.state.projects);
                            }
                            _ => {
                                // Text/tag input - debounced update already requested
                                if search_popup.should_update_search() {
                                    self.state.update_search();
                                }
                            }
                        }
                    }
                } else {
                    self.state.input_buffer_mut().backspace();
                }
                
                // Exit command mode if we deleted the '/' 
                if self.state.is_command_mode() {
                    let content = self.state.input_buffer().content();
                    if !content.starts_with('/') {
                        self.state.exit_command_mode();
                    } else {
                        // Update filter as we delete characters
                        self.state.update_command_filter();
                    }
                } else if self.state.is_mention_mode() {
                    // Check if we deleted the @ or if cursor moved before the mention start
                    let content = self.state.input_buffer().content();
                    let cursor_pos = self.state.input_buffer().cursor_position();
                    
                    if let Some(popup) = self.state.mention_popup() {
                        let mention_start = popup.mention_start_position();
                        
                        // Exit mention mode if we deleted the @ or cursor is before it
                        if mention_start >= content.len() || 
                           cursor_pos < mention_start ||
                           !content.chars().nth(mention_start).map_or(false, |c| c == '@') {
                            self.state.exit_mention_mode();
                        } else {
                            // Update mention filter as we delete characters
                            self.state.update_mention_filter();
                        }
                    }
                }
            }
            KeyCode::Delete => {
                // Only process if input area is focused
                // EXCEPT in form mode where we handle input directly
                if !self.state.is_input_focused() && !self.state.is_form_mode() {
                    // If chat is focused, ignore delete - only Tab switches focus
                    return Ok(());
                }
                
                if self.state.is_form_mode() {
                    // Form mode - route delete to form widget
                    let crossterm_event = crossterm::event::KeyEvent::new(key, modifiers);
                    let event = crossterm::event::Event::Key(crossterm_event);
                    if let Some(form) = self.state.form_mut() {
                        form.handle_input(&event);
                    }
                } else {
                    self.state.input_buffer_mut().delete_char();
                }
                
                // Update command filter if in command mode
                if self.state.is_command_mode() {
                    self.state.update_command_filter();
                } else if self.state.is_mention_mode() {
                    // Update mention filter if in mention mode
                    self.state.update_mention_filter();
                }
            }
            
            // Cursor movement keys (only work when input is focused or in form mode)
            KeyCode::Left => {
                if !self.state.is_input_focused() && !self.state.is_form_mode() {
                    // If chat is focused, ignore cursor movement - only Tab switches focus
                    return Ok(());
                }
                if !self.state.is_form_mode() {
                    self.state.input_buffer_mut().move_left();
                }
                // TODO: Add cursor movement within form fields if needed
            }
            KeyCode::Right => {
                if !self.state.is_input_focused() && !self.state.is_form_mode() {
                    // If chat is focused, ignore cursor movement - only Tab switches focus
                    return Ok(());
                }
                if !self.state.is_form_mode() {
                    self.state.input_buffer_mut().move_right();
                }
                // TODO: Add cursor movement within form fields if needed
            }
            KeyCode::Home => {
                if !self.state.is_input_focused() && !self.state.is_form_mode() {
                    // If chat is focused, ignore Home key - only Tab switches focus
                    return Ok(());
                }
                if !self.state.is_form_mode() {
                    if self.state.input_buffer().is_empty() {
                        // Scroll to bottom of chat if no input
                        self.state.scroll_to_bottom();
                    } else {
                        // Move cursor to start of input
                        self.state.input_buffer_mut().move_to_start();
                    }
                }
                // TODO: Add cursor movement within form fields if needed
            }
            KeyCode::End => {
                if !self.state.is_input_focused() && !self.state.is_form_mode() {
                    // If chat is focused, ignore End key - only Tab switches focus
                    return Ok(());
                }
                if !self.state.is_form_mode() {
                    self.state.input_buffer_mut().move_to_end();
                }
                // TODO: Add cursor movement within form fields if needed
            }
            
            // Up/Down navigation: Popups > Form fields > Projects > Default scroll behavior (unless chat focused)
            KeyCode::Up => {
                if self.state.is_mention_mode() {
                    // Navigate mention popup (always forces input focus)
                    self.state.mention_popup_up();
                } else if self.state.is_command_mode() {
                    // Navigate command popup (always forces input focus)
                    self.state.command_popup_up();
                } else if self.state.input_mode == InputMode::ProjectSearch {
                    // Navigate search results up
                    if let Some(ref mut search_popup) = self.state.search_popup {
                        search_popup.select_previous();
                    }
                } else if self.state.is_form_mode() {
                    // Form mode - try to let form widget handle it first (for Selection fields)
                    let crossterm_event = crossterm::event::KeyEvent::new(key, modifiers);
                    let event = crossterm::event::Event::Key(crossterm_event);
                    let mut handled = false;
                    if let Some(form) = self.state.form_mut() {
                        handled = form.handle_input(&event);
                    }
                    
                    // If form widget didn't handle it, use it for field navigation
                    if !handled {
                        self.handle_form_navigation(false).await;
                    }
                } else if self.state.current_screen == crate::state::Screen::Projects {
                    // Projects screen - navigate project list
                    self.state.select_previous_project();
                } else if self.state.is_chat_focused() {
                    // Chat is focused - do NOT scroll, maybe select individual messages in future
                    // For now, do nothing when chat area has focus
                } else if self.state.input_mode() == &InputMode::History {
                    // Input focused and in history mode - try history navigation first
                    if !self.state.navigate_history_previous() {
                        // No more history, scroll chat messages as fallback
                        self.state.scroll_up();
                    }
                } else if self.state.is_input_focused() && self.state.input_buffer().is_empty() {
                    // Input focused with empty buffer - try history navigation first
                    if !self.state.navigate_history_previous() {
                        // No history available, scroll chat messages
                        self.state.scroll_up();
                    }
                } else {
                    // Default behavior: scroll chat messages
                    self.state.scroll_up();
                }
            }
            KeyCode::Down => {
                if self.state.is_mention_mode() {
                    // Navigate mention popup (always forces input focus)
                    self.state.mention_popup_down();
                } else if self.state.is_command_mode() {
                    // Navigate command popup (always forces input focus)
                    self.state.command_popup_down();
                } else if self.state.input_mode == InputMode::ProjectSearch {
                    // Navigate search results down
                    if let Some(ref mut search_popup) = self.state.search_popup {
                        search_popup.select_next();
                    }
                } else if self.state.is_form_mode() {
                    // Form mode - try to let form widget handle it first (for Selection fields)
                    let crossterm_event = crossterm::event::KeyEvent::new(key, modifiers);
                    let event = crossterm::event::Event::Key(crossterm_event);
                    let mut handled = false;
                    if let Some(form) = self.state.form_mut() {
                        handled = form.handle_input(&event);
                    }
                    
                    // If form widget didn't handle it, use it for field navigation (forward)
                    if !handled {
                        self.handle_form_navigation(true).await;
                    }
                } else if self.state.current_screen == crate::state::Screen::Projects {
                    // Projects screen - navigate project list
                    self.state.select_next_project();
                } else if self.state.is_chat_focused() {
                    // Chat is focused - do NOT scroll, maybe select individual messages in future
                    // For now, do nothing when chat area has focus
                } else if self.state.input_mode() == &InputMode::History {
                    // Input focused and in history mode
                    if !self.state.navigate_history_next() {
                        // Reached end of history, scroll chat messages
                        self.state.scroll_down();
                    }
                } else {
                    // Default behavior: scroll chat messages
                    self.state.scroll_down();
                }
            }
            
            // Submit message or complete mention/command (only when input focused)
            KeyCode::Enter => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    // Shift+Enter - add newline in multiline fields only
                    if self.state.is_form_mode() {
                        // Form mode - handle Shift+Enter specially for multiline fields
                        if self.state.form_current_field_is_multiline() {
                            // Direct newline insertion like chat input
                            if let Some(form) = self.state.form_mut() {
                                form.insert_newline();
                            }
                        }
                    } else if self.state.is_input_focused() {
                        self.state.input_buffer_mut().insert_char('\n');
                    }
                } else if self.state.is_form_mode() {
                    // Form mode - Enter always advances to next field or submits
                    if self.state.form_is_review_step() {
                        // On review step, Enter should always submit
                        if self.state.form_can_submit() {
                            match self.state.submit_form().await {
                                Ok(_) => {
                                    // Success - form was submitted
                                }
                                Err(error_msg) => {
                                    // Show error message
                                    self.state.add_system_message(error_msg);
                                }
                            }
                        } else {
                            self.state.add_system_message("❌ **Form Incomplete**\n\nPlease fill in all required fields before submitting.".to_string());
                        }
                    } else {
                        // Not on review step - use normal navigation
                        if !self.handle_form_navigation(true).await {
                            // Form validation failed or not ready for submission
                            if !self.state.form_can_submit() {
                                self.state.add_system_message("❌ **Form Incomplete**\n\nPlease fill in all required fields before submitting.".to_string());
                            }
                        }
                    }
                } else if self.state.is_mention_mode() {
                    // Complete selected mention (always works in mention mode)
                    if let Some(_completed_mention) = self.state.complete_selected_mention() {
                        // Mention was completed, continue typing
                    }
                } else if self.state.input_mode == InputMode::ProjectSearch {
                    // Handle Enter based on current search mode
                    if let Some(ref mut search_popup) = self.state.search_popup {
                        match search_popup.search_mode() {
                            crate::search_popup::SearchMode::Tags => {
                                // Tags mode - add current text as a tag filter if not empty
                                let current_query = search_popup.search_query().trim().to_string();
                                if !current_query.is_empty() {
                                    search_popup.add_tag_filter(current_query);
                                    search_popup.clear_input();
                                    // Tag addition should update immediately
                                    search_popup.force_search_update(&self.state.projects);
                                }
                            }
                            _ => {
                                // Other modes - select project from search results
                                if let Some(_project_index) = self.state.select_search_result() {
                                    // Navigate to project detail view
                                    if matches!(self.state.current_screen, Screen::Projects) {
                                        self.state.view_selected_project_details();
                                    }
                                }
                            }
                        }
                    }
                } else if self.state.current_screen == crate::state::Screen::Projects && !self.state.is_form_mode() {
                    // Projects screen - view selected project details
                    self.state.view_selected_project_details();
                } else if !self.state.is_input_focused() && !self.state.is_form_mode() {
                    // If chat is focused, ignore Enter - only Tab switches focus
                    return Ok(());
                } else {
                    // Input is focused - handle submission
                    self.handle_input_submission().await;
                }
            }
            
            // Cancel/escape or double-escape for editing
            KeyCode::Esc => {
                // Handle double-escape detection first
                match self.state.handle_escape_key() {
                    EscapeAction::EditPreviousMessage => {
                        // Double escape detected - load previous message for editing
                        if !self.state.load_previous_message_for_edit() {
                            // No previous message to edit
                            self.state.add_system_message("No previous message to edit".to_string());
                        }
                    }
                    EscapeAction::SingleEscape => {
                        // Handle single escape based on current mode
                        if self.state.is_form_mode() {
                            if self.state.form_is_review_step() {
                                // On review step: go back to edit instead of cancelling
                                self.state.form_previous_step();
                            } else {
                                // Cancel form and return to projects list
                                self.state.cancel_form();
                            }
                        } else if self.state.is_editing_message() {
                            // Cancel edit mode
                            self.state.cancel_message_edit();
                        } else if self.state.is_mention_mode() {
                            // Exit mention mode
                            self.state.exit_mention_mode();
                        } else if self.state.is_command_mode() {
                            // Exit command mode
                            self.state.exit_command_mode();
                        } else if self.state.input_mode == InputMode::ProjectSearch {
                            // Close search popup and return to normal mode
                            self.state.close_search();
                            self.state.input_mode = InputMode::Normal;
                        } else if self.state.current_screen == crate::state::Screen::ProjectDetail {
                            // Return to projects list from detail view
                            self.state.return_to_projects_list();
                        } else if self.state.current_screen == crate::state::Screen::Projects {
                            // Return to chat from projects list
                            self.state.current_screen = crate::state::Screen::Chat;
                        } else if !self.state.cancel_history_navigation() {
                            // If not canceling history, clear input buffer
                            self.state.input_buffer_mut().clear();
                        }
                    }
                }
            }
            
            // Tab for focus cycling (Codex-style) or completion or form navigation
            KeyCode::Tab => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    // Shift+Tab - previous field in form or focus cycling
                    if self.state.is_form_mode() {
                        // Special handling for review step or normal field navigation
                        self.handle_form_navigation(false).await;
                    } else {
                        // Reverse cycle focus for shift+tab (not typically used but consistent)
                        self.state.cycle_focus();
                    }
                } else if self.state.is_form_mode() {
                    // Tab in form mode - try to advance to next field or submit
                    if !self.handle_form_navigation(true).await {
                        // Form validation failed or not ready for submission
                        if !self.state.form_can_submit() {
                            self.state.add_system_message("❌ **Form Incomplete**\n\nPlease fill in all required fields before submitting.".to_string());
                        }
                    }
                } else if self.state.is_mention_mode() {
                    // Complete selected mention
                    if let Some(_completed_mention) = self.state.complete_selected_mention() {
                        // Mention was completed, continue typing
                    }
                } else if self.state.is_command_mode() {
                    // Complete selected command
                    if let Some(_completed_command) = self.state.complete_selected_command() {
                        // Command was completed, cursor is positioned for arguments if needed
                    }
                } else if self.state.input_mode == InputMode::ProjectSearch {
                    // Cycle search modes in search popup (Text -> Status -> Priority -> Tags)
                    if let Some(ref mut search_popup) = self.state.search_popup {
                        search_popup.cycle_search_mode();
                        // Mode changes should update immediately
                        search_popup.force_search_update(&self.state.projects);
                    }
                } else {
                    // Cycle focus between chat and input areas (Codex behavior)
                    self.state.cycle_focus();
                }
            }
            
            // Other keys are ignored for now
            _ => {}
        }
        
        Ok(())
    }
    
    
    /// Handle input submission (Enter key)
    async fn handle_input_submission(&mut self) {
        if self.state.is_command_mode() {
            // In command mode, complete the selected command or execute if already complete
            let input_content = self.state.input_buffer().content().to_string();
            
            if let Some(_completed_command) = self.state.complete_selected_command() {
                // Command was completed, check if it needs execution
                if !input_content.contains('<') {
                    // No argument placeholders, execute the command
                    self.execute_slash_command().await;
                }
            } else {
                // No command selected, try to execute what we have
                self.execute_slash_command().await;
            }
        } else if self.state.submit_current_input() {
            // Normal message submission
            // In future phases, this is where we'd send to server
        }
    }
    
    /// Centralized form navigation logic
    async fn handle_form_navigation(&mut self, try_advance: bool) -> bool {
        // Special handling for review step
        if self.state.form_is_review_step() {
            if try_advance {
                // Enter/Tab/Down on review step should submit the form
                if self.state.form_can_submit() {
                    match self.state.submit_form().await {
                        Ok(_) => {
                            // Success message already added by submit_form
                            return true;
                        }
                        Err(error_msg) => {
                            // Show error message
                            self.state.add_system_message(error_msg);
                            return false;
                        }
                    }
                } else {
                    self.state.add_system_message("❌ **Form Incomplete**\n\nPlease fill in all required fields before submitting.".to_string());
                    return false;
                }
            } else {
                // Up arrow/Shift+Tab on review step should not do anything
                // Use Escape to go back to edit
                return false;
            }
        }
        
        if try_advance {
            // Forward navigation - validate current field first
            if !self.state.form_validate_current_field() {
                // Validation failed, stay on current field
                return false;
            }
            
            // Try to move to next field
            if self.state.form_next_field() {
                // Successfully moved to next field
                return true;
            }
        } else {
            // Backward navigation - don't validate current field, allow going back
            if self.state.form_previous_field() {
                // Successfully moved to previous field
                // Validate the new current field (where we moved to)
                self.state.form_validate_current_field();
                return true;
            } else {
                // At first field - don't do anything special
                return false;
            }
        }
        
        // Either not trying to advance, or on last field - try to submit if form is complete
        if self.state.form_can_submit() {
            match self.state.submit_form().await {
                Ok(_) => {
                    // Success message already added by submit_form
                    return true;
                }
                Err(error_msg) => {
                    // Show error message
                    self.state.add_system_message(error_msg);
                    return false;
                }
            }
        }
        
        // Form is not ready for submission
        false
    }
    
    /// Execute a slash command from the input buffer
    async fn execute_slash_command(&mut self) {
        let input_content = self.state.input_buffer().content().to_string();
        
        // Parse the command from input
        match SlashCommand::parse_from_input(&input_content) {
            Ok((command, _args)) => {
                // Clear input buffer and exit command mode
                self.state.input_buffer_mut().clear();
                self.state.exit_command_mode();
                
                // Execute the command
                match command {
                    SlashCommand::Help => {
                        let content = "📚 **Help - Orkee TUI**\n\n**Slash Commands:**\n- `/help` - Show this help\n- `/quit` - Exit the application\n- `/clear` - Clear chat history\n- `/projects` - Open interactive projects screen\n- `/status` - Show application status\n\n**Projects Screen Navigation:**\n- `↑↓` - Navigate project list\n- `Enter` - View project details\n- `Esc` - Return to chat (or projects list from details)\n- `n` - New project • `e` - Edit • `d` - Delete\n\n**Command System:**\n- Type `/` to open command popup\n- `↑↓` - Navigate commands\n- `Tab/Enter` - Complete/execute command\n- `Esc` - Cancel command mode\n\n**Text Input:**\n- `Enter` - Submit message\n- `↑↓` - Navigate input history (when input empty)\n- `Tab` - Switch focus (chat ↔ input)\n- `q` - Quick quit (when input empty)".to_string();
                        self.state.add_system_message(content);
                    }
                    SlashCommand::Quit => {
                        self.state.add_system_message("👋 Goodbye! Exiting Orkee TUI...".to_string());
                        self.quit();
                    }
                    SlashCommand::Clear => {
                        // Clear message history but keep welcome message
                        self.state.message_history.clear();
                        self.state.add_system_message("🧹 Chat history cleared.".to_string());
                    }
                    SlashCommand::Projects => {
                        // Switch to interactive projects screen
                        if let Err(e) = self.load_projects().await {
                            self.state.add_system_message(format!("⚠️ Failed to load projects: {}", e));
                        } else {
                            self.state.current_screen = crate::state::Screen::Projects;
                            // Ensure we have a selection if there are projects
                            if !self.state.projects.is_empty() && self.state.selected_project.is_none() {
                                self.state.selected_project = Some(0);
                            }
                        }
                    }
                    SlashCommand::Status => {
                        let content = format!("📊 **Application Status**\n\n**Projects:** {} loaded\n**Current Screen:** {:?}\n**Input Mode:** {:?}\n**Refresh Interval:** {}s\n**Command System:** ✅ Active (Phase 3)\n\n**Features:**\n- ✅ Slash commands with popup\n- ✅ Fuzzy command matching\n- ✅ Input history navigation\n- ✅ Chat message system\n\n💡 *All systems operational!*", 
                            self.state.projects.len(),
                            self.state.current_screen,
                            self.state.input_mode(),
                            self.state.refresh_interval
                        );
                        self.state.add_system_message(content);
                    }
                }
            }
            Err(error) => {
                // Show error message and clear input
                self.state.add_system_message(format!("❌ **Command Error:** {}\n\n💡 *Type `/help` for available commands*", error));
                self.state.input_buffer_mut().clear();
                self.state.exit_command_mode();
            }
        }
    }
    
    pub fn quit(&mut self) {
        self.should_quit = true;
        
        // Send quit event to break the event loop immediately
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(AppEvent::Quit);
        }
    }
}