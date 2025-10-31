// ABOUTME: Error types for the ideate package
// ABOUTME: Defines all error variants for ideation operations

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

    #[error("Section not found: {0}")]
    SectionNotFound(String),

    #[error("Section data not found: {section} for session {session_id}")]
    SectionDataNotFound { section: String, session_id: String },

    #[error("Session not ready for PRD generation: {0}")]
    NotReadyForPRD(String),

    #[error("AI service error: {0}")]
    AIService(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Prompt loading error: {0}")]
    PromptError(String),
}

pub type Result<T> = std::result::Result<T, IdeateError>;
