// ABOUTME: Discovery manager for one-question-at-a-time PRD discovery
// ABOUTME: Handles sequential question generation, answer tracking, and session context analysis

use crate::chat::QuestionCategory;
use crate::error::{IdeateError, Result};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tracing::{error, info};

/// Type of question being asked
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum QuestionType {
    Open,
    MultipleChoice,
    YesNo,
}

/// A discovery question with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub question_text: String,
    pub question_type: QuestionType,
    pub options: Option<Vec<String>>,
    pub category: QuestionCategory,
    pub can_skip: bool,
}

/// Context for generating the next question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub answers: Vec<DiscoveryAnswer>,
    pub categories_covered: Vec<QuestionCategory>,
    pub total_questions_asked: usize,
}

/// An answer from the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryAnswer {
    pub id: String,
    pub session_id: String,
    pub question_number: i32,
    pub question_text: String,
    pub question_type: QuestionType,
    pub options: Option<serde_json::Value>,
    pub user_answer: String,
    pub asked_at: String,
    pub answered_at: Option<String>,
}

impl Question {
    /// Create an open-ended question
    pub fn open(text: impl Into<String>, category: QuestionCategory) -> Self {
        Self {
            question_text: text.into(),
            question_type: QuestionType::Open,
            options: None,
            category,
            can_skip: false,
        }
    }

    /// Create a multiple choice question
    pub fn multiple_choice(
        text: impl Into<String>,
        options: Vec<String>,
        category: QuestionCategory,
    ) -> Self {
        Self {
            question_text: text.into(),
            question_type: QuestionType::MultipleChoice,
            options: Some(options),
            category,
            can_skip: false,
        }
    }

    /// Create a yes/no question
    pub fn yes_no(text: impl Into<String>, category: QuestionCategory) -> Self {
        Self {
            question_text: text.into(),
            question_type: QuestionType::YesNo,
            options: Some(vec!["Yes".to_string(), "No".to_string()]),
            category,
            can_skip: false,
        }
    }

    /// Make this question skippable
    pub fn skippable(mut self) -> Self {
        self.can_skip = true;
        self
    }
}

/// Discovery manager for one-question-at-a-time flow
pub struct DiscoveryManager {
    pool: SqlitePool,
}

impl DiscoveryManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get the next question for a session based on context
    pub async fn get_next_question(&self, session_id: &str) -> Result<Question> {
        info!("Getting next question for session: {}", session_id);

        // Get session context
        let context = self.get_session_context(session_id).await?;

        // Determine next question based on what we've already asked
        let question = match context.total_questions_asked {
            0 => Question::open(
                "What problem are you trying to solve?",
                QuestionCategory::Problem,
            ),
            1 => Question::multiple_choice(
                "Who is the primary user?",
                vec![
                    "Internal team".to_string(),
                    "External customers".to_string(),
                    "Both".to_string(),
                    "Other".to_string(),
                ],
                QuestionCategory::Users,
            ),
            2 => Question::open(
                "What would success look like for this project?",
                QuestionCategory::Success,
            ),
            3 => Question::multiple_choice(
                "What's the main technical approach?",
                vec![
                    "Web application".to_string(),
                    "Mobile app".to_string(),
                    "Desktop application".to_string(),
                    "API/Backend service".to_string(),
                    "CLI tool".to_string(),
                ],
                QuestionCategory::Technical,
            ),
            4 => Question::yes_no(
                "Are there specific technical constraints or requirements?",
                QuestionCategory::Constraints,
            ),
            5 => Question::open(
                "What are the key features or capabilities?",
                QuestionCategory::Features,
            )
            .skippable(),
            6 => Question::yes_no(
                "Are there any known risks or concerns?",
                QuestionCategory::Risks,
            )
            .skippable(),
            _ => self.generate_contextual_question(&context).await?,
        };

        // Store the question in discovery_sessions table
        self.save_question(session_id, &question, context.total_questions_asked as i32)
            .await?;

        Ok(question)
    }

    /// Generate a contextual follow-up question based on previous answers
    async fn generate_contextual_question(&self, context: &SessionContext) -> Result<Question> {
        // Check which categories haven't been covered yet
        let uncovered: Vec<QuestionCategory> = vec![
            QuestionCategory::Problem,
            QuestionCategory::Users,
            QuestionCategory::Features,
            QuestionCategory::Technical,
            QuestionCategory::Risks,
            QuestionCategory::Constraints,
            QuestionCategory::Success,
        ]
        .into_iter()
        .filter(|cat| !context.categories_covered.contains(cat))
        .collect();

        if let Some(category) = uncovered.first() {
            // Generate question for uncovered category
            Ok(match category {
                QuestionCategory::Problem => Question::open(
                    "Can you elaborate more on the problem?",
                    QuestionCategory::Problem,
                ),
                QuestionCategory::Users => Question::open(
                    "Tell me more about who will use this",
                    QuestionCategory::Users,
                ),
                QuestionCategory::Features => Question::open(
                    "What other features should be included?",
                    QuestionCategory::Features,
                ),
                QuestionCategory::Technical => Question::open(
                    "Are there any specific technical requirements?",
                    QuestionCategory::Technical,
                ),
                QuestionCategory::Risks => Question::open(
                    "What could go wrong with this project?",
                    QuestionCategory::Risks,
                ),
                QuestionCategory::Constraints => Question::open(
                    "Are there any limitations or constraints?",
                    QuestionCategory::Constraints,
                ),
                QuestionCategory::Success => {
                    Question::open("How will we measure success?", QuestionCategory::Success)
                }
            })
        } else {
            // All categories covered, ask refinement question
            Ok(Question::open(
                "Is there anything else you'd like to add?",
                QuestionCategory::Problem,
            )
            .skippable())
        }
    }

    /// Get the current session context
    async fn get_session_context(&self, session_id: &str) -> Result<SessionContext> {
        let answers = self.get_answers(session_id).await?;
        let total_questions_asked = answers.len();

        let categories_covered: Vec<QuestionCategory> = answers
            .iter()
            .filter_map(|a| {
                // Try to infer category from question text (simplified for now)
                // In a real implementation, this would be stored with each question
                if a.question_text.to_lowercase().contains("problem") {
                    Some(QuestionCategory::Problem)
                } else if a.question_text.to_lowercase().contains("user") {
                    Some(QuestionCategory::Users)
                } else if a.question_text.to_lowercase().contains("feature") {
                    Some(QuestionCategory::Features)
                } else if a.question_text.to_lowercase().contains("technical") {
                    Some(QuestionCategory::Technical)
                } else if a.question_text.to_lowercase().contains("risk") {
                    Some(QuestionCategory::Risks)
                } else if a.question_text.to_lowercase().contains("constraint") {
                    Some(QuestionCategory::Constraints)
                } else if a.question_text.to_lowercase().contains("success") {
                    Some(QuestionCategory::Success)
                } else {
                    None
                }
            })
            .collect();

        Ok(SessionContext {
            answers,
            categories_covered,
            total_questions_asked,
        })
    }

    /// Get all answers for a session
    pub async fn get_answers(&self, session_id: &str) -> Result<Vec<DiscoveryAnswer>> {
        info!("Getting discovery answers for session: {}", session_id);

        let rows = sqlx::query(
            r#"
            SELECT
                id,
                session_id,
                question_number,
                question_text,
                question_type,
                options,
                user_answer,
                asked_at,
                answered_at
            FROM discovery_sessions
            WHERE session_id = ?
            ORDER BY question_number ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get discovery answers: {}", e);
            IdeateError::Database(e)
        })?;

        let answers: Vec<DiscoveryAnswer> = rows
            .into_iter()
            .map(|row| {
                let question_type_str: String = row.get("question_type");
                let question_type = match question_type_str.as_str() {
                    "open" => QuestionType::Open,
                    "multiple_choice" => QuestionType::MultipleChoice,
                    "yes_no" => QuestionType::YesNo,
                    _ => QuestionType::Open,
                };

                DiscoveryAnswer {
                    id: row.get("id"),
                    session_id: row.get("session_id"),
                    question_number: row.get("question_number"),
                    question_text: row.get("question_text"),
                    question_type,
                    options: row.get("options"),
                    user_answer: row.get("user_answer"),
                    asked_at: row.get("asked_at"),
                    answered_at: row.get("answered_at"),
                }
            })
            .collect();

        Ok(answers)
    }

    /// Save a question to the database
    async fn save_question(
        &self,
        session_id: &str,
        question: &Question,
        question_number: i32,
    ) -> Result<String> {
        info!(
            "Saving question #{} for session: {}",
            question_number, session_id
        );

        let id = nanoid!(12);
        let now = chrono::Utc::now().to_rfc3339();

        let options_json = question
            .options
            .as_ref()
            .map(|opts| serde_json::to_value(opts).unwrap());

        sqlx::query(
            r#"
            INSERT INTO discovery_sessions (
                id,
                session_id,
                question_number,
                question_text,
                question_type,
                options,
                user_answer,
                asked_at
            ) VALUES (?, ?, ?, ?, ?, ?, '', ?)
            "#,
        )
        .bind(&id)
        .bind(session_id)
        .bind(question_number)
        .bind(&question.question_text)
        .bind(&question.question_type)
        .bind(&options_json)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to save question: {}", e);
            IdeateError::Database(e)
        })?;

        Ok(id)
    }

    /// Save a user's answer
    pub async fn save_answer(
        &self,
        session_id: &str,
        question_number: i32,
        answer: String,
    ) -> Result<()> {
        info!(
            "Saving answer for session: {}, question: {}",
            session_id, question_number
        );

        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE discovery_sessions
            SET user_answer = ?, answered_at = ?
            WHERE session_id = ? AND question_number = ?
            "#,
        )
        .bind(&answer)
        .bind(&now)
        .bind(session_id)
        .bind(question_number)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to save answer: {}", e);
            IdeateError::Database(e)
        })?;

        Ok(())
    }

    /// Check if discovery is complete (all required questions answered)
    pub async fn is_discovery_complete(&self, session_id: &str) -> Result<bool> {
        let context = self.get_session_context(session_id).await?;

        // Consider discovery complete if we have at least 5 questions answered
        // and have covered the core categories
        let min_questions = 5;
        let core_categories = vec![
            QuestionCategory::Problem,
            QuestionCategory::Users,
            QuestionCategory::Features,
        ];

        let has_min_questions = context.total_questions_asked >= min_questions;
        let has_core_categories = core_categories
            .iter()
            .all(|cat| context.categories_covered.contains(cat));

        Ok(has_min_questions && has_core_categories)
    }
}
