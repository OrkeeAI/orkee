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
        match key {
            KeyCode::Char('q') => {
                self.quit();
            }
            KeyCode::Tab => {
                self.state.next_screen();
            }
            KeyCode::Up => {
                self.state.scroll_up();
            }
            KeyCode::Down => {
                self.state.scroll_down();
            }
            KeyCode::Home => {
                self.state.scroll_to_bottom();
            }
            KeyCode::Char('d') => {
                // Show dashboard info as chat message
                let content = format!(
                    "📊 **Dashboard Status**\n\nProjects: {}\nCurrent Screen: {:?}\nRefresh Interval: {}s\n\n💡 *Tip: Press 'p' for projects, 's' for settings, 'q' to quit*",
                    self.state.projects.len(),
                    self.state.current_screen,
                    self.state.refresh_interval
                );
                self.state.add_system_message(content);
            }
            KeyCode::Char('p') => {
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
            KeyCode::Char('s') => {
                // Show settings info as chat message
                let content = format!("⚙️ **Settings**\n\nRefresh Interval: {}s\nCurrent Theme: Dark\n\n💡 *Settings management coming in future phases*", self.state.refresh_interval);
                self.state.add_system_message(content);
            }
            KeyCode::Char('h') => {
                // Show help message
                let content = "📚 **Help - Orkee TUI**\n\n**Navigation:**\n- `Tab` - Switch between screens\n- `↑/↓` - Scroll messages\n- `Home` - Jump to bottom\n\n**Commands:**\n- `d` - Show dashboard info\n- `p` - List projects\n- `s` - Show settings\n- `h` - Show this help\n- `q` - Quit application\n\n💡 *Phase 2 will add proper input handling and slash commands*".to_string();
                self.state.add_system_message(content);
            }
            KeyCode::Char('c') if crossterm::event::KeyModifiers::CONTROL.intersects(crossterm::event::KeyModifiers::CONTROL) => {
                self.quit();
            }
            _ => {
                // For Phase 1, we'll just ignore other keys
                // Input handling will come in Phase 2
            }
        }
        Ok(())
    }
    
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}