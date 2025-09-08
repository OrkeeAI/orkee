use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod history;

pub use history::MessageHistory;

/// Represents who sent a message in the chat
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageAuthor {
    User,
    System,
    Assistant,
}

/// A chat message with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub content: String,
    pub author: MessageAuthor,
    pub timestamp: DateTime<Utc>,
    pub edited: bool,
}

impl ChatMessage {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(content.into(), MessageAuthor::User)
    }

    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(content.into(), MessageAuthor::System)
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(content.into(), MessageAuthor::Assistant)
    }

    /// Create a new message with specified author
    fn new(content: String, author: MessageAuthor) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            author,
            timestamp: Utc::now(),
            edited: false,
        }
    }

    /// Mark this message as edited
    pub fn mark_edited(&mut self) {
        self.edited = true;
    }

    /// Get a display-friendly author label
    pub fn author_label(&self) -> &'static str {
        match self.author {
            MessageAuthor::User => "You",
            MessageAuthor::System => "System",
            MessageAuthor::Assistant => "Assistant",
        }
    }

    /// Check if this message can be edited
    pub fn can_edit(&self) -> bool {
        matches!(self.author, MessageAuthor::User)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_message() {
        let msg = ChatMessage::user("Hello world");
        assert_eq!(msg.content, "Hello world");
        assert_eq!(msg.author, MessageAuthor::User);
        assert!(!msg.edited);
        assert!(msg.can_edit());
    }

    #[test]
    fn test_create_system_message() {
        let msg = ChatMessage::system("System status");
        assert_eq!(msg.author, MessageAuthor::System);
        assert!(!msg.can_edit());
    }

    #[test]
    fn test_mark_edited() {
        let mut msg = ChatMessage::user("Original");
        assert!(!msg.edited);
        msg.mark_edited();
        assert!(msg.edited);
    }
}
