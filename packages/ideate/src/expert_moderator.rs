// ABOUTME: Expert discussion moderator utilities
// ABOUTME: Handles user interjections and helper functions (AI moved to frontend)

use crate::error::Result;
use crate::roundtable::*;
use crate::roundtable_manager::RoundtableManager;
use tracing::info;

/// Expert moderator for roundtable discussion utilities
pub struct ExpertModerator {
    manager: RoundtableManager,
}

impl ExpertModerator {
    pub fn new(manager: RoundtableManager) -> Self {
        Self { manager }
    }

    /// Handle user interjection mid-discussion
    pub async fn handle_interjection(
        &self,
        roundtable_id: &str,
        user_message: &str,
    ) -> Result<UserInterjectionResponse> {
        info!(
            "Handling user interjection in roundtable: {}",
            roundtable_id
        );

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

    // ========================================================================
    // PUBLIC HELPER METHODS (for frontend AI)
    // ========================================================================

    /// Get messages for a roundtable (helper for frontend AI)
    pub async fn get_messages_for_ai(&self, roundtable_id: &str) -> Result<Vec<RoundtableMessage>> {
        self.manager.get_messages(roundtable_id).await
    }

    /// Get participants for a roundtable (helper for frontend AI)
    pub async fn get_participants_for_ai(&self, roundtable_id: &str) -> Result<Vec<ExpertPersona>> {
        self.manager.get_participants(roundtable_id).await
    }
}

// ============================================================================
// AI HELPER FUNCTIONS (for frontend use)
// ============================================================================

/// Format conversation history for AI context
pub fn format_conversation_history(messages: &[RoundtableMessage]) -> String {
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

    formatted
}

/// Generate moderator opening statement
pub fn build_moderator_opening(topic: &str, participants: &[ExpertPersona]) -> String {
    let expert_list = participants
        .iter()
        .map(|e| format!("{} ({})", e.name, e.role))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "Welcome everyone! Today we're discussing: \"{}\". \
         Our panel includes: {}. \
         Let's begin by having each expert share their initial thoughts on this topic.",
        topic, expert_list
    )
}

/// Build expert suggestion prompt
pub fn build_expert_suggestion_prompt(request: &SuggestExpertsRequest) -> String {
    let num_experts = request.num_experts.unwrap_or(3);

    let mut prompt = format!("Project Description:\n{}\n\n", request.project_description);

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

    prompt
}

/// Build insight extraction prompt
pub fn build_insight_extraction_prompt(
    messages: &[RoundtableMessage],
    categories: Option<Vec<String>>,
) -> String {
    let conversation = format_conversation_history(messages);

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

    prompt
}

// ============================================================================
// AI SYSTEM PROMPTS (exported for frontend AI)
// ============================================================================

pub const EXPERT_SUGGESTION_SYSTEM_PROMPT: &str = "\
You are an expert in assembling high-quality advisory panels for software projects. \
Your goal is to suggest the most relevant experts who will provide diverse, valuable perspectives. \
Consider technical, business, design, and operational viewpoints. \
Ensure suggested experts complement each other and cover key project aspects. \
Assign relevance scores between 0.0 and 1.0 based on how critical each expert is for this specific project.";

pub const INSIGHT_EXTRACTION_SYSTEM_PROMPT: &str = "\
You are an expert discussion analyst specializing in extracting actionable insights from expert conversations. \
Your goal is to identify key themes, consensus points, disagreements, and actionable recommendations. \
Prioritize insights that are: (1) actionable, (2) backed by expert reasoning, (3) address critical project concerns. \
Assign priority levels based on impact: critical for must-have items, high for important, medium for nice-to-have, low for optional. \
Attribute insights to the experts who mentioned them.";

pub const EXPERT_RESPONSE_SYSTEM_PROMPT_PREFIX: &str = "\
You are participating in an expert roundtable discussion. Stay true to your persona and provide insights \
based on your specific expertise. Build on what other experts have said while adding your unique perspective. \
Be concise and focused - aim for 150-250 words per response.";

#[cfg(test)]
mod tests {
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
