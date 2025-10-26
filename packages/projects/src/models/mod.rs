// ABOUTME: Models and agents registry module
// ABOUTME: Provides JSON-backed configuration for AI models and coding agents

pub mod registry;
pub mod types;

pub use registry::{ModelRegistry, REGISTRY};
pub use types::{Agent, AgentConfig, AgentModelRef, Model, ModelCapabilities, ModelPricing};
