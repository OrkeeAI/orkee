// ABOUTME: Chunk manager for conversational chunking in Chat Mode
// ABOUTME: Handles 200-300 word chunks with validation, editing, and approval tracking

use crate::error::{IdeateError, Result};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tracing::{error, info};

/// Target minimum word count for content chunks
const TARGET_MIN_WORDS: usize = 200;

/// Target maximum word count for content chunks
const TARGET_MAX_WORDS: usize = 300;

/// Status of a chunk validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ChunkStatus {
    Pending,
    Approved,
    Rejected,
    Edited,
}

/// A chunk of PRD content for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrdChunk {
    pub id: String,
    pub session_id: String,
    pub section_name: String,
    pub chunk_number: i32,
    pub content: String,
    pub word_count: usize,
    pub status: ChunkStatus,
    pub edited_content: Option<String>,
    pub user_feedback: Option<String>,
    pub validated_at: Option<String>,
    pub created_at: String,
}

/// Input for validating a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateChunkInput {
    pub status: ChunkStatus,
    pub edited_content: Option<String>,
    pub feedback: Option<String>,
}

/// Manager for chunk-based PRD generation
pub struct ChunkManager {
    pool: SqlitePool,
}

impl ChunkManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Split content into 200-300 word chunks at natural break points
    pub fn chunk_content(content: &str, _section_name: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_word_count = 0;

        // Split by paragraphs (double newline or single newline for lists)
        let paragraphs: Vec<&str> = content
            .split("\n\n")
            .flat_map(|para| {
                // Also split on single newlines if they look like list items
                if para.contains("\n- ") || para.contains("\n* ") || para.contains("\n1. ") {
                    para.split('\n').collect::<Vec<_>>()
                } else {
                    vec![para]
                }
            })
            .filter(|p| !p.trim().is_empty())
            .collect();

        for paragraph in paragraphs {
            let para_words = paragraph.split_whitespace().count();

            // If adding this paragraph exceeds max, save current chunk
            if current_word_count + para_words > TARGET_MAX_WORDS
                && current_word_count >= TARGET_MIN_WORDS
            {
                chunks.push(current_chunk.trim().to_string());
                current_chunk = String::new();
                current_word_count = 0;
            }

            // Add paragraph to current chunk
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(paragraph);
            current_word_count += para_words;
        }

        // Add final chunk if not empty
        if !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        // If no chunks were created (content too short), return the whole content
        if chunks.is_empty() && !content.is_empty() {
            chunks.push(content.trim().to_string());
        }

        chunks
    }

    /// Save a chunk to the database
    pub async fn save_chunk(
        &self,
        session_id: &str,
        section_name: &str,
        chunk_number: i32,
        content: &str,
    ) -> Result<PrdChunk> {
        info!(
            "Saving chunk #{} for session: {}, section: {}",
            chunk_number, session_id, section_name
        );

        let id = nanoid!(12);
        let now = chrono::Utc::now().to_rfc3339();
        let word_count = content.split_whitespace().count();

        sqlx::query(
            r#"
            INSERT INTO prd_validation_history (
                id,
                session_id,
                section_name,
                chunk_number,
                chunk_content,
                chunk_word_count,
                validation_status,
                validated_at
            ) VALUES (?, ?, ?, ?, ?, ?, 'pending', ?)
            "#,
        )
        .bind(&id)
        .bind(session_id)
        .bind(section_name)
        .bind(chunk_number)
        .bind(content)
        .bind(word_count as i32)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to save chunk: {}", e);
            IdeateError::Database(e)
        })?;

        Ok(PrdChunk {
            id,
            session_id: session_id.to_string(),
            section_name: section_name.to_string(),
            chunk_number,
            content: content.to_string(),
            word_count,
            status: ChunkStatus::Pending,
            edited_content: None,
            user_feedback: None,
            validated_at: Some(now.clone()),
            created_at: now,
        })
    }

    /// Validate a chunk (approve, reject, or edit)
    pub async fn validate_chunk(
        &self,
        session_id: &str,
        chunk_number: i32,
        input: ValidateChunkInput,
    ) -> Result<()> {
        info!(
            "Validating chunk #{} for session: {} with status: {:?}",
            chunk_number, session_id, input.status
        );

        let now = chrono::Utc::now().to_rfc3339();
        let status_str = match input.status {
            ChunkStatus::Pending => "pending",
            ChunkStatus::Approved => "approved",
            ChunkStatus::Rejected => "rejected",
            ChunkStatus::Edited => "approved", // Edited chunks are auto-approved
        };

        sqlx::query(
            r#"
            UPDATE prd_validation_history
            SET validation_status = ?,
                edited_content = ?,
                user_feedback = ?,
                validated_at = ?
            WHERE session_id = ? AND chunk_number = ?
            "#,
        )
        .bind(status_str)
        .bind(&input.edited_content)
        .bind(&input.feedback)
        .bind(&now)
        .bind(session_id)
        .bind(chunk_number)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to validate chunk: {}", e);
            IdeateError::Database(e)
        })?;

        Ok(())
    }

    /// Get all chunks for a session
    pub async fn get_chunks(&self, session_id: &str) -> Result<Vec<PrdChunk>> {
        info!("Getting chunks for session: {}", session_id);

        let rows = sqlx::query(
            r#"
            SELECT
                id,
                session_id,
                section_name,
                chunk_number,
                chunk_content,
                chunk_word_count,
                validation_status,
                edited_content,
                user_feedback,
                validated_at
            FROM prd_validation_history
            WHERE session_id = ? AND chunk_number IS NOT NULL
            ORDER BY chunk_number ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get chunks: {}", e);
            IdeateError::Database(e)
        })?;

        let chunks = rows
            .into_iter()
            .map(|row| {
                let status_str: String = row.get("validation_status");
                let status = match status_str.as_str() {
                    "approved" => ChunkStatus::Approved,
                    "rejected" => ChunkStatus::Rejected,
                    "pending" => ChunkStatus::Pending,
                    _ => ChunkStatus::Pending,
                };

                PrdChunk {
                    id: row.get("id"),
                    session_id: row.get("session_id"),
                    section_name: row.get("section_name"),
                    chunk_number: row.get("chunk_number"),
                    content: row.get("chunk_content"),
                    word_count: row.get::<i32, _>("chunk_word_count") as usize,
                    status,
                    edited_content: row.get("edited_content"),
                    user_feedback: row.get("user_feedback"),
                    validated_at: row.get("validated_at"),
                    created_at: row.get("validated_at"), // Use validated_at as created_at
                }
            })
            .collect();

        Ok(chunks)
    }

    /// Get chunks for a specific section
    pub async fn get_chunks_by_section(
        &self,
        session_id: &str,
        section_name: &str,
    ) -> Result<Vec<PrdChunk>> {
        info!(
            "Getting chunks for session: {}, section: {}",
            session_id, section_name
        );

        let rows = sqlx::query(
            r#"
            SELECT
                id,
                session_id,
                section_name,
                chunk_number,
                chunk_content,
                chunk_word_count,
                validation_status,
                edited_content,
                user_feedback,
                validated_at
            FROM prd_validation_history
            WHERE session_id = ? AND section_name = ? AND chunk_number IS NOT NULL
            ORDER BY chunk_number ASC
            "#,
        )
        .bind(session_id)
        .bind(section_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get chunks by section: {}", e);
            IdeateError::Database(e)
        })?;

        let chunks = rows
            .into_iter()
            .map(|row| {
                let status_str: String = row.get("validation_status");
                let status = match status_str.as_str() {
                    "approved" => ChunkStatus::Approved,
                    "rejected" => ChunkStatus::Rejected,
                    "pending" => ChunkStatus::Pending,
                    _ => ChunkStatus::Pending,
                };

                PrdChunk {
                    id: row.get("id"),
                    session_id: row.get("session_id"),
                    section_name: row.get("section_name"),
                    chunk_number: row.get("chunk_number"),
                    content: row.get("chunk_content"),
                    word_count: row.get::<i32, _>("chunk_word_count") as usize,
                    status,
                    edited_content: row.get("edited_content"),
                    user_feedback: row.get("user_feedback"),
                    validated_at: row.get("validated_at"),
                    created_at: row.get("validated_at"),
                }
            })
            .collect();

        Ok(chunks)
    }

    /// Regenerate a rejected chunk
    pub async fn regenerate_chunk(
        &self,
        session_id: &str,
        chunk_number: i32,
        new_content: &str,
    ) -> Result<()> {
        info!(
            "Regenerating chunk #{} for session: {}",
            chunk_number, session_id
        );

        let now = chrono::Utc::now().to_rfc3339();
        let word_count = new_content.split_whitespace().count();

        sqlx::query(
            r#"
            UPDATE prd_validation_history
            SET chunk_content = ?,
                chunk_word_count = ?,
                validation_status = 'pending',
                validated_at = ?
            WHERE session_id = ? AND chunk_number = ?
            "#,
        )
        .bind(new_content)
        .bind(word_count as i32)
        .bind(&now)
        .bind(session_id)
        .bind(chunk_number)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to regenerate chunk: {}", e);
            IdeateError::Database(e)
        })?;

        Ok(())
    }

    /// Check if all chunks for a session are approved
    pub async fn are_all_chunks_approved(&self, session_id: &str) -> Result<bool> {
        let rows = sqlx::query(
            r#"
            SELECT COUNT(*) as total,
                   SUM(CASE WHEN validation_status = 'approved' THEN 1 ELSE 0 END) as approved
            FROM prd_validation_history
            WHERE session_id = ? AND chunk_number IS NOT NULL
            "#,
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to check chunk approval status: {}", e);
            IdeateError::Database(e)
        })?;

        let total: i32 = rows.get("total");
        let approved: i32 = rows.get("approved");

        Ok(total > 0 && total == approved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_content_simple() {
        let content = "This is a test paragraph with about ten words in it.

This is another paragraph that also has some words.

And here is a third paragraph to test chunking.";

        let chunks = ChunkManager::chunk_content(content, "test");

        // With such short content, should be one chunk
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].contains("This is a test paragraph"));
    }

    #[test]
    fn test_chunk_content_long() {
        // Create content with ~500 words
        let paragraph = "This is a test sentence. ".repeat(20); // ~100 words
        let content = format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}",
            paragraph, paragraph, paragraph, paragraph, paragraph
        );

        let chunks = ChunkManager::chunk_content(&content, "test");

        // Should split into multiple chunks
        assert!(chunks.len() >= 2);

        // Each chunk should be roughly 200-300 words
        for chunk in &chunks {
            let word_count = chunk.split_whitespace().count();
            // Allow some flexibility due to natural break points
            assert!(word_count >= 100 && word_count <= 400);
        }
    }

    #[test]
    fn test_chunk_content_with_lists() {
        let content = "Here are some features:

- Feature one with description
- Feature two with description
- Feature three with description

And here is more content.";

        let chunks = ChunkManager::chunk_content(content, "test");

        assert!(!chunks.is_empty());
        assert!(chunks[0].contains("Feature one"));
    }
}
