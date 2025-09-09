use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::client::{CloudClient, CloudError, CloudProject, CloudResult};
use crate::subscription::CloudTier;

/// Sync direction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncDirection {
    LocalToCloud,
    CloudToLocal,
    Bidirectional,
}

/// Sync status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Synced,
    Pending,
    InProgress,
    Conflict,
    Error(String),
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub projects_synced: usize,
    pub projects_failed: usize,
    pub conflicts: Vec<SyncConflict>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

impl SyncResult {
    /// Check if sync was successful
    pub fn is_success(&self) -> bool {
        self.projects_failed == 0 && self.conflicts.is_empty()
    }

    /// Get a summary message
    pub fn summary(&self) -> String {
        if self.is_success() {
            format!("✅ Synced {} projects in {}ms", self.projects_synced, self.duration_ms)
        } else if !self.conflicts.is_empty() {
            format!("⚠️ Synced {} projects with {} conflicts", self.projects_synced, self.conflicts.len())
        } else {
            format!("❌ Synced {} projects, {} failed", self.projects_synced, self.projects_failed)
        }
    }
}

/// Sync conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub project_id: String,
    pub project_name: String,
    pub local_version: i32,
    pub cloud_version: i32,
    pub local_updated: DateTime<Utc>,
    pub cloud_updated: DateTime<Utc>,
    pub resolution: ConflictResolution,
}

/// How to resolve a sync conflict
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    UseLocal,
    UseCloud,
    Merge,
    Manual,
    Pending,
}

/// Cloud sync operations
#[async_trait]
pub trait CloudSync: Send + Sync {
    /// Sync all projects
    async fn sync_all(&self, direction: SyncDirection) -> CloudResult<SyncResult>;
    
    /// Sync a specific project
    async fn sync_project(&self, project_id: &str, direction: SyncDirection) -> CloudResult<SyncResult>;
    
    /// Check for sync conflicts
    async fn check_conflicts(&self) -> CloudResult<Vec<SyncConflict>>;
    
    /// Resolve a conflict
    async fn resolve_conflict(&self, conflict: &SyncConflict, resolution: ConflictResolution) -> CloudResult<()>;
}

/// Sync engine for managing cloud synchronization
pub struct SyncEngine {
    client: CloudClient,
    local_storage: Arc<RwLock<LocalStorage>>,
    sync_state: Arc<RwLock<SyncState>>,
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new(client: CloudClient) -> Self {
        Self {
            client,
            local_storage: Arc::new(RwLock::new(LocalStorage::new())),
            sync_state: Arc::new(RwLock::new(SyncState::default())),
        }
    }

    /// Sync all projects to cloud
    pub async fn sync_all(&self) -> CloudResult<SyncResult> {
        let start = std::time::Instant::now();
        let mut synced = 0;
        let mut failed = 0;
        let mut conflicts = Vec::new();

        // Check subscription limits
        let subscription = self.client.subscription();
        if subscription.tier == CloudTier::Free && !subscription.auto_sync_enabled {
            println!("ℹ️ Manual sync for free tier");
        }

        // Get local projects
        let local_storage = self.local_storage.read().await;
        let local_projects = local_storage.list_projects().await?;

        // Get cloud projects for comparison
        let cloud_projects = self.client.list_projects().await?;
        let cloud_map: std::collections::HashMap<_, _> = cloud_projects
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect();

        // Sync each local project
        for local_project in local_projects {
            match cloud_map.get(&local_project.id) {
                Some(cloud_project) => {
                    // Check for conflicts
                    if local_project.local_version != cloud_project.cloud_version {
                        conflicts.push(SyncConflict {
                            project_id: local_project.id.clone(),
                            project_name: local_project.name.clone(),
                            local_version: local_project.local_version,
                            cloud_version: cloud_project.cloud_version,
                            local_updated: local_project.updated_at,
                            cloud_updated: cloud_project.updated_at,
                            resolution: ConflictResolution::Pending,
                        });
                    } else {
                        // Update existing project
                        match self.client.update_project(&local_project).await {
                            Ok(_) => synced += 1,
                            Err(e) => {
                                eprintln!("Failed to sync project {}: {}", local_project.name, e);
                                failed += 1;
                            }
                        }
                    }
                }
                None => {
                    // New project, sync to cloud
                    match self.client.sync_project(&local_project).await {
                        Ok(_) => synced += 1,
                        Err(e) => {
                            eprintln!("Failed to sync project {}: {}", local_project.name, e);
                            failed += 1;
                        }
                    }
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(SyncResult {
            projects_synced: synced,
            projects_failed: failed,
            conflicts,
            duration_ms,
            timestamp: Utc::now(),
        })
    }

    /// Sync a specific project
    pub async fn sync_project(&self, project_id: &str) -> CloudResult<SyncResult> {
        let start = std::time::Instant::now();
        
        let local_storage = self.local_storage.read().await;
        let local_project = local_storage.get_project(project_id).await?
            .ok_or_else(|| CloudError::NotFound(format!("Project {} not found locally", project_id)))?;

        // Try to get cloud version
        let cloud_project = self.client.get_project(project_id).await.ok();

        let mut conflicts = Vec::new();
        let mut synced = 0;
        let mut failed = 0;

        if let Some(cloud_project) = cloud_project {
            // Check for conflicts
            if local_project.local_version != cloud_project.cloud_version {
                conflicts.push(SyncConflict {
                    project_id: local_project.id.clone(),
                    project_name: local_project.name.clone(),
                    local_version: local_project.local_version,
                    cloud_version: cloud_project.cloud_version,
                    local_updated: local_project.updated_at,
                    cloud_updated: cloud_project.updated_at,
                    resolution: ConflictResolution::Pending,
                });
            } else {
                // Update existing
                match self.client.update_project(&local_project).await {
                    Ok(_) => synced = 1,
                    Err(_) => failed = 1,
                }
            }
        } else {
            // New to cloud
            match self.client.sync_project(&local_project).await {
                Ok(_) => synced = 1,
                Err(_) => failed = 1,
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(SyncResult {
            projects_synced: synced,
            projects_failed: failed,
            conflicts,
            duration_ms,
            timestamp: Utc::now(),
        })
    }

    /// Restore all projects from cloud
    pub async fn restore_all(&self) -> CloudResult<Vec<CloudProject>> {
        let cloud_projects = self.client.list_projects().await?;
        
        let mut local_storage = self.local_storage.write().await;
        for project in &cloud_projects {
            local_storage.save_project(project).await?;
        }

        println!("✅ Restored {} projects from cloud", cloud_projects.len());
        Ok(cloud_projects)
    }

    /// Get sync status for all projects
    pub async fn get_sync_status(&self) -> CloudResult<Vec<ProjectSyncStatus>> {
        let local_storage = self.local_storage.read().await;
        let local_projects = local_storage.list_projects().await?;
        
        let mut statuses = Vec::new();
        for project in local_projects {
            statuses.push(ProjectSyncStatus {
                project_id: project.id.clone(),
                project_name: project.name.clone(),
                status: if project.sync_status == "synced" {
                    SyncStatus::Synced
                } else {
                    SyncStatus::Pending
                },
                last_synced: project.last_synced_at,
            });
        }

        Ok(statuses)
    }
}

/// Status of a project's sync state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSyncStatus {
    pub project_id: String,
    pub project_name: String,
    pub status: SyncStatus,
    pub last_synced: Option<DateTime<Utc>>,
}

/// Sync state tracking
#[derive(Debug, Default)]
struct SyncState {
    last_full_sync: Option<DateTime<Utc>>,
    in_progress: bool,
    pending_conflicts: Vec<SyncConflict>,
}

/// Local storage interface (minimal, to avoid circular dependencies)
struct LocalStorage;

impl LocalStorage {
    fn new() -> Self {
        Self
    }

    async fn list_projects(&self) -> CloudResult<Vec<CloudProject>> {
        // This would integrate with the actual local SQLite storage
        // For now, return empty to compile
        Ok(Vec::new())
    }

    async fn get_project(&self, _id: &str) -> CloudResult<Option<CloudProject>> {
        // This would integrate with the actual local SQLite storage
        Ok(None)
    }

    async fn save_project(&mut self, _project: &CloudProject) -> CloudResult<()> {
        // This would integrate with the actual local SQLite storage
        Ok(())
    }
}