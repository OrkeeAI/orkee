// ABOUTME: Orkee ideate library - brainstorming and PRD ideation functionality
// ABOUTME: Provides session management, templates, and prompts for ideation

pub mod error;
pub mod manager;
pub mod prompts;
pub mod templates;
pub mod types;

pub use error::{IdeateError, Result};
pub use manager::IdeateManager;
pub use templates::TemplateManager;
pub use types::*;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::error::{IdeateError, Result};
    pub use crate::manager::IdeateManager;
    pub use crate::types::{
        CreateIdeateSessionInput, IdeateMode, IdeateSession, IdeateStatus, SessionCompletionStatus,
        SkipSectionRequest, UpdateIdeateSessionInput,
    };
}
