// ABOUTME: AI model and agent registry
// ABOUTME: JSON-backed configuration for models and agents with in-memory lookup

pub mod registry;
pub mod types;

pub use registry::{ModelRegistry, REGISTRY};
pub use types::{Agent, AgentConfig, AgentModelRef, Model, ModelCapabilities, ModelPricing};
