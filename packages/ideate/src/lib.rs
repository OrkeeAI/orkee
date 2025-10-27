// ABOUTME: Orkee ideate library - brainstorming and PRD ideation functionality
// ABOUTME: Provides session management, section handling, and PRD generation support

pub mod build_optimizer;
pub mod dependency_analyzer;
pub mod error;
pub mod manager;
pub mod prd_generator;
pub mod prompts;
pub mod research_analyzer;
pub mod research_prompts;
pub mod types;

pub use build_optimizer::{
    BuildOptimizer, BuildOrderResult, CircularDependency, CircularDependencySeverity,
    OptimizationStrategy,
};
pub use dependency_analyzer::{
    CreateDependencyInput, DependencyAnalysis, DependencyAnalyzer, DependencyStrength,
    DependencyType, FeatureDependency,
};
pub use error::{IdeateError, Result};
pub use manager::IdeateManager;
pub use prd_generator::PRDGenerator;
pub use research_analyzer::{
    GapAnalysis, Lesson, Opportunity, ResearchAnalyzer, ResearchSynthesis, UIPattern,
};
pub use types::*;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::error::{IdeateError, Result};
    pub use crate::manager::IdeateManager;
    pub use crate::types::{
        CreateIdeateSessionInput, IdeateMode, IdeateSession, IdeateStatus, SessionCompletionStatus,
        SkipSectionRequest, UpdateIdeateSessionInput,
    };
}
