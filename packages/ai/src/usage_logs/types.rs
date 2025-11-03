// ABOUTME: AI usage log type definitions
// ABOUTME: Structures for tracking AI API usage, costs, and token consumption

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageLog {
    pub id: String,
    pub project_id: String,
    pub request_id: Option<String>,
    pub operation: String,
    pub model: String,
    pub provider: String,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub estimated_cost: Option<f64>,
    pub duration_ms: Option<i64>,
    pub error: Option<String>,
    pub tool_calls_count: Option<i64>,
    pub tool_calls_json: Option<String>,
    pub response_metadata: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageStats {
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
    pub average_duration_ms: f64,
    #[serde(rename = "byOperation")]
    pub by_operation: Vec<OperationStats>,
    #[serde(rename = "byModel")]
    pub by_model: Vec<ModelStats>,
    #[serde(rename = "byProvider")]
    pub by_provider: Vec<ProviderStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStats {
    pub operation: String,
    pub count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub model: String,
    pub count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    pub provider: String,
    pub count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiUsageQuery {
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(rename = "endDate")]
    pub end_date: Option<DateTime<Utc>>,
    pub operation: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
