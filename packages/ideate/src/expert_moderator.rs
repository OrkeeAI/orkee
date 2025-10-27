// ABOUTME: AI-powered expert discussion moderator
// ABOUTME: Orchestrates roundtable discussions, manages expert turns, and extracts insights

use crate::error::{IdeateError, Result};
use crate::roundtable::*;
use crate::roundtable_manager::RoundtableManager;
use ai::AIService;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

const MAX_DISCUSSION_ROUNDS: usize = 15; // Maximum number of back-and-forth exchanges
const MAX_RESPONSE_TOKENS: usize = 500; // Max tokens per expert response

/// Expert moderator for orchestrating roundtable discussions
pub struct ExpertModerator {
    manager: RoundtableManager,
    ai_service: AIService,
}

impl ExpertModerator {
    pub fn new(manager: RoundtableManager, ai_service: AIService) -> Self {
        Self {
            manager,
            ai_service,
        }
    }

    /// Start a moderated roundtable discussion
    pub async fn run_discussion(
        &self,
        roundtable_id: &str,
        initial_topic: &str,
    ) -> Result<()> {
        info!("Starting moderated discussion for roundtable: {}", roundtable_id);

        // Mark roundtable as started
        self.manager.start_roundtable(roundtable_id).await?;

        // Get participants
        let participants = self.manager.get_participants(roundtable_id).await?;
        if participants.is_empty() {
            return Err(IdeateError::Validation(
                "Cannot start discussion without participants".to_string(),
            ));
        }

        // Add initial system message
        self.manager
            .add_message(
                roundtable_id,
                MessageRole::System,
                None,
                None,
                format!(
                    "Discussion started with {} experts on topic: {}",
                    participants.len(),
                    initial_topic
                ),
                None,
            )
            .await?;

        // Add moderator opening
        let opening = self.generate_moderator_opening(initial_topic, &participants)?;
        self.manager
            .add_message(
                roundtable_id,
                MessageRole::Moderator,
                None,
                Some("Moderator".to_string()),
                opening,
                None,
            )
            .await?;

        // Run discussion rounds
        for round in 0..MAX_DISCUSSION_ROUNDS {
            debug!("Discussion round {}/{}", round + 1, MAX_DISCUSSION_ROUNDS);

            // Get messages so far for context
            let messages = self.manager.get_messages(roundtable_id).await?;

            // Determine which expert should speak next
            let next_expert = self.select_next_expert(&participants, &messages)?;

            // Generate expert response
            let expert_response = self
                .generate_expert_response(
                    &next_expert,
                    initial_topic,
                    &messages,
                    &participants,
                )
                .await?;

            // Add expert message
            self.manager
                .add_message(
                    roundtable_id,
                    MessageRole::Expert,
                    Some(next_expert.id.clone()),
                    Some(next_expert.name.clone()),
                    expert_response,
                    None,
                )
                .await?;

            // Check if discussion should naturally end
            if self.should_end_discussion(&messages)? {
                info!("Discussion reached natural conclusion at round {}", round + 1);
                break;
            }
        }

        // Add closing moderator message
        let closing = "Thank you all for your valuable insights. This discussion is now concluded.";
        self.manager
            .add_message(
                roundtable_id,
                MessageRole::Moderator,
                None,
                Some("Moderator".to_string()),
                closing.to_string(),
                None,
            )
            .await?;

        // Mark roundtable as completed
        self.manager.complete_roundtable(roundtable_id).await?;

        info!("Discussion completed for roundtable: {}", roundtable_id);
        Ok(())
    }

    /// Handle user interjection mid-discussion
    pub async fn handle_interjection(
        &self,
        roundtable_id: &str,
        user_message: &str,
    ) -> Result<UserInterjectionResponse> {
        info!("Handling user interjection in roundtable: {}", roundtable_id);

        // Add user message
        let message = self
            .manager
            .add_message(
                roundtable_id,
                MessageRole::User,
                None,
                Some("You".to_string()),
                user_message.to_string(),
                None,
            )
            .await?;

        // Generate moderator acknowledgment
        let acknowledgment = format!(
            "Thank you for that input. Let's consider this question: {}",
            user_message
        );

        self.manager
            .add_message(
                roundtable_id,
                MessageRole::Moderator,
                None,
                Some("Moderator".to_string()),
                acknowledgment,
                Some(MessageMetadata {
                    response_time_ms: None,
                    token_count: None,
                    interjection_acknowledged: Some(true),
                }),
            )
            .await?;

        Ok(UserInterjectionResponse {
            message_id: message.id,
            acknowledged: true,
        })
    }

    /// Suggest experts based on project context
    pub async fn suggest_experts(
        &self,
        request: &SuggestExpertsRequest,
    ) -> Result<SuggestExpertsResponse> {
        info!("Generating expert suggestions");

        let prompt = self.build_expert_suggestion_prompt(request)?;

        #[derive(Deserialize)]
        struct RawSuggestionResponse {
            suggestions: Vec<ExpertSuggestionRaw>,
        }

        #[derive(Deserialize)]
        struct ExpertSuggestionRaw {
            name: String,
            role: String,
            expertise_area: String,
            reason: String,
            relevance_score: f32,
        }

        let ai_response = self
            .ai_service
            .generate_structured::<RawSuggestionResponse>(
                prompt,
                Some(EXPERT_SUGGESTION_SYSTEM_PROMPT.to_string()),
            )
            .await
            .map_err(|e| IdeateError::AIService(format!("Failed to suggest experts: {}", e)))?;

        let suggestions = ai_response
            .data
            .suggestions
            .into_iter()
            .map(|s| ExpertSuggestion {
                id: format!("suggestion_{}", uuid::Uuid::new_v4().to_string().replace("-", "")),
                session_id: "".to_string(), // Will be set by handler
                expert_name: s.name,
                role: s.role,
                expertise_area: s.expertise_area,
                reason: s.reason,
                relevance_score: Some(s.relevance_score),
                created_at: chrono::Utc::now(),
            })
            .collect();

        Ok(SuggestExpertsResponse { suggestions })
    }

    /// Extract insights from completed discussion
    pub async fn extract_insights(
        &self,
        roundtable_id: &str,
        categories: Option<Vec<String>>,
    ) -> Result<ExtractInsightsResponse> {
        info!("Extracting insights from roundtable: {}", roundtable_id);

        let messages = self.manager.get_messages(roundtable_id).await?;

        if messages.is_empty() {
            return Ok(ExtractInsightsResponse {
                insights: vec![],
                summary: None,
            });
        }

        let prompt = self.build_insight_extraction_prompt(&messages, categories)?;

        #[derive(Deserialize)]
        struct RawInsightResponse {
            insights: Vec<InsightRaw>,
            summary: String,
        }

        #[derive(Deserialize)]
        struct InsightRaw {
            insight_text: String,
            category: String,
            priority: String,
            source_experts: Vec<String>,
        }

        let ai_response = self
            .ai_service
            .generate_structured::<RawInsightResponse>(
                prompt,
                Some(INSIGHT_EXTRACTION_SYSTEM_PROMPT.to_string()),
            )
            .await
            .map_err(|e| {
                IdeateError::AIService(format!("Failed to extract insights: {}", e))
            })?;

        // Store insights in database
        let mut stored_insights = Vec::new();
        for raw_insight in ai_response.data.insights {
            let priority = match raw_insight.priority.to_lowercase().as_str() {
                "low" => InsightPriority::Low,
                "medium" => InsightPriority::Medium,
                "high" => InsightPriority::High,
                "critical" => InsightPriority::Critical,
                _ => InsightPriority::Medium,
            };

            let insight = self
                .manager
                .add_insight(
                    roundtable_id,
                    raw_insight.insight_text,
                    raw_insight.category,
                    priority,
                    raw_insight.source_experts,
                    None, // Could enhance to track source message IDs
                )
                .await?;

            stored_insights.push(insight);
        }

        info!("Extracted {} insights from discussion", stored_insights.len());

        Ok(ExtractInsightsResponse {
            insights: stored_insights,
            summary: Some(ai_response.data.summary),
        })
    }

    // ========================================================================
    // PRIVATE HELPER METHODS
    // ========================================================================

    /// Generate moderator opening statement
    fn generate_moderator_opening(
        &self,
        topic: &str,
        participants: &[ExpertPersona],
    ) -> Result<String> {
        let expert_list = participants
            .iter()
            .map(|e| format!("{} ({})", e.name, e.role))
            .collect::<Vec<_>>()
            .join(", ");

        Ok(format!(
            "Welcome everyone! Today we're discussing: \"{}\". \
             Our panel includes: {}. \
             Let's begin by having each expert share their initial thoughts on this topic.",
            topic, expert_list
        ))
    }

    /// Select which expert should speak next
    fn select_next_expert<'a>(
        &self,
        participants: &'a [ExpertPersona],
        messages: &[RoundtableMessage],
    ) -> Result<&'a ExpertPersona> {
        // Count how many times each expert has spoken
        let mut speak_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for message in messages {
            if message.role == MessageRole::Expert {
                if let Some(expert_id) = &message.expert_id {
                    *speak_counts.entry(expert_id.clone()).or_insert(0) += 1;
                }
            }
        }

        // Find expert who has spoken the least
        let next_expert = participants
            .iter()
            .min_by_key(|expert| speak_counts.get(&expert.id).unwrap_or(&0))
            .ok_or_else(|| IdeateError::Validation("No participants available".to_string()))?;

        Ok(next_expert)
    }

    /// Generate expert response using AI
    async fn generate_expert_response(
        &self,
        expert: &ExpertPersona,
        topic: &str,
        messages: &[RoundtableMessage],
        all_experts: &[ExpertPersona],
    ) -> Result<String> {
        let conversation_context = self.format_conversation_history(messages, all_experts)?;

        let prompt = format!(
            "Topic: {}\n\n\
             Previous discussion:\n{}\n\n\
             As {}, provide your perspective on this topic. \
             Consider what other experts have said and add your unique insights. \
             Keep your response focused and under {} words.",
            topic,
            conversation_context,
            expert.name,
            MAX_RESPONSE_TOKENS / 2 // Rough words estimate
        );

        #[derive(Deserialize)]
        struct ResponseWrapper {
            response: String,
        }

        let ai_response = self
            .ai_service
            .generate_structured::<ResponseWrapper>(prompt, Some(expert.system_prompt.clone()))
            .await
            .map_err(|e| {
                IdeateError::AIService(format!(
                    "Failed to generate expert response for {}: {}",
                    expert.name, e
                ))
            })?;

        Ok(ai_response.data.response)
    }

    /// Format conversation history for context
    fn format_conversation_history(
        &self,
        messages: &[RoundtableMessage],
        all_experts: &[ExpertPersona],
    ) -> Result<String> {
        let mut formatted = String::new();

        for message in messages {
            match message.role {
                MessageRole::Expert => {
                    let name = message.expert_name.as_deref().unwrap_or("Expert");
                    formatted.push_str(&format!("{}: {}\n\n", name, message.content));
                }
                MessageRole::User => {
                    formatted.push_str(&format!("User: {}\n\n", message.content));
                }
                MessageRole::Moderator => {
                    formatted.push_str(&format!("Moderator: {}\n\n", message.content));
                }
                MessageRole::System => {
                    // Skip system messages in context
                }
            }
        }

        Ok(formatted)
    }

    /// Check if discussion should naturally end
    fn should_end_discussion(&self, messages: &[RoundtableMessage]) -> Result<bool> {
        // Simple heuristic: end if we have enough expert messages
        let expert_message_count = messages
            .iter()
            .filter(|m| m.role == MessageRole::Expert)
            .count();

        // End after each expert has spoken at least twice
        Ok(expert_message_count >= 10)
    }

    /// Build expert suggestion prompt
    fn build_expert_suggestion_prompt(&self, request: &SuggestExpertsRequest) -> Result<String> {
        let num_experts = request.num_experts.unwrap_or(3);

        let mut prompt = format!(
            "Project Description:\n{}\n\n",
            request.project_description
        );

        if let Some(content) = &request.existing_content {
            prompt.push_str(&format!("Existing Content:\n{}\n\n", content));
        }

        prompt.push_str(&format!(
            "Suggest {} expert personas who would provide the most valuable insights \
             for this project. Consider what perspectives would be most helpful.\n\n\
             Respond with JSON in this format:\n\
             {{\n\
               \"suggestions\": [\n\
                 {{\n\
                   \"name\": \"Expert Name\",\n\
                   \"role\": \"Job Title\",\n\
                   \"expertise_area\": \"Primary expertise\",\n\
                   \"reason\": \"Why this expert is relevant\",\n\
                   \"relevance_score\": 0.95\n\
                 }}\n\
               ]\n\
             }}",
            num_experts
        ));

        Ok(prompt)
    }

    /// Build insight extraction prompt
    fn build_insight_extraction_prompt(
        &self,
        messages: &[RoundtableMessage],
        categories: Option<Vec<String>>,
    ) -> Result<String> {
        let conversation = self.format_conversation_history(messages, &[])?;

        let mut prompt = format!(
            "Discussion:\n{}\n\n\
             Extract key insights from this expert roundtable discussion. \
             Identify important points, consensus areas, disagreements, and actionable recommendations.\n\n",
            conversation
        );

        if let Some(cats) = categories {
            prompt.push_str(&format!(
                "Organize insights into these categories: {}\n\n",
                cats.join(", ")
            ));
        }

        prompt.push_str(
            "Respond with JSON in this format:\n\
             {\n\
               \"insights\": [\n\
                 {\n\
                   \"insight_text\": \"The key insight\",\n\
                   \"category\": \"Technical\" or \"UX\" or \"Business\",\n\
                   \"priority\": \"low\" or \"medium\" or \"high\" or \"critical\",\n\
                   \"source_experts\": [\"Expert Name 1\", \"Expert Name 2\"]\n\
                 }\n\
               ],\n\
               \"summary\": \"Overall discussion summary\"\n\
             }",
        );

        Ok(prompt)
    }
}

// ============================================================================
// AI SYSTEM PROMPTS
// ============================================================================

const EXPERT_SUGGESTION_SYSTEM_PROMPT: &str = "\
You are an expert in assembling high-quality advisory panels for software projects. \
Your goal is to suggest the most relevant experts who will provide diverse, valuable perspectives. \
Consider technical, business, design, and operational viewpoints. \
Ensure suggested experts complement each other and cover key project aspects. \
Assign relevance scores between 0.0 and 1.0 based on how critical each expert is for this specific project.";

const INSIGHT_EXTRACTION_SYSTEM_PROMPT: &str = "\
You are an expert discussion analyst specializing in extracting actionable insights from expert conversations. \
Your goal is to identify key themes, consensus points, disagreements, and actionable recommendations. \
Prioritize insights that are: (1) actionable, (2) backed by expert reasoning, (3) address critical project concerns. \
Assign priority levels based on impact: critical for must-have items, high for important, medium for nice-to-have, low for optional. \
Attribute insights to the experts who mentioned them.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expert_selection_round_robin() {
        // Test that experts are selected fairly in round-robin fashion
        // This would require more setup, left as placeholder
    }

    #[test]
    fn test_discussion_termination() {
        // Test that discussions end at appropriate times
        // This would require message fixtures, left as placeholder
    }
}
