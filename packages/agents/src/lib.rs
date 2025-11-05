// ABOUTME: Agent registry and user agent configuration
// ABOUTME: Loads agent definitions from JSON and manages user-specific agent settings

pub mod registry;
pub mod storage;
pub mod types;

pub use registry::{Agent, AgentRegistry};
pub use storage::UserAgentStorage;
pub use types::UserAgent;
