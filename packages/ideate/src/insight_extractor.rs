// ABOUTME: AI-powered insight extraction from chat messages
// ABOUTME: Uses Claude to intelligently extract requirements, risks, constraints, assumptions, and decisions

use orkee_ai::AIService;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{ChatInsight, CreateInsightInput, InsightType, Result};

/// AI-extracted insight with metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtractedInsight {
    pub insight_type: String, // "requirement", "risk", "constraint", "assumption", "decision"
    pub insight_text: String,
    pub confidence: f64, // 0.0 to 1.0
    pub reasoning: Option<String>, // Why this was identified as an insight
}

/// Response from AI insight extraction
#[derive(Debug, Deserialize, Serialize)]
pub struct InsightExtractionResponse {
    pub insights: Vec<ExtractedInsight>,
}

/// Extract insights from a chat message using AI with deduplication
pub async fn extract_insights_with_ai(
    message_content: &str,
    conversation_context: &[String], // Recent messages for context
    existing_insights: &[ChatInsight], // Existing insights for deduplication
) -> Result<Vec<CreateInsightInput>> {
    info!(
        "Extracting insights with AI from message ({} existing insights for deduplication)",
        existing_insights.len()
    );

    let ai_service = AIService::new(); // Uses default model and env API key

    // Build context from recent messages
    let context = if conversation_context.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nRecent conversation context:\n{}",
            conversation_context.join("\n")
        )
    };

    // Format existing insights for AI to avoid duplicates
    let existing_insights_text = if existing_insights.is_empty() {
        String::new()
    } else {
        let insights_list: Vec<String> = existing_insights
            .iter()
            .map(|i| format!("- [{:?}] {}", i.insight_type, i.insight_text))
            .collect();
        format!(
            "\n\nEXISTING INSIGHTS (do NOT duplicate these):\n{}",
            insights_list.join("\n")
        )
    };

    let prompt = format!(
        r#"Analyze the following chat message and extract any PRD-relevant insights.

MESSAGE:
{}{}{}

Identify and extract:
1. **Requirements** - Things that must be implemented or features needed
2. **Risks** - Potential problems, challenges, or concerns
3. **Constraints** - Limitations, restrictions, or things that can't be done
4. **Assumptions** - Implicit beliefs or dependencies being assumed
5. **Decisions** - Technical or product decisions being made

For each insight:
- Extract the specific sentence or phrase
- Assign confidence (0.0-1.0) based on how explicitly stated it is
- Provide brief reasoning

Respond with JSON:
{{
  "insights": [
    {{
      "insight_type": "requirement|risk|constraint|assumption|decision",
      "insight_text": "The extracted insight text",
      "confidence": 0.9,
      "reasoning": "Why this is categorized this way"
    }}
  ]
}}

If no insights are found, return: {{"insights": []}}
"#,
        message_content, context, existing_insights_text
    );

    let system_prompt = Some(
        "You are an expert at analyzing product requirement discussions and extracting structured insights. \
         Be precise and only extract genuinely relevant insights. NEVER duplicate existing insights - carefully \
         check the EXISTING INSIGHTS section and only extract NEW insights not already captured. \
         Focus on actionable information that would be useful in a PRD.".to_string()
    );

    match ai_service.generate_structured::<InsightExtractionResponse>(prompt, system_prompt).await {
        Ok(response) => {
            info!("AI extracted {} insights", response.data.insights.len());

            let insights = response
                .data
                .insights
                .into_iter()
                .filter_map(|insight| {
                    // Map string type to InsightType enum
                    let insight_type = match insight.insight_type.to_lowercase().as_str() {
                        "requirement" => InsightType::Requirement,
                        "risk" => InsightType::Risk,
                        "constraint" => InsightType::Constraint,
                        "assumption" => InsightType::Assumption,
                        "decision" => InsightType::Decision,
                        _ => {
                            warn!("Unknown insight type: {}, skipping", insight.insight_type);
                            return None;
                        }
                    };

                    // Only include insights with reasonable confidence
                    if insight.confidence < 0.3 {
                        warn!("Skipping low-confidence insight: {}", insight.insight_text);
                        return None;
                    }

                    Some(CreateInsightInput {
                        insight_type,
                        insight_text: insight.insight_text,
                        confidence_score: Some(insight.confidence),
                        source_message_ids: None,
                    })
                })
                .collect();

            Ok(insights)
        }
        Err(e) => {
            warn!("AI insight extraction failed: {}, falling back to empty list", e);
            // Don't fail the whole operation if AI extraction fails
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insight_extraction_compiles() {
        // Just ensure the types compile correctly
        let _input = CreateInsightInput {
            insight_type: InsightType::Requirement,
            insight_text: "Test".to_string(),
            confidence_score: Some(0.9),
            source_message_ids: None,
        };
    }
}
