// ABOUTME: Tag management system for organizing tasks
// ABOUTME: Provides types and storage layer for task tags

pub mod storage;
pub mod types;

// Re-export main types
pub use storage::TagStorage;
pub use types::{Tag, TagCreateInput, TagUpdateInput};
