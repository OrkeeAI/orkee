// ABOUTME: Insight extraction CRUD operations
// ABOUTME: Manages storage and retrieval of chat insights (AI extraction moved to frontend)

use serde::{Deserialize, Serialize};

// TODO: AI functionality moved to frontend - see packages/dashboard/src/services/chat-ai.ts:extractInsights()

/// AI-extracted insight with metadata (used for type compatibility only)
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtractedInsight {
    pub insight_type: String, // "requirement", "risk", "constraint", "assumption", "decision"
    pub insight_text: String,
    pub confidence: f64,           // 0.0 to 1.0
    pub reasoning: Option<String>, // Why this was identified as an insight
}

/// Response from AI insight extraction (used for type compatibility only)
#[derive(Debug, Deserialize, Serialize)]
pub struct InsightExtractionResponse {
    pub insights: Vec<ExtractedInsight>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insight_extraction_types_compile() {
        // Verify ExtractedInsight serialization
        let insight = ExtractedInsight {
            insight_type: "requirement".to_string(),
            insight_text: "User needs authentication".to_string(),
            confidence: 0.9,
            reasoning: Some("Mentioned in conversation".to_string()),
        };

        let json = serde_json::to_string(&insight).unwrap();
        let deserialized: ExtractedInsight = serde_json::from_str(&json).unwrap();

        assert_eq!(insight.insight_type, deserialized.insight_type);
        assert_eq!(insight.insight_text, deserialized.insight_text);
    }
}
