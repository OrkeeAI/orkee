// ABOUTME: Type definitions for API token authentication
// ABOUTME: Structures for token generation, storage, and validation

use serde::{Deserialize, Serialize};

/// API token stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    pub id: String,
    pub token_hash: String,
    pub name: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub is_active: bool,
}

/// Token generation result - includes plaintext token for display
/// This is the ONLY time the plaintext token is available
#[derive(Debug, Clone)]
pub struct TokenGeneration {
    pub token: String,      // Plaintext token - show once to user
    pub token_hash: String, // Hash to store in database
    pub id: String,         // Token ID
}

impl TokenGeneration {
    pub fn new(token: String, token_hash: String, id: String) -> Self {
        Self {
            token,
            token_hash,
            id,
        }
    }
}
