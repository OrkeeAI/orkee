// ABOUTME: Error types for the ideate package
// ABOUTME: Defines all error variants for brainstorming operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdeateError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid session mode: {0}")]
    InvalidMode(String),

    #[error("Invalid session status: {0}")]
    InvalidStatus(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid section: {0}")]
    InvalidSection(String),

    #[error("Section data not found: {section} for session {session_id}")]
    SectionNotFound { section: String, session_id: String },

    #[error("Session not ready for PRD generation: {0}")]
    NotReadyForPRD(String),

    #[error("AI service error: {0}")]
    AIService(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, IdeateError>;
