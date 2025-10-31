// ABOUTME: AI prompts for competitor analysis and research tools - now using centralized PromptManager
// ABOUTME: Wrapper functions that load prompts from JSON files via PromptManager

use orkee_prompts::PromptManager;
use std::sync::Mutex;

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

/// Get system prompt for research analysis
pub fn get_research_system_prompt() -> String {
    RESEARCH_PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_system_prompt("research")
        .expect("Failed to load research system prompt")
}

/// System prompt for all research analysis tasks
/// DEPRECATED: Use get_research_system_prompt() instead
pub const RESEARCH_SYSTEM_PROMPT: &str = "Use get_research_system_prompt() function instead";

/// Analyze competitor from scraped HTML content
pub fn competitor_analysis_prompt(
    project_description: &str,
    html_content: &str,
    url: &str,
) -> String {
    // Truncate HTML content to first 10000 chars to match original behavior
    let truncated_html = html_content.chars().take(10000).collect::<String>();

    RESEARCH_PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt(
            "competitor-analysis",
            &[
                ("projectDescription", project_description),
                ("htmlContent", &truncated_html),
                ("url", url),
            ],
        )
        .expect("Failed to load competitor analysis prompt")
}

/// Extract features from HTML content
pub fn feature_extraction_prompt(html_content: &str) -> String {
    // Truncate to first 8000 chars to match original behavior
    let truncated_html = html_content.chars().take(8000).collect::<String>();

    RESEARCH_PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt("feature-extraction", &[("htmlContent", &truncated_html)])
        .expect("Failed to load feature extraction prompt")
}

/// Extract UI/UX patterns from content and structure
pub fn ui_pattern_prompt(project_description: &str, html_structure: &str) -> String {
    // Truncate to first 8000 chars to match original behavior
    let truncated_structure = html_structure.chars().take(8000).collect::<String>();

    RESEARCH_PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt(
            "ui-pattern",
            &[
                ("projectDescription", project_description),
                ("htmlStructure", &truncated_structure),
            ],
        )
        .expect("Failed to load UI pattern prompt")
}

/// Compare features across competitors
pub fn gap_analysis_prompt(
    project_description: &str,
    your_features: &[String],
    competitor_features: &[&(String, Vec<String>)],
) -> String {
    let competitor_list = competitor_features
        .iter()
        .map(|(name, features)| format!("{}: {}", name, features.join(", ")))
        .collect::<Vec<_>>()
        .join("\n");

    let your_features_str = your_features.join(", ");

    RESEARCH_PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt(
            "gap-analysis",
            &[
                ("projectDescription", project_description),
                ("yourFeatures", &your_features_str),
                ("competitorFeatures", &competitor_list),
            ],
        )
        .expect("Failed to load gap analysis prompt")
}

/// Extract lessons from similar projects
pub fn lessons_learned_prompt(
    project_description: &str,
    similar_project_name: &str,
    positive_aspects: &[String],
    negative_aspects: &[String],
) -> String {
    let positive_str = positive_aspects.join("\n- ");
    let negative_str = negative_aspects.join("\n- ");

    RESEARCH_PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt(
            "lessons-learned",
            &[
                ("projectDescription", project_description),
                ("similarProjectName", similar_project_name),
                ("positiveAspects", &positive_str),
                ("negativeAspects", &negative_str),
            ],
        )
        .expect("Failed to load lessons learned prompt")
}

/// Synthesize research findings
pub fn research_synthesis_prompt(
    project_description: &str,
    competitors: &[(String, Vec<String>, Vec<String>)], // (name, strengths, gaps)
    similar_projects_count: usize,
) -> String {
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

    RESEARCH_PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt(
            "research-synthesis",
            &[
                ("projectDescription", project_description),
                ("competitorSummary", &competitor_summary),
                ("similarProjectsCount", &count_str),
            ],
        )
        .expect("Failed to load research synthesis prompt")
}
