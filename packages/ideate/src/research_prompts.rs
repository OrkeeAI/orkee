// ABOUTME: AI prompts for competitor analysis and research tools - now using centralized PromptManager
// ABOUTME: Wrapper functions that load prompts from JSON files via PromptManager

use orkee_prompts::{PromptError, PromptManager};
use std::sync::{Mutex, PoisonError};

// Thread-safe singleton PromptManager
lazy_static::lazy_static! {
    static ref RESEARCH_PROMPT_MANAGER: Mutex<PromptManager> = {
        // For tests, try to find the prompts directory in the workspace
        let prompts_dir = if cfg!(test) {
            let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .and_then(|dir| std::path::PathBuf::from(dir).parent().map(|p| p.to_path_buf()));
            workspace_root.map(|root| root.join("prompts"))
        } else {
            None
        };

        Mutex::new(PromptManager::new(prompts_dir).expect("Failed to initialize PromptManager"))
    };
}

/// Helper function to safely access RESEARCH_PROMPT_MANAGER and handle lock/prompt errors
fn with_research_prompt_manager<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce(&mut PromptManager) -> Result<T, PromptError>,
{
    let mut manager = RESEARCH_PROMPT_MANAGER
        .lock()
        .map_err(|e: PoisonError<_>| format!("Research prompt manager lock poisoned: {}", e))?;
    f(&mut manager).map_err(|e| format!("Prompt error: {}", e))
}

/// Get system prompt for research analysis
pub fn get_research_system_prompt() -> Result<String, String> {
    with_research_prompt_manager(|manager| manager.get_system_prompt("research"))
}

/// Analyze competitor from scraped HTML content
pub fn competitor_analysis_prompt(
    project_description: &str,
    html_content: &str,
    url: &str,
) -> Result<String, String> {
    // Truncate HTML content to first 10000 chars to match original behavior
    let truncated_html = html_content.chars().take(10000).collect::<String>();

    with_research_prompt_manager(|manager| {
        manager.get_prompt(
            "competitor-analysis",
            &[
                ("projectDescription", project_description),
                ("htmlContent", &truncated_html),
                ("url", url),
            ],
        )
    })
}

/// Extract features from HTML content
pub fn feature_extraction_prompt(html_content: &str) -> Result<String, String> {
    // Truncate to first 8000 chars to match original behavior
    let truncated_html = html_content.chars().take(8000).collect::<String>();

    with_research_prompt_manager(|manager| {
        manager.get_prompt("feature-extraction", &[("htmlContent", &truncated_html)])
    })
}

/// Extract UI/UX patterns from content and structure
pub fn ui_pattern_prompt(
    project_description: &str,
    html_structure: &str,
) -> Result<String, String> {
    // Truncate to first 8000 chars to match original behavior
    let truncated_structure = html_structure.chars().take(8000).collect::<String>();

    with_research_prompt_manager(|manager| {
        manager.get_prompt(
            "ui-pattern",
            &[
                ("projectDescription", project_description),
                ("htmlStructure", &truncated_structure),
            ],
        )
    })
}

/// Compare features across competitors
pub fn gap_analysis_prompt(
    project_description: &str,
    your_features: &[String],
    competitor_features: &[&(String, Vec<String>)],
) -> Result<String, String> {
    let competitor_list = competitor_features
        .iter()
        .map(|(name, features)| format!("{}: {}", name, features.join(", ")))
        .collect::<Vec<_>>()
        .join("\n");

    let your_features_str = your_features.join(", ");

    with_research_prompt_manager(|manager| {
        manager.get_prompt(
            "gap-analysis",
            &[
                ("projectDescription", project_description),
                ("yourFeatures", &your_features_str),
                ("competitorFeatures", &competitor_list),
            ],
        )
    })
}

/// Extract lessons from similar projects
pub fn lessons_learned_prompt(
    project_description: &str,
    similar_project_name: &str,
    positive_aspects: &[String],
    negative_aspects: &[String],
) -> Result<String, String> {
    let positive_str = positive_aspects.join("\n- ");
    let negative_str = negative_aspects.join("\n- ");

    with_research_prompt_manager(|manager| {
        manager.get_prompt(
            "lessons-learned",
            &[
                ("projectDescription", project_description),
                ("similarProjectName", similar_project_name),
                ("positiveAspects", &positive_str),
                ("negativeAspects", &negative_str),
            ],
        )
    })
}

/// Synthesize research findings
pub fn research_synthesis_prompt(
    project_description: &str,
    competitors: &[(String, Vec<String>, Vec<String>)], // (name, strengths, gaps)
    similar_projects_count: usize,
) -> Result<String, String> {
    let competitor_summary = competitors
        .iter()
        .map(|(name, strengths, gaps)| {
            format!(
                "{}: Strengths: {}. Gaps: {}",
                name,
                strengths.join(", "),
                gaps.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let count_str = similar_projects_count.to_string();

    with_research_prompt_manager(|manager| {
        manager.get_prompt(
            "research-synthesis",
            &[
                ("projectDescription", project_description),
                ("competitorSummary", &competitor_summary),
                ("similarProjectsCount", &count_str),
            ],
        )
    })
}
