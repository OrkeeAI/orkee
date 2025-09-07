use rmcp::{
    model::{CallToolResult, Content, ServerInfo}, 
    tool_router, tool, prompt_router, tool_handler, prompt_handler, ServerHandler,
    handler::server::{
        tool::ToolRouter, 
        router::prompt::PromptRouter, 
        wrapper::Parameters
    },
    ServiceError
};
use schemars::JsonSchema;
use serde::Deserialize;
use anyhow::Result;

use crate::tools::{projects_tool_execute, project_manage_tool_execute};

/// Request parameters for the projects tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProjectsRequest {
    #[serde(default = "default_action")]
    pub action: String,
    pub id: Option<String>,
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(default = "default_status")]
    pub status: String,
    pub priority: Option<String>,
    pub has_git: Option<bool>,
}

fn default_action() -> String {
    "list".to_string()
}

fn default_status() -> String {
    "active".to_string()
}

/// Request parameters for the project management tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProjectManageRequest {
    pub action: String,
    pub id: Option<String>,
    // Project data fields
    pub name: Option<String>,
    #[serde(rename = "projectRoot")]
    pub project_root: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "setupScript")]
    pub setup_script: Option<String>,
    #[serde(rename = "devScript")]
    pub dev_script: Option<String>,
    #[serde(rename = "cleanupScript")]
    pub cleanup_script: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

/// The main Orkee MCP server
#[derive(Clone)]
pub struct OrkeeServer {
    tool_router: ToolRouter<OrkeeServer>,
    prompt_router: PromptRouter<OrkeeServer>,
}

#[tool_router]
impl OrkeeServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    #[tool(description = "Say hello from Orkee")]
    async fn say_hello(&self) -> Result<CallToolResult, ServiceError> {
        Ok(CallToolResult::success(vec![Content::text("Hello from Orkee MCP Server!".to_string())]))
    }
}

#[prompt_router]
impl OrkeeServer {
    // Empty for now - can add prompts later if needed
}

#[tool_handler]
impl ServerHandler for OrkeeServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::default()
    }
}