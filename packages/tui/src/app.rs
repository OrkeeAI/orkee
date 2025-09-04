use crate::state::AppState;
use crate::events::{EventHandler, AppEvent};
use crate::ui;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEventKind};
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
                            self.handle_key_event(key_event.code).await?;
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
    async fn handle_key_event(&mut self, key: KeyCode) -> Result<()> {
        
        // Handle input-related keys when in input modes or with modifiers
        match key {
            // Text input keys
            KeyCode::Char(c) => {
                // Check for global shortcuts first
                if self.is_global_shortcut(c) && self.state.input_buffer().is_empty() {
                    return self.handle_global_shortcut(c).await;
                }
                
                // Otherwise, add to input buffer
                self.state.input_buffer_mut().insert_char(c);
            }
            
            // Input editing keys
            KeyCode::Backspace => {
                self.state.input_buffer_mut().backspace();
            }
            KeyCode::Delete => {
                self.state.input_buffer_mut().delete_char();
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
            
            // History navigation
            KeyCode::Up => {
                if !self.state.navigate_history_previous() {
                    // If not in history mode or empty history, scroll chat
                    self.state.scroll_up();
                }
            }
            KeyCode::Down => {
                if !self.state.navigate_history_next() {
                    // If not in history mode, scroll chat
                    self.state.scroll_down();
                }
            }
            
            // Submit message
            KeyCode::Enter => {
                self.handle_input_submission().await;
            }
            
            // Cancel/escape
            KeyCode::Esc => {
                if !self.state.cancel_history_navigation() {
                    // If not canceling history, clear input buffer
                    self.state.input_buffer_mut().clear();
                }
            }
            
            // Tab for screen switching (only if input is empty)
            KeyCode::Tab => {
                if self.state.input_buffer().is_empty() {
                    self.state.next_screen();
                } else {
                    // TODO: Could implement tab completion here in future
                }
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
        if self.state.submit_current_input() {
            // Message was submitted successfully
            // In future phases, this is where we'd process commands or send to server
        }
    }
    
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}