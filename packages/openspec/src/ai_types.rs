// ABOUTME: AI analysis types for PRD analysis and spec generation
// ABOUTME: Types used for structured AI output when analyzing PRDs and generating specs

use serde::{Deserialize, Serialize};

/// Scenario from AI analysis (simplified version for AI input/output)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecScenario {
    pub name: String,
    pub when: String,
    pub then: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub and: Option<Vec<String>>,
}

/// Requirement from AI analysis (simplified version for AI input/output)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecRequirement {
    pub name: String,
    pub content: String,
    pub scenarios: Vec<SpecScenario>,
}

/// Capability from AI analysis (simplified version for AI input/output)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecCapability {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub requirements: Vec<SpecRequirement>,
}

/// Task suggestion from AI analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskSuggestion {
    pub title: String,
    pub description: String,
    #[serde(rename = "capabilityId")]
    pub capability_id: String,
    #[serde(rename = "requirementName")]
    pub requirement_name: String,
    pub complexity: u8,
    #[serde(rename = "estimatedHours", skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f32>,
    pub priority: String,
}

/// Complete PRD analysis output from AI
#[derive(Serialize, Deserialize)]
pub struct PRDAnalysisData {
    pub summary: String,
    pub capabilities: Vec<SpecCapability>,
    #[serde(rename = "suggestedTasks")]
    pub suggested_tasks: Vec<TaskSuggestion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
    #[serde(
        rename = "technicalConsiderations",
        skip_serializing_if = "Option::is_none"
    )]
    pub technical_considerations: Option<Vec<String>>,
}
