use crate::state::{AppState, EscapeAction};
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
}

impl App {
    pub fn new(refresh_interval: u64) -> Self {
        Self {
            state: AppState::new(refresh_interval),
            should_quit: false,
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
                match event {
                    AppEvent::Key(key_event) => {
                        if key_event.kind == KeyEventKind::Press {
                            self.handle_key_event(key_event).await?;
                        }
                    }
                    AppEvent::Tick => {
                        // Handle periodic tasks
                    }
                    AppEvent::Refresh => {
                        // Handle refresh requests
                        if let Err(e) = self.load_projects().await {
                            self.state.add_system_message(format!("Failed to refresh projects: {}", e));
                        }
                    }
                    AppEvent::Quit => {
                        self.quit();
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle keyboard input
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let key = key_event.code;
        let modifiers = key_event.modifiers;
        
        // Handle Ctrl+C to clear input buffer
        if let KeyCode::Char('c') = key {
            if modifiers.contains(KeyModifiers::CONTROL) {
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
                
                return Ok(());
            }
        }
        
        // Handle input-related keys when in input modes or with modifiers
        match key {
            // Text input keys
            KeyCode::Char(c) => {
                // Check for global shortcuts first (only if input buffer is empty and not in command mode)
                if self.is_global_shortcut(c) && self.state.input_buffer().is_empty() && !self.state.is_command_mode() {
                    return self.handle_global_shortcut(c).await;
                }
                
                // Add character to input buffer
                self.state.input_buffer_mut().insert_char(c);
                
                // Check if we just typed '/' and should enter command mode
                if c == '/' && self.state.input_buffer().content() == "/" && !self.state.is_command_mode() {
                    self.state.enter_command_mode();
                } else if self.state.is_command_mode() {
                    // Update command filter as we type
                    self.state.update_command_filter();
                }
                
                // Check if we just typed '@' and should enter mention mode
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
                self.state.input_buffer_mut().backspace();
                
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
                self.state.input_buffer_mut().delete_char();
                
                // Update command filter if in command mode
                if self.state.is_command_mode() {
                    self.state.update_command_filter();
                } else if self.state.is_mention_mode() {
                    // Update mention filter if in mention mode
                    self.state.update_mention_filter();
                }
            }
            
            // Cursor movement keys
            KeyCode::Left => {
                self.state.input_buffer_mut().move_left();
            }
            KeyCode::Right => {
                self.state.input_buffer_mut().move_right();
            }
            KeyCode::Home => {
                if self.state.input_buffer().is_empty() {
                    // Scroll to bottom of chat if no input
                    self.state.scroll_to_bottom();
                } else {
                    // Move cursor to start of input
                    self.state.input_buffer_mut().move_to_start();
                }
            }
            KeyCode::End => {
                self.state.input_buffer_mut().move_to_end();
            }
            
            // Up/Down navigation: Popups > Chat scrolling > History (only when input empty)
            KeyCode::Up => {
                if self.state.is_mention_mode() {
                    // Navigate mention popup
                    self.state.mention_popup_up();
                } else if self.state.is_command_mode() {
                    // Navigate command popup
                    self.state.command_popup_up();
                } else if self.state.input_mode() == &InputMode::History {
                    // Already in history mode, continue navigating
                    if !self.state.navigate_history_previous() {
                        // No more history, scroll chat instead
                        self.state.scroll_up();
                    }
                } else if self.state.input_buffer().is_empty() {
                    // Empty input - try history navigation, otherwise scroll chat
                    if !self.state.navigate_history_previous() {
                        self.state.scroll_up();
                    }
                } else {
                    // Input has content - scroll chat
                    self.state.scroll_up();
                }
            }
            KeyCode::Down => {
                if self.state.is_mention_mode() {
                    // Navigate mention popup
                    self.state.mention_popup_down();
                } else if self.state.is_command_mode() {
                    // Navigate command popup
                    self.state.command_popup_down();
                } else if self.state.input_mode() == &InputMode::History {
                    // Already in history mode, continue navigating
                    if !self.state.navigate_history_next() {
                        // Reached end of history, scroll chat instead
                        self.state.scroll_down();
                    }
                } else {
                    // Always scroll chat for Down arrow (history is typically Up-only)
                    self.state.scroll_down();
                }
            }
            
            // Submit message or complete mention/command
            KeyCode::Enter => {
                if self.state.is_mention_mode() {
                    // Complete selected mention
                    if let Some(_completed_mention) = self.state.complete_selected_mention() {
                        // Mention was completed, continue typing
                    }
                } else {
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
                        if self.state.is_editing_message() {
                            // Cancel edit mode
                            self.state.cancel_message_edit();
                        } else if self.state.is_mention_mode() {
                            // Exit mention mode
                            self.state.exit_mention_mode();
                        } else if self.state.is_command_mode() {
                            // Exit command mode
                            self.state.exit_command_mode();
                        } else if !self.state.cancel_history_navigation() {
                            // If not canceling history, clear input buffer
                            self.state.input_buffer_mut().clear();
                        }
                    }
                }
            }
            
            // Tab for mention/command completion or screen switching
            KeyCode::Tab => {
                if self.state.is_mention_mode() {
                    // Complete selected mention
                    if let Some(_completed_mention) = self.state.complete_selected_mention() {
                        // Mention was completed, continue typing
                    }
                } else if self.state.is_command_mode() {
                    // Complete selected command
                    if let Some(_completed_command) = self.state.complete_selected_command() {
                        // Command was completed, cursor is positioned for arguments if needed
                    }
                } else if self.state.input_buffer().is_empty() {
                    self.state.next_screen();
                }
                // TODO: Could implement other tab completion here in future
            }
            
            // Other keys are ignored for now
            _ => {}
        }
        
        Ok(())
    }
    
    /// Check if a character is a global shortcut
    fn is_global_shortcut(&self, c: char) -> bool {
        matches!(c, 'q' | 'h' | 'd' | 'p' | 's')
    }
    
    /// Handle global shortcuts (only when input buffer is empty)
    async fn handle_global_shortcut(&mut self, c: char) -> Result<()> {
        match c {
            'q' => {
                self.quit();
            }
            'h' => {
                // Show help message
                let content = "📚 **Help - Orkee TUI (Phase 2)**\n\n**Text Input:**\n- Type to enter text\n- `Enter` - Submit message\n- `↑/↓` - Navigate input history (when input empty)\n- `Esc` - Clear input or cancel history\n\n**Cursor Movement:**\n- `←/→` - Move cursor\n- `Home/End` - Start/end of input\n- `Ctrl+←/→` - Word movement (coming soon)\n\n**Navigation:**\n- `Tab` - Switch screens (when input empty)\n- `↑/↓` - Scroll messages (when not in history)\n\n**Commands (only when input empty):**\n- `d` - Show dashboard\n- `p` - List projects\n- `s` - Show settings\n- `h` - This help\n- `q` - Quit\n\n💡 *Slash commands coming in Phase 3!*".to_string();
                self.state.add_system_message(content);
            }
            'd' => {
                // Show dashboard info as chat message
                let content = format!(
                    "📊 **Dashboard Status**\n\nProjects: {}\nCurrent Screen: {:?}\nRefresh Interval: {}s\nInput Mode: {:?}\n\n💡 *Now with input system! Type a message to try it.*",
                    self.state.projects.len(),
                    self.state.current_screen,
                    self.state.refresh_interval,
                    self.state.input_mode()
                );
                self.state.add_system_message(content);
            }
            'p' => {
                // Show projects as chat message
                if self.state.projects.is_empty() {
                    self.state.add_system_message("📁 **Projects**\n\nNo projects found.\n\n💡 *Tip: Use the CLI to add projects: `orkee projects add`*".to_string());
                } else {
                    let mut content = String::from("📁 **Projects**\n\n");
                    for (i, project) in self.state.projects.iter().enumerate() {
                        let status = format!("{:?}", project.status).to_lowercase();
                        content.push_str(&format!("{}. **{}** ({})\n", i + 1, project.name, status));
                        if let Some(description) = &project.description {
                            if !description.is_empty() {
                                content.push_str(&format!("   └─ {}\n", description));
                            }
                        }
                        content.push_str(&format!("   📂 {}\n\n", project.project_root));
                    }
                    self.state.add_system_message(content);
                }
            }
            's' => {
                // Show settings info as chat message
                let content = format!("⚙️ **Settings**\n\nRefresh Interval: {}s\nCurrent Theme: Dark\nInput Buffer Length: {} chars\nInput History: {} entries\n\n💡 *Settings management coming in future phases*", 
                    self.state.refresh_interval,
                    self.state.input_buffer().len(),
                    self.state.input_history.len()
                );
                self.state.add_system_message(content);
            }
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
    
    /// Execute a slash command from the input buffer
    async fn execute_slash_command(&mut self) {
        let input_content = self.state.input_buffer().content().to_string();
        
        // Parse the command from input
        match SlashCommand::parse_from_input(&input_content) {
            Ok((command, args)) => {
                // Clear input buffer and exit command mode
                self.state.input_buffer_mut().clear();
                self.state.exit_command_mode();
                
                // Execute the command
                match command {
                    SlashCommand::Help => {
                        let content = "📚 **Help - Orkee TUI (Phase 3)**\n\n**Slash Commands:**\n- `/help` - Show this help\n- `/quit` - Exit the application\n- `/clear` - Clear chat history\n- `/projects` - List all projects\n- `/project <name>` - Switch to a project\n- `/status` - Show application status\n\n**Navigation:**\n- Type `/` to open command popup\n- `↑↓` - Navigate commands\n- `Tab/Enter` - Complete/execute command\n- `Esc` - Cancel command mode\n\n**Text Input:**\n- `Enter` - Submit message\n- `↑↓` - Navigate input history (when input empty)\n- `Esc` - Clear input or cancel\n\n**Other:**\n- `Tab` - Switch screens (when input empty)\n- `q` - Quick quit (when input empty)".to_string();
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
                        // Show projects as before, but refreshed
                        if let Err(e) = self.load_projects().await {
                            self.state.add_system_message(format!("⚠️ Failed to load projects: {}", e));
                        } else if self.state.projects.is_empty() {
                            self.state.add_system_message("📁 **Projects**\n\nNo projects found.\n\n💡 *Tip: Use the CLI to add projects: `orkee projects add`*".to_string());
                        } else {
                            let mut content = String::from("📁 **Projects** (Refreshed)\n\n");
                            for (i, project) in self.state.projects.iter().enumerate() {
                                let status = format!("{:?}", project.status).to_lowercase();
                                content.push_str(&format!("{}. **{}** ({})\n", i + 1, project.name, status));
                                if let Some(description) = &project.description {
                                    if !description.is_empty() {
                                        content.push_str(&format!("   └─ {}\n", description));
                                    }
                                }
                                content.push_str(&format!("   📂 {}\n\n", project.project_root));
                            }
                            self.state.add_system_message(content);
                        }
                    }
                    SlashCommand::Project => {
                        if let Some(project_name) = args.first() {
                            // Find project by name
                            if let Some(project) = self.state.projects.iter().find(|p| p.name == *project_name) {
                                self.state.add_system_message(format!("📂 **Switched to project: {}**\n\n**Path:** {}\n**Status:** {:?}\n\n💡 *Project switching functionality coming soon!*", project.name, project.project_root, project.status));
                            } else {
                                self.state.add_system_message(format!("❌ **Project not found:** {}\n\n💡 *Use `/projects` to see available projects*", project_name));
                            }
                        } else {
                            self.state.add_system_message("❌ **Missing project name**\n\nUsage: `/project <name>`\n\n💡 *Use `/projects` to see available projects*".to_string());
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
    }
}