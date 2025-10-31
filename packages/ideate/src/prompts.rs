// ABOUTME: AI prompts for PRD section generation - now using centralized PromptManager
// ABOUTME: Wrapper functions that load prompts from JSON files via PromptManager

use orkee_prompts::{PromptError, PromptManager};
use std::sync::{Mutex, PoisonError};

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

/// Helper function to safely access PROMPT_MANAGER and handle lock/prompt errors
fn with_prompt_manager<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce(&mut PromptManager) -> Result<T, PromptError>,
{
    let mut manager = PROMPT_MANAGER
        .lock()
        .map_err(|e: PoisonError<_>| format!("Prompt manager lock poisoned: {}", e))?;
    f(&mut manager).map_err(|e| format!("Prompt error: {}", e))
}

/// Unwrap helper that logs errors - used for backwards compatibility during migration
fn unwrap_or_log<T>(result: Result<T, String>, context: &str) -> T {
    result.unwrap_or_else(|e| {
        tracing::error!("{}: {}", context, e);
        panic!("{}: {}", context, e)
    })
}

/// Get system prompt for PRD generation
pub fn get_system_prompt() -> Result<String, String> {
    with_prompt_manager(|manager| manager.get_system_prompt("prd"))
}

/// Generate the overview section (Problem, Target, Value)
pub fn overview_prompt(description: &str) -> Result<String, String> {
    with_prompt_manager(|manager| manager.get_prompt("overview", &[("description", description)]))
}

/// Generate core features section
pub fn features_prompt(description: &str) -> Result<String, String> {
    with_prompt_manager(|manager| manager.get_prompt("features", &[("description", description)]))
}

/// Generate UX section (Personas, Flows)
pub fn ux_prompt(description: &str) -> Result<String, String> {
    with_prompt_manager(|manager| manager.get_prompt("ux", &[("description", description)]))
}

/// Generate technical architecture section
pub fn technical_prompt(description: &str) -> Result<String, String> {
    with_prompt_manager(|manager| manager.get_prompt("technical", &[("description", description)]))
}

/// Generate development roadmap (scope only, NO timelines)
pub fn roadmap_prompt(description: &str, features: &str) -> Result<String, String> {
    with_prompt_manager(|manager| {
        manager.get_prompt("roadmap", &[("description", description), ("features", features)])
    })
}

/// Generate dependency chain section
pub fn dependencies_prompt(description: &str, features: &str) -> Result<String, String> {
    with_prompt_manager(|manager| {
        manager.get_prompt("dependencies", &[("description", description), ("features", features)])
    })
}

/// Generate risks and mitigations section
pub fn risks_prompt(description: &str) -> Result<String, String> {
    with_prompt_manager(|manager| manager.get_prompt("risks", &[("description", description)]))
}

/// Generate research/appendix section
pub fn research_prompt(description: &str) -> Result<String, String> {
    with_prompt_manager(|manager| manager.get_prompt("research", &[("description", description)]))
}

/// Generate a complete PRD from a one-liner description
pub fn complete_prd_prompt(description: &str) -> Result<String, String> {
    with_prompt_manager(|manager| manager.get_prompt("complete", &[("description", description)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overview_prompt_includes_description() {
        let desc = "A mobile app for tracking water intake";
        let prompt = overview_prompt(desc).expect("Failed to load prompt");
        assert!(prompt.contains(desc));
        assert!(prompt.contains("problemStatement"));
        assert!(prompt.contains("targetAudience"));
    }

    #[test]
    fn test_features_prompt_requests_json() {
        let prompt = features_prompt("Test project").expect("Failed to load prompt");
        assert!(prompt.contains("features"));
        assert!(prompt.contains("buildPhase"));
        assert!(prompt.contains("dependsOn"));
    }

    #[test]
    fn test_complete_prd_prompt_has_all_sections() {
        let prompt = complete_prd_prompt("Test").expect("Failed to load prompt");
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
