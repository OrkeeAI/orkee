// ABOUTME: Health checking for sandboxes with configurable intervals
// ABOUTME: Performs periodic health checks and handles unhealthy sandboxes

use crate::manager::SandboxManager;
use crate::providers::ContainerStatus;
use crate::settings::SettingsManager;
use crate::storage::{Sandbox, SandboxStatus};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub sandbox_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub container_status: Option<String>,
    pub response_time_ms: Option<u64>,
}

/// Health checker for sandboxes
pub struct HealthChecker {
    manager: Arc<SandboxManager>,
    settings: Arc<RwLock<SettingsManager>>,
    checks: Arc<RwLock<HashMap<String, Vec<HealthCheck>>>>,
    running: Arc<RwLock<bool>>,
}

impl HealthChecker {
    pub fn new(manager: Arc<SandboxManager>, settings: Arc<RwLock<SettingsManager>>) -> Self {
        Self {
            manager,
            settings,
            checks: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the health checking task
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(()); // Already running
        }
        *running = true;
        drop(running);

        let manager = self.manager.clone();
        let settings = self.settings.clone();
        let checks = self.checks.clone();
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            info!("Health checker started");

            loop {
                // Check if still running
                if !*running_flag.read().await {
                    info!("Health checker stopped");
                    break;
                }

                // Get health check interval from settings
                let interval_secs = {
                    let settings_guard = settings.read().await;
                    match settings_guard.get_sandbox_settings().await {
                        Ok(sandbox_settings) => {
                            sandbox_settings.health_check_interval_seconds as u64
                        }
                        Err(e) => {
                            error!("Failed to get sandbox settings: {}", e);
                            60 // Default to 60 seconds
                        }
                    }
                };

                // Check all running sandboxes
                match manager
                    .list_sandboxes(None, Some(SandboxStatus::Running))
                    .await
                {
                    Ok(sandboxes) => {
                        for sandbox in sandboxes {
                            if let Err(e) = Self::check_sandbox(&manager, &checks, &sandbox).await {
                                warn!("Failed to health check sandbox {}: {}", sandbox.id, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to list sandboxes: {}", e);
                    }
                }

                // Check for stuck sandboxes (in transition states)
                match manager
                    .list_sandboxes(None, Some(SandboxStatus::Starting))
                    .await
                {
                    Ok(sandboxes) => {
                        for sandbox in sandboxes {
                            Self::check_stuck_sandbox(&manager, &checks, &sandbox).await;
                        }
                    }
                    Err(e) => {
                        error!("Failed to list starting sandboxes: {}", e);
                    }
                }

                match manager
                    .list_sandboxes(None, Some(SandboxStatus::Stopping))
                    .await
                {
                    Ok(sandboxes) => {
                        for sandbox in sandboxes {
                            Self::check_stuck_sandbox(&manager, &checks, &sandbox).await;
                        }
                    }
                    Err(e) => {
                        error!("Failed to list stopping sandboxes: {}", e);
                    }
                }

                // Sleep for the configured interval
                time::sleep(Duration::from_secs(interval_secs)).await;
            }
        });

        Ok(())
    }

    /// Stop the health checking task
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Check health of a single sandbox
    async fn check_sandbox(
        manager: &Arc<SandboxManager>,
        checks: &Arc<RwLock<HashMap<String, Vec<HealthCheck>>>>,
        sandbox: &Sandbox,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Get container info from provider
        let container_info = match manager.get_container_info(&sandbox.id).await {
            Ok(info) => info,
            Err(e) => {
                // Container not found or provider error
                let check = HealthCheck {
                    sandbox_id: sandbox.id.clone(),
                    timestamp: Utc::now(),
                    status: HealthStatus::Unhealthy,
                    message: Some(format!("Failed to get container info: {}", e)),
                    container_status: None,
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                };

                Self::store_check(checks, &sandbox.id, check).await;
                return Err(e.into());
            }
        };

        let elapsed_ms = start.elapsed().as_millis() as u64;

        // Determine health status based on container status
        let (health_status, message) = match container_info.status {
            ContainerStatus::Running => {
                // Check if response time is acceptable (< 5 seconds)
                if elapsed_ms > 5000 {
                    (
                        HealthStatus::Degraded,
                        Some("Slow response time".to_string()),
                    )
                } else {
                    (HealthStatus::Healthy, None)
                }
            }
            ContainerStatus::Paused => (
                HealthStatus::Degraded,
                Some("Container is paused".to_string()),
            ),
            ContainerStatus::Dead | ContainerStatus::Error(_) => (
                HealthStatus::Unhealthy,
                Some(format!(
                    "Container in error state: {:?}",
                    container_info.status
                )),
            ),
            _ => (
                HealthStatus::Unknown,
                Some(format!(
                    "Unexpected container status: {:?}",
                    container_info.status
                )),
            ),
        };

        let check = HealthCheck {
            sandbox_id: sandbox.id.clone(),
            timestamp: Utc::now(),
            status: health_status,
            message,
            container_status: Some(format!("{:?}", container_info.status)),
            response_time_ms: Some(elapsed_ms),
        };

        Self::store_check(checks, &sandbox.id, check).await;
        Ok(())
    }

    /// Check for sandboxes stuck in transition states
    async fn check_stuck_sandbox(
        _manager: &Arc<SandboxManager>,
        checks: &Arc<RwLock<HashMap<String, Vec<HealthCheck>>>>,
        sandbox: &Sandbox,
    ) {
        // Check if sandbox has been in transition state for too long (> 5 minutes)
        let now = Utc::now();
        let state_duration = if let Some(started_at) = sandbox.started_at {
            now.signed_duration_since(started_at)
        } else {
            now.signed_duration_since(sandbox.created_at)
        };

        if state_duration.num_minutes() > 5 {
            let check = HealthCheck {
                sandbox_id: sandbox.id.clone(),
                timestamp: now,
                status: HealthStatus::Unhealthy,
                message: Some(format!(
                    "Sandbox stuck in {:?} state for {} minutes",
                    sandbox.status,
                    state_duration.num_minutes()
                )),
                container_status: None,
                response_time_ms: None,
            };

            Self::store_check(checks, &sandbox.id, check).await;

            // Optionally attempt recovery
            warn!(
                "Sandbox {} stuck in {:?} state for {} minutes",
                sandbox.id,
                sandbox.status,
                state_duration.num_minutes()
            );
        }
    }

    /// Store a health check result
    async fn store_check(
        checks: &Arc<RwLock<HashMap<String, Vec<HealthCheck>>>>,
        sandbox_id: &str,
        check: HealthCheck,
    ) {
        let mut checks_map = checks.write().await;
        let sandbox_checks = checks_map
            .entry(sandbox_id.to_string())
            .or_insert_with(Vec::new);
        sandbox_checks.push(check);

        // Keep only last 100 checks per sandbox
        if sandbox_checks.len() > 100 {
            sandbox_checks.drain(0..10); // Remove oldest 10
        }
    }

    /// Get recent health checks for a sandbox
    pub async fn get_health_checks(
        &self,
        sandbox_id: &str,
        limit: Option<usize>,
    ) -> Vec<HealthCheck> {
        let checks = self.checks.read().await;
        if let Some(sandbox_checks) = checks.get(sandbox_id) {
            let start = if let Some(limit) = limit {
                sandbox_checks.len().saturating_sub(limit)
            } else {
                0
            };
            sandbox_checks[start..].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Get latest health status for a sandbox
    pub async fn get_latest_health_status(&self, sandbox_id: &str) -> Option<HealthCheck> {
        let checks = self.checks.read().await;
        checks
            .get(sandbox_id)
            .and_then(|sandbox_checks| sandbox_checks.last().cloned())
    }

    /// Get health summary for all sandboxes
    pub async fn get_health_summary(&self) -> HashMap<String, HealthStatus> {
        let checks = self.checks.read().await;
        let mut summary = HashMap::new();

        for (sandbox_id, sandbox_checks) in checks.iter() {
            if let Some(latest) = sandbox_checks.last() {
                summary.insert(sandbox_id.clone(), latest.status.clone());
            }
        }

        summary
    }

    /// Get count of unhealthy sandboxes
    pub async fn get_unhealthy_count(&self) -> usize {
        let summary = self.get_health_summary().await;
        summary
            .values()
            .filter(|s| matches!(s, HealthStatus::Unhealthy))
            .count()
    }

    /// Get count of degraded sandboxes
    pub async fn get_degraded_count(&self) -> usize {
        let summary = self.get_health_summary().await;
        summary
            .values()
            .filter(|s| matches!(s, HealthStatus::Degraded))
            .count()
    }

    /// Clear health checks for a sandbox
    pub async fn clear_checks(&self, sandbox_id: &str) {
        let mut checks = self.checks.write().await;
        checks.remove(sandbox_id);
    }

    /// Clear old health checks
    pub async fn clear_old_checks(&self, age_minutes: i64) {
        let cutoff = Utc::now() - chrono::Duration::minutes(age_minutes);
        let mut checks = self.checks.write().await;

        for (_, sandbox_checks) in checks.iter_mut() {
            sandbox_checks.retain(|c| c.timestamp >= cutoff);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_storage() {
        let checks = Arc::new(RwLock::new(HashMap::new()));
        let sandbox_id = "test-sandbox";

        let check = HealthCheck {
            sandbox_id: sandbox_id.to_string(),
            timestamp: Utc::now(),
            status: HealthStatus::Healthy,
            message: None,
            container_status: Some("Running".to_string()),
            response_time_ms: Some(100),
        };

        HealthChecker::store_check(&checks, sandbox_id, check.clone()).await;

        let stored_checks = checks.read().await;
        let sandbox_checks = stored_checks.get(sandbox_id).unwrap();
        assert_eq!(sandbox_checks.len(), 1);
        assert_eq!(sandbox_checks[0].status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_check_retention() {
        let checks = Arc::new(RwLock::new(HashMap::new()));
        let sandbox_id = "test-sandbox";

        // Add 110 checks
        for i in 0..110 {
            let check = HealthCheck {
                sandbox_id: sandbox_id.to_string(),
                timestamp: Utc::now(),
                status: if i % 2 == 0 {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Degraded
                },
                message: Some(format!("Check {}", i)),
                container_status: Some("Running".to_string()),
                response_time_ms: Some(100),
            };

            HealthChecker::store_check(&checks, sandbox_id, check).await;
        }

        // Should have pruned to 100 checks
        let stored_checks = checks.read().await;
        let sandbox_checks = stored_checks.get(sandbox_id).unwrap();
        assert_eq!(sandbox_checks.len(), 100);
    }
}
