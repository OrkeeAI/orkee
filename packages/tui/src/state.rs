use serde_json::Value;

/// Application state management
#[derive(Debug, Clone)]
pub struct AppState {
    pub projects: Vec<Value>,
    pub selected_project: Option<usize>,
    pub current_screen: Screen,
    pub connection_status: ConnectionStatus,
    pub refresh_interval: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    Projects,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
}

impl AppState {
    pub fn new(refresh_interval: u64) -> Self {
        Self {
            projects: Vec::new(),
            selected_project: None,
            current_screen: Screen::Dashboard,
            connection_status: ConnectionStatus::Disconnected,
            refresh_interval,
        }
    }
    
    pub fn set_projects(&mut self, projects: Vec<Value>) {
        self.projects = projects;
    }
    
    pub fn next_screen(&mut self) {
        self.current_screen = match self.current_screen {
            Screen::Dashboard => Screen::Projects,
            Screen::Projects => Screen::Settings,
            Screen::Settings => Screen::Dashboard,
        }
    }
    
    pub fn set_connection_status(&mut self, status: ConnectionStatus) {
        self.connection_status = status;
    }
}