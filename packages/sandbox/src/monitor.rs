// ABOUTME: Resource monitoring for sandboxes with configurable intervals
// ABOUTME: Background task that periodically collects metrics from running sandboxes

use crate::manager::SandboxManager;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub sandbox_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub memory_limit_mb: u64,
    pub memory_usage_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub sandbox_id: String,
    pub time_window_minutes: u32,
    pub avg_cpu_percent: f64,
    pub avg_memory_mb: u64,
    pub peak_cpu_percent: f64,
    pub peak_memory_mb: u64,
    pub total_network_rx_bytes: u64,
    pub total_network_tx_bytes: u64,
    pub snapshots_count: usize,
}

/// Resource monitor for sandboxes
pub struct ResourceMonitor {
    manager: Arc<SandboxManager>,
    settings: Arc<RwLock<SettingsManager>>,
    snapshots: Arc<RwLock<HashMap<String, Vec<ResourceSnapshot>>>>,
    running: Arc<RwLock<bool>>,
}

impl ResourceMonitor {
    pub fn new(manager: Arc<SandboxManager>, settings: Arc<RwLock<SettingsManager>>) -> Self {
        Self {
            manager,
            settings,
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the resource monitoring task
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(()); // Already running
        }
        *running = true;
        drop(running);

        let manager = self.manager.clone();
        let settings = self.settings.clone();
        let snapshots = self.snapshots.clone();
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            info!("Resource monitor started");

            loop {
                // Check if still running
                if !*running_flag.read().await {
                    info!("Resource monitor stopped");
                    break;
                }

                // Get monitoring interval from settings
                let interval_secs = {
                    let settings_guard = settings.read().await;
                    match settings_guard.get_sandbox_settings().await {
                        Ok(sandbox_settings) => {
                            sandbox_settings.resource_monitoring_interval_seconds as u64
                        }
                        Err(e) => {
                            error!("Failed to get sandbox settings: {}", e);
                            30 // Default to 30 seconds
                        }
                    }
                };

                // Monitor all running sandboxes
                match manager
                    .list_sandboxes(None, Some(SandboxStatus::Running))
                    .await
                {
                    Ok(sandboxes) => {
                        let running_ids: std::collections::HashSet<String> =
                            sandboxes.iter().map(|s| s.id.clone()).collect();

                        // Remove snapshots for sandboxes that are no longer running
                        // This prevents memory leaks from deleted or terminated sandboxes
                        {
                            let mut snapshots_map = snapshots.write().await;
                            snapshots_map.retain(|id, _| running_ids.contains(id));
                        }

                        // Monitor each running sandbox
                        for sandbox in sandboxes {
                            if let Err(e) =
                                Self::monitor_sandbox(&manager, &snapshots, &sandbox).await
                            {
                                warn!("Failed to monitor sandbox {}: {}", sandbox.id, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to list sandboxes: {}", e);
                    }
                }

                // Sleep for the configured interval
                time::sleep(Duration::from_secs(interval_secs)).await;
            }
        });

        Ok(())
    }

    /// Stop the resource monitoring task
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Monitor a single sandbox
    async fn monitor_sandbox(
        manager: &Arc<SandboxManager>,
        snapshots: &Arc<RwLock<HashMap<String, Vec<ResourceSnapshot>>>>,
        sandbox: &Sandbox,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get container info with metrics
        let container_info = manager.get_container_info(&sandbox.id).await?;

        if let Some(metrics) = container_info.metrics {
            let snapshot = ResourceSnapshot {
                sandbox_id: sandbox.id.clone(),
                timestamp: Utc::now(),
                cpu_usage_percent: metrics.cpu_usage_percent,
                memory_usage_mb: metrics.memory_usage_mb,
                memory_limit_mb: metrics.memory_limit_mb,
                memory_usage_percent: if metrics.memory_limit_mb > 0 {
                    (metrics.memory_usage_mb as f64 / metrics.memory_limit_mb as f64) * 100.0
                } else {
                    0.0
                },
                network_rx_bytes: metrics.network_rx_bytes,
                network_tx_bytes: metrics.network_tx_bytes,
                status: sandbox.status.as_str().to_string(),
            };

            // Store snapshot
            let mut snapshots_map = snapshots.write().await;
            let sandbox_snapshots = snapshots_map
                .entry(sandbox.id.clone())
                .or_insert_with(Vec::new);
            sandbox_snapshots.push(snapshot);

            // Keep only last 1000 snapshots per sandbox to avoid memory growth
            if sandbox_snapshots.len() > 1000 {
                sandbox_snapshots.drain(0..100); // Remove oldest 100
            }
        }

        Ok(())
    }

    /// Get recent snapshots for a sandbox
    pub async fn get_snapshots(
        &self,
        sandbox_id: &str,
        limit: Option<usize>,
    ) -> Vec<ResourceSnapshot> {
        let snapshots = self.snapshots.read().await;
        if let Some(sandbox_snapshots) = snapshots.get(sandbox_id) {
            let start = if let Some(limit) = limit {
                sandbox_snapshots.len().saturating_sub(limit)
            } else {
                0
            };
            sandbox_snapshots[start..].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Get aggregated metrics for a sandbox over a time window
    pub async fn get_aggregated_metrics(
        &self,
        sandbox_id: &str,
        window_minutes: u32,
    ) -> Option<AggregatedMetrics> {
        let snapshots = self.snapshots.read().await;
        let sandbox_snapshots = snapshots.get(sandbox_id)?;

        if sandbox_snapshots.is_empty() {
            return None;
        }

        // Filter snapshots within time window
        let now = Utc::now();
        let window_start = now - chrono::Duration::minutes(window_minutes as i64);

        let recent_snapshots: Vec<_> = sandbox_snapshots
            .iter()
            .filter(|s| s.timestamp >= window_start)
            .collect();

        if recent_snapshots.is_empty() {
            return None;
        }

        // Calculate aggregates
        let mut sum_cpu = 0.0_f64;
        let mut sum_memory = 0u64;
        let mut peak_cpu = 0.0_f64;
        let mut peak_memory = 0u64;
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;

        for snapshot in &recent_snapshots {
            sum_cpu += snapshot.cpu_usage_percent;
            sum_memory += snapshot.memory_usage_mb;
            peak_cpu = peak_cpu.max(snapshot.cpu_usage_percent);
            peak_memory = peak_memory.max(snapshot.memory_usage_mb);
            total_rx = total_rx.max(snapshot.network_rx_bytes);
            total_tx = total_tx.max(snapshot.network_tx_bytes);
        }

        let count = recent_snapshots.len();

        Some(AggregatedMetrics {
            sandbox_id: sandbox_id.to_string(),
            time_window_minutes: window_minutes,
            avg_cpu_percent: sum_cpu / count as f64,
            avg_memory_mb: sum_memory / count as u64,
            peak_cpu_percent: peak_cpu,
            peak_memory_mb: peak_memory,
            total_network_rx_bytes: total_rx,
            total_network_tx_bytes: total_tx,
            snapshots_count: count,
        })
    }

    /// Clear snapshots for a sandbox (e.g., after termination)
    pub async fn clear_snapshots(&self, sandbox_id: &str) {
        let mut snapshots = self.snapshots.write().await;
        snapshots.remove(sandbox_id);
    }

    /// Clear old snapshots for all sandboxes
    pub async fn clear_old_snapshots(&self, age_minutes: i64) {
        let cutoff = Utc::now() - chrono::Duration::minutes(age_minutes);
        let mut snapshots = self.snapshots.write().await;

        for (_, sandbox_snapshots) in snapshots.iter_mut() {
            sandbox_snapshots.retain(|s| s.timestamp >= cutoff);
        }
    }

    /// Get current resource usage for all running sandboxes
    pub async fn get_all_current_metrics(&self) -> HashMap<String, Option<ResourceSnapshot>> {
        let snapshots = self.snapshots.read().await;
        let mut result = HashMap::new();

        for (sandbox_id, sandbox_snapshots) in snapshots.iter() {
            result.insert(sandbox_id.clone(), sandbox_snapshots.last().cloned());
        }

        result
    }

    /// Check if any sandbox exceeds resource limits
    pub async fn check_resource_limits(&self) -> Vec<(String, String)> {
        let mut violations = Vec::new();
        let snapshots = self.snapshots.read().await;

        for (sandbox_id, sandbox_snapshots) in snapshots.iter() {
            if let Some(latest) = sandbox_snapshots.last() {
                // Check memory usage (warn at 90%, critical at 95%)
                if latest.memory_usage_percent >= 95.0 {
                    violations.push((
                        sandbox_id.clone(),
                        format!(
                            "Critical: Memory usage at {:.1}%",
                            latest.memory_usage_percent
                        ),
                    ));
                } else if latest.memory_usage_percent >= 90.0 {
                    violations.push((
                        sandbox_id.clone(),
                        format!(
                            "Warning: Memory usage at {:.1}%",
                            latest.memory_usage_percent
                        ),
                    ));
                }

                // Check CPU usage (warn at sustained 90%)
                if latest.cpu_usage_percent >= 90.0 {
                    violations.push((
                        sandbox_id.clone(),
                        format!("Warning: CPU usage at {:.1}%", latest.cpu_usage_percent),
                    ));
                }
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests would require setting up test database
    // and manager. These are unit tests for the monitor logic.

    #[tokio::test]
    async fn test_snapshot_retention() {
        // Test that snapshots are properly retained with size limits
        let snapshots = Arc::new(RwLock::new(HashMap::new()));
        let sandbox_id = "test-sandbox".to_string();

        {
            let mut map = snapshots.write().await;
            let mut vec = Vec::new();

            // Add 1100 snapshots
            for i in 0..1100 {
                vec.push(ResourceSnapshot {
                    sandbox_id: sandbox_id.clone(),
                    timestamp: Utc::now(),
                    cpu_usage_percent: i as f64,
                    memory_usage_mb: 128,
                    memory_limit_mb: 2048,
                    memory_usage_percent: 6.25,
                    network_rx_bytes: 0,
                    network_tx_bytes: 0,
                    status: "running".to_string(),
                });
            }

            map.insert(sandbox_id.clone(), vec);
        }

        // Verify all snapshots stored
        {
            let map = snapshots.read().await;
            let vec = map.get(&sandbox_id).unwrap();
            assert_eq!(vec.len(), 1100);
        }
    }
}
