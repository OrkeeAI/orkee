use orkee_projects::Project;

/// Application state management
#[derive(Debug, Clone)]
pub struct AppState {
    pub projects: Vec<Project>,
    pub selected_project: Option<usize>,
    pub current_screen: Screen,
    pub refresh_interval: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    Projects,
    Settings,
}

impl AppState {
    pub fn new(refresh_interval: u64) -> Self {
        Self {
            projects: Vec::new(),
            selected_project: None,
            current_screen: Screen::Dashboard,
            refresh_interval,
        }
    }
    
    pub fn set_projects(&mut self, projects: Vec<Project>) {
        self.projects = projects;
    }
    
    pub fn next_screen(&mut self) {
        self.current_screen = match self.current_screen {
            Screen::Dashboard => Screen::Projects,
            Screen::Projects => Screen::Settings,
            Screen::Settings => Screen::Dashboard,
        }
    }
}