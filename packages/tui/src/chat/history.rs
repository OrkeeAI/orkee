use super::{ChatMessage, MessageAuthor};

/// Manages the chat message history
#[derive(Debug, Clone)]
pub struct MessageHistory {
    messages: Vec<ChatMessage>,
    max_messages: usize,
}

impl Default for MessageHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageHistory {
    /// Create a new message history with default capacity
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create a new message history with specified max capacity
    pub fn with_capacity(max_messages: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_messages,
        }
    }

    /// Add a message to the history
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);

        // Keep only the most recent messages within capacity
        if self.messages.len() > self.max_messages {
            let excess = self.messages.len() - self.max_messages;
            self.messages.drain(0..excess);
        }
    }

    /// Add a user message
    pub fn add_user_message(&mut self, content: impl Into<String>) -> &ChatMessage {
        let message = ChatMessage::user(content);
        self.add_message(message);
        // Safe: we just added a message, so last() will always be Some
        self.messages.last().expect("Message was just added")
    }

    /// Add a system message
    pub fn add_system_message(&mut self, content: impl Into<String>) -> &ChatMessage {
        let message = ChatMessage::system(content);
        self.add_message(message);
        // Safe: we just added a message, so last() will always be Some
        self.messages.last().expect("Message was just added")
    }

    /// Add an assistant message
    pub fn add_assistant_message(&mut self, content: impl Into<String>) -> &ChatMessage {
        let message = ChatMessage::assistant(content);
        self.add_message(message);
        // Safe: we just added a message, so last() will always be Some
        self.messages.last().expect("Message was just added")
    }

    /// Get all messages
    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    /// Get messages in reverse chronological order (newest first)
    pub fn messages_reverse(&self) -> impl Iterator<Item = &ChatMessage> {
        self.messages.iter().rev()
    }

    /// Get the last message
    pub fn last_message(&self) -> Option<&ChatMessage> {
        self.messages.last()
    }

    /// Get the last user message
    pub fn last_user_message(&self) -> Option<&ChatMessage> {
        self.messages
            .iter()
            .rev()
            .find(|msg| matches!(msg.author, MessageAuthor::User))
    }

    /// Get message by ID
    pub fn get_message(&self, id: &str) -> Option<&ChatMessage> {
        self.messages.iter().find(|msg| msg.id == id)
    }

    /// Get message by ID (mutable)
    pub fn get_message_mut(&mut self, id: &str) -> Option<&mut ChatMessage> {
        self.messages.iter_mut().find(|msg| msg.id == id)
    }

    /// Get user message history for navigation (only user messages)
    pub fn user_messages(&self) -> impl Iterator<Item = &ChatMessage> {
        self.messages
            .iter()
            .filter(|msg| matches!(msg.author, MessageAuthor::User))
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get the number of messages
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_messages() {
        let mut history = MessageHistory::new();

        history.add_user_message("Hello");
        history.add_system_message("Hi there");
        history.add_assistant_message("How can I help?");

        assert_eq!(history.len(), 3);
        assert_eq!(history.messages()[0].content, "Hello");
        assert_eq!(history.messages()[1].content, "Hi there");
        assert_eq!(history.messages()[2].content, "How can I help?");
    }

    #[test]
    fn test_last_user_message() {
        let mut history = MessageHistory::new();

        history.add_user_message("First user message");
        history.add_system_message("System response");
        history.add_user_message("Second user message");

        let last_user = history.last_user_message().unwrap();
        assert_eq!(last_user.content, "Second user message");
    }

    #[test]
    fn test_capacity_limit() {
        let mut history = MessageHistory::with_capacity(2);

        history.add_user_message("Message 1");
        history.add_user_message("Message 2");
        history.add_user_message("Message 3");

        assert_eq!(history.len(), 2);
        assert_eq!(history.messages()[0].content, "Message 2");
        assert_eq!(history.messages()[1].content, "Message 3");
    }

    #[test]
    fn test_user_messages_filter() {
        let mut history = MessageHistory::new();

        history.add_user_message("User 1");
        history.add_system_message("System");
        history.add_user_message("User 2");
        history.add_assistant_message("Assistant");
        history.add_user_message("User 3");

        let user_messages: Vec<_> = history.user_messages().collect();
        assert_eq!(user_messages.len(), 3);
        assert_eq!(user_messages[0].content, "User 1");
        assert_eq!(user_messages[1].content, "User 2");
        assert_eq!(user_messages[2].content, "User 3");
    }
}
