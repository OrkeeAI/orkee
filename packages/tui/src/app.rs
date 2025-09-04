use crate::api::ApiClient;
use crate::state::AppState;
use anyhow::Result;

/// Main TUI application struct
pub struct App {
    pub state: AppState,
    pub api_client: ApiClient,
    pub should_quit: bool,
}

impl App {
    pub fn new(server_url: String, refresh_interval: u64) -> Self {
        Self {
            state: AppState::new(refresh_interval),
            api_client: ApiClient::new(server_url),
            should_quit: false,
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