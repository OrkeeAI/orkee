// ABOUTME: Orkee ideate library - brainstorming and PRD ideation functionality
// ABOUTME: Provides session management, section handling, and PRD generation support

pub mod error;
pub mod manager;
pub mod types;

pub use error::{IdeateError, Result};
pub use manager::BrainstormManager;
pub use types::*;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::error::{IdeateError, Result};
    pub use crate::manager::BrainstormManager;
    pub use crate::types::{
        BrainstormMode, BrainstormSession, BrainstormStatus, CreateBrainstormSessionInput,
        SessionCompletionStatus, SkipSectionRequest, UpdateBrainstormSessionInput,
    };
}
