// ABOUTME: AI prompts for PRD section generation - now using centralized PromptManager
// ABOUTME: Wrapper functions that load prompts from JSON files via PromptManager

use orkee_prompts::PromptManager;
use std::sync::Mutex;

// Thread-safe singleton PromptManager
lazy_static::lazy_static! {
    static ref PROMPT_MANAGER: Mutex<PromptManager> = {
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

/// Get system prompt for PRD generation
pub fn get_system_prompt() -> String {
    PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_system_prompt("prd")
        .expect("Failed to load PRD system prompt")
}

/// Generate the overview section (Problem, Target, Value)
pub fn overview_prompt(description: &str) -> String {
    PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt("overview", &[("description", description)])
        .expect("Failed to load overview prompt")
}

/// Generate core features section
pub fn features_prompt(description: &str) -> String {
    PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt("features", &[("description", description)])
        .expect("Failed to load features prompt")
}

/// Generate UX section (Personas, Flows)
pub fn ux_prompt(description: &str) -> String {
    PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt("ux", &[("description", description)])
        .expect("Failed to load ux prompt")
}

/// Generate technical architecture section
pub fn technical_prompt(description: &str) -> String {
    PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt("technical", &[("description", description)])
        .expect("Failed to load technical prompt")
}

/// Generate development roadmap (scope only, NO timelines)
pub fn roadmap_prompt(description: &str, features: &str) -> String {
    PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt(
            "roadmap",
            &[("description", description), ("features", features)],
        )
        .expect("Failed to load roadmap prompt")
}

/// Generate dependency chain section
pub fn dependencies_prompt(description: &str, features: &str) -> String {
    PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt(
            "dependencies",
            &[("description", description), ("features", features)],
        )
        .expect("Failed to load dependencies prompt")
}

/// Generate risks and mitigations section
pub fn risks_prompt(description: &str) -> String {
    PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt("risks", &[("description", description)])
        .expect("Failed to load risks prompt")
}

/// Generate research/appendix section
pub fn research_prompt(description: &str) -> String {
    PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt("research", &[("description", description)])
        .expect("Failed to load research prompt")
}

/// Generate a complete PRD from a one-liner description
pub fn complete_prd_prompt(description: &str) -> String {
    PROMPT_MANAGER
        .lock()
        .unwrap()
        .get_prompt("complete", &[("description", description)])
        .expect("Failed to load complete PRD prompt")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overview_prompt_includes_description() {
        let desc = "A mobile app for tracking water intake";
        let prompt = overview_prompt(desc);
        assert!(prompt.contains(desc));
        assert!(prompt.contains("problemStatement"));
        assert!(prompt.contains("targetAudience"));
    }

    #[test]
    fn test_features_prompt_requests_json() {
        let prompt = features_prompt("Test project");
        assert!(prompt.contains("features"));
        assert!(prompt.contains("buildPhase"));
        assert!(prompt.contains("dependsOn"));
    }

    #[test]
    fn test_complete_prd_prompt_has_all_sections() {
        let prompt = complete_prd_prompt("Test");
        assert!(prompt.contains("overview"));
        assert!(prompt.contains("features"));
        assert!(prompt.contains("ux"));
        assert!(prompt.contains("technical"));
        assert!(prompt.contains("roadmap"));
        assert!(prompt.contains("dependencies"));
        assert!(prompt.contains("risks"));
        assert!(prompt.contains("research"));
    }
}
