use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::context::ToolContext;
use orkee_projects::{Priority, ProjectCreateInput, ProjectStatus, ProjectUpdateInput};

// MCP Tool Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsRequest {
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: ToolInputSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub type_name: String,
    pub properties: HashMap<String, ToolInputSchemaProperty>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchemaProperty {
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolRequest {
    pub name: String,
    pub arguments: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<ToolContent>,
    #[serde(rename = "isError")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

// Request types for our tools
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

// Tool Registration is now handled by router_builder! macro in main.rs

// Tool handlers
pub async fn tools_list(
    _request: Option<ListToolsRequest>,
    _context: Option<ToolContext>,
) -> Result<ListToolsResult> {
    let mut properties = HashMap::new();

    // Projects tool schema
    properties.insert(
        "action".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            description: Some("Action to perform: list, get, or search".to_string()),
            enum_values: Some(vec![
                "list".to_string(),
                "get".to_string(),
                "search".to_string(),
            ]),
        },
    );

    properties.insert(
        "id".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            description: Some("Project ID for get action".to_string()),
            enum_values: None,
        },
    );

    properties.insert(
        "query".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            description: Some("Search query for search action".to_string()),
            enum_values: None,
        },
    );

    properties.insert(
        "status".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            description: Some("Filter by project status".to_string()),
            enum_values: Some(vec![
                "active".to_string(),
                "archived".to_string(),
                "all".to_string(),
            ]),
        },
    );

    properties.insert(
        "priority".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            description: Some("Filter by project priority".to_string()),
            enum_values: Some(vec![
                "high".to_string(),
                "medium".to_string(),
                "low".to_string(),
            ]),
        },
    );

    properties.insert(
        "has_git".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("boolean".to_string()),
            description: Some("Filter by git repository presence".to_string()),
            enum_values: None,
        },
    );

    let projects_tool = Tool {
        name: "projects".to_string(),
        description: Some("List, get, or search Orkee projects. Supports filtering by git repository presence and enhanced search across project and git information.".to_string()),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties,
            required: vec![],
        },
    };

    // Project manage tool schema
    let mut manage_properties = HashMap::new();
    manage_properties.insert(
        "action".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            description: Some("Action to perform: create, update, or delete".to_string()),
            enum_values: Some(vec![
                "create".to_string(),
                "update".to_string(),
                "delete".to_string(),
            ]),
        },
    );

    manage_properties.insert(
        "id".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            description: Some("Project ID for update/delete actions".to_string()),
            enum_values: None,
        },
    );

    manage_properties.insert(
        "name".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            description: Some("Project name".to_string()),
            enum_values: None,
        },
    );

    manage_properties.insert(
        "projectRoot".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            description: Some("Absolute path to project root directory".to_string()),
            enum_values: None,
        },
    );

    manage_properties.insert(
        "description".to_string(),
        ToolInputSchemaProperty {
            type_name: Some("string".to_string()),
            description: Some("Project description".to_string()),
            enum_values: None,
        },
    );

    let project_manage_tool = Tool {
        name: "project_manage".to_string(),
        description: Some("Create, update, or delete an Orkee project. Provides full project lifecycle management with validation and error handling.".to_string()),
        input_schema: ToolInputSchema {
            type_name: "object".to_string(),
            properties: manage_properties,
            required: vec!["action".to_string()],
        },
    };

    let response = ListToolsResult {
        tools: vec![projects_tool, project_manage_tool],
        next_cursor: None,
    };

    Ok(response)
}

pub async fn tools_call(
    request: Option<CallToolRequest>,
    context: Option<ToolContext>,
) -> Result<CallToolResult> {
    // If no context provided, create a default one with global storage manager
    let context = match context {
        Some(ctx) => ctx,
        None => ToolContext::new()
            .await
            .map_err(|e| anyhow!("Failed to create context: {}", e))?,
    };
    let call_request = request.ok_or_else(|| anyhow!("Missing tool call request"))?;

    match call_request.name.as_str() {
        "projects" => {
            let args: ProjectsRequest = if let Some(arguments) = call_request.arguments {
                serde_json::from_value(arguments)
                    .map_err(|e| anyhow!("Failed to parse arguments: {}", e))?
            } else {
                ProjectsRequest {
                    action: "list".to_string(),
                    id: None,
                    query: None,
                    tags: None,
                    status: "active".to_string(),
                    priority: None,
                    has_git: None,
                }
            };

            let result = execute_projects_tool(args, &context).await;
            Ok(CallToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: result,
                }],
                is_error: None,
            })
        }
        "project_manage" => {
            let args: ProjectManageRequest = if let Some(arguments) = call_request.arguments {
                serde_json::from_value(arguments)
                    .map_err(|e| anyhow!("Failed to parse arguments: {}", e))?
            } else {
                return Ok(CallToolResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: json!({"error": "Missing arguments for project_manage"}).to_string(),
                    }],
                    is_error: Some(true),
                });
            };

            let result = execute_project_manage_tool(args, &context).await;
            Ok(CallToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: result,
                }],
                is_error: None,
            })
        }
        _ => Ok(CallToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: format!("Unknown tool: {}", call_request.name),
            }],
            is_error: Some(true),
        }),
    }
}

// Tool implementations (using existing tool logic)
async fn execute_projects_tool(request: ProjectsRequest, context: &ToolContext) -> String {
    match request.action.as_str() {
        "get" => {
            if let Some(id) = request.id {
                match context.projects_manager().get_project(&id).await {
                    Ok(Some(project)) => {
                        serde_json::to_string_pretty(&project).unwrap_or_else(|e| {
                            json!({ "error": format!("Serialization error: {}", e) }).to_string()
                        })
                    }
                    Ok(None) => {
                        json!({ "error": format!("Project not found: {}", id) }).to_string()
                    }
                    Err(e) => {
                        json!({ "error": format!("Failed to get project: {}", e) }).to_string()
                    }
                }
            } else {
                json!({ "error": "ID is required for get action" }).to_string()
            }
        }
        "list" | "search" => {
            match context.projects_manager().list_projects().await {
                Ok(mut projects) => {
                    // Filter by status
                    if request.status != "all" {
                        let status_filter = match request.status.as_str() {
                            "active" => ProjectStatus::Active,
                            "archived" => ProjectStatus::Archived,
                            _ => ProjectStatus::Active,
                        };
                        projects.retain(|p| p.status == status_filter);
                    }

                    // Filter by git repository presence
                    if let Some(has_git) = request.has_git {
                        if has_git {
                            projects.retain(|p| p.git_repository.is_some());
                        } else {
                            projects.retain(|p| p.git_repository.is_none());
                        }
                    }

                    // Apply search filters for search action
                    if request.action == "search" {
                        if let Some(query) = &request.query {
                            let query_lower = query.to_lowercase();
                            projects.retain(|p| {
                                // Search in project name and description
                                let name_match = p.name.to_lowercase().contains(&query_lower);
                                let desc_match = p
                                    .description
                                    .as_ref()
                                    .map(|d| d.to_lowercase().contains(&query_lower))
                                    .unwrap_or(false);

                                // Search in git repository info
                                let git_match = if let Some(ref git) = p.git_repository {
                                    git.owner.to_lowercase().contains(&query_lower)
                                        || git.repo.to_lowercase().contains(&query_lower)
                                        || git.url.to_lowercase().contains(&query_lower)
                                } else {
                                    false
                                };

                                name_match || desc_match || git_match
                            });
                        }

                        // Filter by tags
                        if let Some(filter_tags) = &request.tags {
                            if !filter_tags.is_empty() {
                                projects.retain(|p| {
                                    if let Some(project_tags) = &p.tags {
                                        filter_tags.iter().any(|tag| project_tags.contains(tag))
                                    } else {
                                        false
                                    }
                                });
                            }
                        }

                        // Filter by priority
                        if let Some(priority_str) = &request.priority {
                            let priority_filter = match priority_str.as_str() {
                                "high" => Priority::High,
                                "medium" => Priority::Medium,
                                "low" => Priority::Low,
                                _ => {
                                    // Don't filter if invalid priority
                                    Priority::Medium
                                }
                            };
                            if priority_str != "all"
                                && ["high", "medium", "low"].contains(&priority_str.as_str())
                            {
                                projects.retain(|p| p.priority == priority_filter);
                            }
                        }
                    }

                    let response = json!({
                        "projects": projects,
                        "total": projects.len()
                    });

                    serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                        json!({ "error": format!("Serialization error: {}", e) }).to_string()
                    })
                }
                Err(e) => json!({ "error": format!("Failed to get projects: {}", e) }).to_string(),
            }
        }
        _ => json!({ "error": format!("Invalid action: {}", request.action) }).to_string(),
    }
}

async fn execute_project_manage_tool(
    request: ProjectManageRequest,
    context: &ToolContext,
) -> String {
    match request.action.as_str() {
        "create" => {
            if let (Some(name), Some(project_root)) = (&request.name, &request.project_root) {
                let project_data = ProjectCreateInput {
                    name: name.clone(),
                    project_root: project_root.clone(),
                    description: request.description.clone(),
                    setup_script: request.setup_script.clone(),
                    dev_script: request.dev_script.clone(),
                    cleanup_script: request.cleanup_script.clone(),
                    tags: request.tags.clone(),
                    status: request.status.as_ref().and_then(|s| match s.as_str() {
                        "active" => Some(ProjectStatus::Active),
                        "archived" => Some(ProjectStatus::Archived),
                        _ => None,
                    }),
                    priority: request.priority.as_ref().and_then(|p| match p.as_str() {
                        "high" => Some(Priority::High),
                        "medium" => Some(Priority::Medium),
                        "low" => Some(Priority::Low),
                        _ => None,
                    }),
                    rank: None,
                    task_source: None,
                    manual_tasks: None,
                    mcp_servers: None,
                };

                match context.projects_manager().create_project(project_data).await {
                    Ok(project) => {
                        let response = json!({
                            "success": true,
                            "project": project
                        });
                        serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                            json!({ "success": false, "error": format!("Serialization error: {}", e) }).to_string()
                        })
                    }
                    Err(e) => {
                        json!({ "success": false, "error": format!("Failed to create project: {}", e) }).to_string()
                    }
                }
            } else {
                json!({ "success": false, "error": "Name and projectRoot are required for creating a project" }).to_string()
            }
        }
        "update" => {
            if let Some(id) = &request.id {
                let updates = ProjectUpdateInput {
                    name: request.name.clone(),
                    project_root: request.project_root.clone(),
                    description: request.description.clone(),
                    setup_script: request.setup_script.clone(),
                    dev_script: request.dev_script.clone(),
                    cleanup_script: request.cleanup_script.clone(),
                    tags: request.tags.clone(),
                    status: request.status.as_ref().and_then(|s| match s.as_str() {
                        "active" => Some(ProjectStatus::Active),
                        "archived" => Some(ProjectStatus::Archived),
                        _ => None,
                    }),
                    priority: request.priority.as_ref().and_then(|p| match p.as_str() {
                        "high" => Some(Priority::High),
                        "medium" => Some(Priority::Medium),
                        "low" => Some(Priority::Low),
                        _ => None,
                    }),
                    rank: None,
                    task_source: None,
                    manual_tasks: None,
                    mcp_servers: None,
                };

                match context.projects_manager().update_project(id, updates).await {
                    Ok(project) => {
                        let response = json!({
                            "success": true,
                            "project": project
                        });
                        serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                            json!({ "success": false, "error": format!("Serialization error: {}", e) }).to_string()
                        })
                    }
                    Err(e) => {
                        json!({ "success": false, "error": format!("Failed to update project: {}", e) }).to_string()
                    }
                }
            } else {
                json!({ "success": false, "error": "ID is required for updating a project" })
                    .to_string()
            }
        }
        "delete" => {
            if let Some(id) = &request.id {
                match context.projects_manager().delete_project(id).await {
                    Ok(success) => {
                        let response = json!({
                            "success": success,
                            "message": if success { "Project deleted successfully" } else { "Project not found" }
                        });
                        serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
                            json!({ "success": false, "error": format!("Serialization error: {}", e) }).to_string()
                        })
                    }
                    Err(e) => {
                        json!({ "success": false, "error": format!("Failed to delete project: {}", e) }).to_string()
                    }
                }
            } else {
                json!({ "success": false, "error": "ID is required for deleting a project" })
                    .to_string()
            }
        }
        _ => json!({ "success": false, "error": format!("Invalid action: {}", request.action) })
            .to_string(),
    }
}
