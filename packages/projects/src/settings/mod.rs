// ABOUTME: System settings module
// ABOUTME: Runtime configuration management with database persistence

pub mod storage;
pub mod types;
pub mod validation;

#[cfg(test)]
mod storage_tests;

pub use storage::SettingsStorage;
pub use types::*;
pub use validation::{validate_setting_value, ValidationError};
