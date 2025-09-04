use crate::state::AppState;
use anyhow::Result;
use orkee_projects::get_all_projects;

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
    
    pub async fn run(&mut self) -> Result<()> {
        // TODO: Implement main TUI loop
        println!("Running TUI application...");
        Ok(())
    }
    
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}