// ABOUTME: AI-powered insight extraction from chat messages
// ABOUTME: Uses Claude to intelligently extract requirements, risks, constraints, assumptions, and decisions

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{ChatInsight, CreateInsightInput, InsightType, Result};

#[derive(Debug, serde::Deserialize)]
struct AnthropicResponse {
    #[allow(dead_code)]
    id: String,
    content: Vec<ContentBlock>,
    usage: Usage,
    #[allow(dead_code)]
    stop_reason: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    content_type: String,
    text: String,
}

#[derive(Debug, serde::Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

impl Usage {
    fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

/// AI-extracted insight with metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtractedInsight {
    pub insight_type: String, // "requirement", "risk", "constraint", "assumption", "decision"
    pub insight_text: String,
    pub confidence: f64,           // 0.0 to 1.0
    pub reasoning: Option<String>, // Why this was identified as an insight
}

/// Response from AI insight extraction
#[derive(Debug, Deserialize, Serialize)]
pub struct InsightExtractionResponse {
    pub insights: Vec<ExtractedInsight>,
}

/// Extract insights from a chat message using AI with deduplication
pub async fn extract_insights_with_ai(
    user_id: &str,
    message_content: &str,
    conversation_context: &[String],   // Recent messages for context
    existing_insights: &[ChatInsight], // Existing insights for deduplication
) -> Result<Vec<CreateInsightInput>> {
    info!(
        "Extracting insights with AI from message ({} existing insights for deduplication)",
        existing_insights.len()
    );

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

    let system_prompt =
        "You are an expert at analyzing product requirement discussions and extracting structured insights. \
         Be precise and only extract genuinely relevant insights. NEVER duplicate existing insights - carefully \
         check the EXISTING INSIGHTS section and only extract NEW insights not already captured. \
         Focus on actionable information that would be useful in a PRD.".to_string();

    let client = Client::new();

    let request_body = serde_json::json!({
        "model": "claude-3-opus-20240229",
        "max_tokens": 64000,
        "temperature": 0.7,
        "messages": [{
            "role": "user",
            "content": prompt
        }],
        "system": system_prompt
    });

    let api_port = std::env::var("ORKEE_API_PORT").unwrap_or_else(|_| "4001".to_string());
    let proxy_url = format!("http://localhost:{}/api/ai/anthropic/v1/messages", api_port);

    let response = client
        .post(&proxy_url)
        .header("x-user-id", user_id)
        .json(&request_body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                warn!(
                    "AI proxy returned error {}: {}, falling back to empty list",
                    status, error_text
                );
                return Ok(Vec::new());
            }

            let anthropic_response: AnthropicResponse = match resp.json().await {
                Ok(r) => r,
                Err(e) => {
                    warn!(
                        "Failed to parse AI response: {}, falling back to empty list",
                        e
                    );
                    return Ok(Vec::new());
                }
            };

            let data_text = anthropic_response
                .content
                .first()
                .map(|c| c.text.as_str())
                .unwrap_or("{}");

            let extraction_response: InsightExtractionResponse = match serde_json::from_str(data_text) {
                Ok(r) => r,
                Err(e) => {
                    warn!(
                        "Failed to parse insight extraction response: {}, falling back to empty list",
                        e
                    );
                    return Ok(Vec::new());
                }
            };

            info!("AI extracted {} insights", extraction_response.insights.len());

            let insights = extraction_response
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

                    // Skip insights with empty or whitespace-only text
                    if insight.insight_text.trim().is_empty() {
                        warn!("Skipping insight with empty text");
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
            warn!(
                "AI insight extraction failed: {}, falling back to empty list",
                e
            );
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
