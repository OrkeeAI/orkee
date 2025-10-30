pub mod buffer;
pub mod history;

pub use buffer::InputBuffer;
pub use history::InputHistory;

/// Input modes for the TUI
#[derive(Debug, Clone, PartialEq, Default)]
pub enum InputMode {
    /// Normal typing mode
    #[default]
    Normal,
    /// After typing '/' - command mode (Phase 3)
    Command,
    /// After typing '@' - search mode (Phase 4)
    Search,
    /// When navigating input history
    History,
    /// When editing a previous message (Phase 5)
    Edit,
    /// When navigating form fields
    Form,
    /// When searching and filtering projects (Phase 6)
    ProjectSearch,
}
