// ABOUTME: Orkee ideate library - brainstorming and PRD ideation functionality
// ABOUTME: Provides session management, section handling, and PRD generation support

pub mod error;
pub mod manager;
pub mod prd_generator;
pub mod prompts;
pub mod types;

pub use error::{IdeateError, Result};
pub use manager::IdeateManager;
pub use prd_generator::PRDGenerator;
pub use types::*;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::error::{IdeateError, Result};
    pub use crate::manager::IdeateManager;
    pub use crate::types::{
        IdeateMode, IdeateSession, IdeateStatus, CreateIdeateSessionInput,
        SessionCompletionStatus, SkipSectionRequest, UpdateIdeateSessionInput,
    };
}
