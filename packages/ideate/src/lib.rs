// ABOUTME: Orkee ideate library - brainstorming and PRD ideation functionality
// ABOUTME: Provides session management, section handling, and PRD generation support

pub mod build_optimizer;
pub mod conversational;
pub mod conversational_manager;
pub mod dependency_analyzer;
pub mod epic;
pub mod epic_manager;
pub mod error;
pub mod expert_moderator;
pub mod export_service;
pub mod github_sync;
pub mod manager;
pub mod prd_aggregator;
pub mod prd_generator;
pub mod prompts;
pub mod research_analyzer;
pub mod research_prompts;
pub mod roundtable;
pub mod roundtable_manager;
pub mod task_decomposer;
pub mod templates;
pub mod types;

pub use build_optimizer::{
    BuildOptimizer, BuildOrderResult, CircularDependency, CircularDependencySeverity,
    OptimizationStrategy,
};
pub use conversational::{
    ConversationInsight, ConversationMessage, CreateInsightInput, DiscoveryQuestion,
    DiscoveryStatus, GeneratePRDFromConversationInput, GeneratePRDFromConversationResult,
    InsightType, MessageRole, MessageType, QualityMetrics, QuestionCategory, SendMessageInput,
    TopicCoverage, ValidationResult,
};
pub use conversational_manager::ConversationalManager;
pub use dependency_analyzer::{
    CreateDependencyInput, DependencyAnalysis, DependencyAnalyzer, DependencyStrength,
    DependencyType, FeatureDependency,
};
pub use epic::{
    ArchitectureDecision, ConflictAnalysis, CreateEpicInput, DependencyGraph, Epic,
    EpicComplexity, EpicStatus, EstimatedEffort, ExternalDependency, GraphEdge, GraphNode,
    SuccessCriterion, TaskConflict, UpdateEpicInput, WorkAnalysis, WorkStream,
};
pub use epic_manager::EpicManager;
pub use github_sync::{
    EntityType, GitHubConfig, GitHubSync, GitHubSyncError, GitHubSyncService, SyncDirection,
    SyncResult, SyncStatus,
};
pub use error::{IdeateError, Result};
pub use expert_moderator::ExpertModerator;
pub use export_service::{ExportFormat, ExportOptions, ExportResult, ExportService};
pub use manager::IdeateManager;
pub use prd_aggregator::{AggregatedPRDData, CompletenessMetrics, PRDAggregator};
pub use prd_generator::PRDGenerator;
pub use research_analyzer::{
    GapAnalysis, Lesson, Opportunity, ResearchAnalyzer, ResearchSynthesis, UIPattern,
};
pub use roundtable::{
    CreateExpertPersonaInput, ExpertPersona, ExpertSuggestion, ExtractInsightsRequest,
    ExtractInsightsResponse, InsightPriority, InsightsByCategory, MessageMetadata,
    RoundtableEvent, RoundtableInsight, RoundtableMessage, RoundtableParticipant,
    RoundtableSession, RoundtableStatistics, RoundtableStatus, RoundtableWithParticipants,
    StartRoundtableRequest, SuggestExpertsRequest, SuggestExpertsResponse, UserInterjectionInput,
    UserInterjectionResponse,
};
pub use roundtable_manager::RoundtableManager;
pub use task_decomposer::{
    DecomposeEpicInput, DecompositionResult, ParallelGroup, TaskCategory, TaskDecomposer,
    TaskTemplate,
};
pub use templates::TemplateManager;
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
