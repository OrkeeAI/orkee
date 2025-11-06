// ABOUTME: Cost calculation utilities for sandbox resource usage
// ABOUTME: Calculates costs based on provider pricing and resource consumption

use crate::storage::{Sandbox, SandboxExecution};
use crate::ProviderRegistry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub base_cost: f64,
    pub compute_cost: f64,
    pub memory_cost: f64,
    pub storage_cost: f64,
    pub network_cost: f64,
    pub gpu_cost: f64,
    pub total_cost: f64,
    pub hours_running: f64,
}

impl CostBreakdown {
    pub fn new() -> Self {
        Self {
            base_cost: 0.0,
            compute_cost: 0.0,
            memory_cost: 0.0,
            storage_cost: 0.0,
            network_cost: 0.0,
            gpu_cost: 0.0,
            total_cost: 0.0,
            hours_running: 0.0,
        }
    }
}

impl Default for CostBreakdown {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CostCalculator {
    registry: ProviderRegistry,
}

impl CostCalculator {
    pub fn new(registry: ProviderRegistry) -> Self {
        Self { registry }
    }

    /// Calculate cost for a sandbox based on runtime
    pub fn calculate_sandbox_cost(&self, sandbox: &Sandbox) -> Option<CostBreakdown> {
        let provider = self.registry.get(&sandbox.provider)?;

        // Calculate runtime hours
        let hours_running = self.calculate_runtime_hours(
            sandbox.started_at.as_ref()?,
            sandbox.stopped_at.as_ref().or(Some(&Utc::now())),
        );

        let mut breakdown = CostBreakdown::new();
        breakdown.hours_running = hours_running;

        // Base cost
        breakdown.base_cost = provider.pricing.base_cost;

        // Compute cost (per hour or per CPU hour)
        if let Some(per_hour) = provider.pricing.per_hour {
            breakdown.compute_cost = per_hour * hours_running;
        } else if let Some(per_cpu_hour) = provider.pricing.per_cpu_hour {
            breakdown.compute_cost = per_cpu_hour * sandbox.cpu_cores as f64 * hours_running;
        }

        // Memory cost
        if let Some(per_gb_memory) = provider.pricing.per_gb_memory {
            let memory_gb = sandbox.memory_mb as f64 / 1024.0;
            breakdown.memory_cost = per_gb_memory * memory_gb * hours_running;
        } else if let Some(per_gb_hour) = provider.pricing.per_gb_hour {
            let memory_gb = sandbox.memory_mb as f64 / 1024.0;
            breakdown.memory_cost = per_gb_hour * memory_gb * hours_running;
        }

        // Storage cost
        if let Some(per_gb_storage) = provider.pricing.per_gb_storage {
            breakdown.storage_cost = per_gb_storage * sandbox.storage_gb as f64 * hours_running;
        }

        // GPU cost
        if sandbox.gpu_enabled {
            if let Some(gpu_per_hour) = &provider.pricing.gpu_per_hour {
                if let Some(gpu_model) = &sandbox.gpu_model {
                    if let Some(gpu_cost) = gpu_per_hour.get(gpu_model) {
                        breakdown.gpu_cost = gpu_cost * hours_running;
                    }
                }
            }
        }

        // Calculate total
        breakdown.total_cost = breakdown.base_cost
            + breakdown.compute_cost
            + breakdown.memory_cost
            + breakdown.storage_cost
            + breakdown.network_cost
            + breakdown.gpu_cost;

        Some(breakdown)
    }

    /// Calculate cost for an execution based on resource usage
    pub fn calculate_execution_cost(
        &self,
        sandbox: &Sandbox,
        execution: &SandboxExecution,
    ) -> Option<CostBreakdown> {
        let provider = self.registry.get(&sandbox.provider)?;

        // Calculate execution hours
        let hours_running = if let (Some(started), Some(completed)) = (
            execution.started_at.as_ref(),
            execution.completed_at.as_ref(),
        ) {
            self.calculate_runtime_hours(started, Some(completed))
        } else {
            return None;
        };

        let mut breakdown = CostBreakdown::new();
        breakdown.hours_running = hours_running;

        // For executions, we use execution-specific pricing if available
        if let Some(per_execution) = provider.pricing.per_execution {
            breakdown.base_cost = per_execution;
        }

        // Compute cost based on CPU time if available
        if let Some(cpu_time_secs) = execution.cpu_time_seconds {
            let cpu_hours = cpu_time_secs / 3600.0;
            if let Some(per_cpu_hour) = provider.pricing.per_cpu_hour {
                breakdown.compute_cost = per_cpu_hour * sandbox.cpu_cores as f64 * cpu_hours;
            } else if let Some(per_hour) = provider.pricing.per_hour {
                breakdown.compute_cost = per_hour * cpu_hours;
            }
        } else {
            // Fallback to wall-clock time
            if let Some(per_hour) = provider.pricing.per_hour {
                breakdown.compute_cost = per_hour * hours_running;
            } else if let Some(per_cpu_hour) = provider.pricing.per_cpu_hour {
                breakdown.compute_cost = per_cpu_hour * sandbox.cpu_cores as f64 * hours_running;
            }
        }

        // Memory cost based on peak usage if available
        if let Some(memory_peak_mb) = execution.memory_peak_mb {
            let memory_gb = memory_peak_mb as f64 / 1024.0;
            if let Some(per_gb_hour) = provider.pricing.per_gb_hour {
                breakdown.memory_cost = per_gb_hour * memory_gb * hours_running;
            } else if let Some(per_gb_memory) = provider.pricing.per_gb_memory {
                breakdown.memory_cost = per_gb_memory * memory_gb * hours_running;
            }
        } else {
            // Fallback to sandbox memory limit
            let memory_gb = sandbox.memory_mb as f64 / 1024.0;
            if let Some(per_gb_hour) = provider.pricing.per_gb_hour {
                breakdown.memory_cost = per_gb_hour * memory_gb * hours_running;
            } else if let Some(per_gb_memory) = provider.pricing.per_gb_memory {
                breakdown.memory_cost = per_gb_memory * memory_gb * hours_running;
            }
        }

        // Calculate total
        breakdown.total_cost = breakdown.base_cost
            + breakdown.compute_cost
            + breakdown.memory_cost
            + breakdown.storage_cost
            + breakdown.network_cost
            + breakdown.gpu_cost;

        Some(breakdown)
    }

    /// Calculate aggregated costs for multiple sandboxes
    pub fn calculate_total_cost(&self, sandboxes: &[Sandbox]) -> CostBreakdown {
        let mut total = CostBreakdown::new();

        for sandbox in sandboxes {
            if let Some(breakdown) = self.calculate_sandbox_cost(sandbox) {
                total.base_cost += breakdown.base_cost;
                total.compute_cost += breakdown.compute_cost;
                total.memory_cost += breakdown.memory_cost;
                total.storage_cost += breakdown.storage_cost;
                total.network_cost += breakdown.network_cost;
                total.gpu_cost += breakdown.gpu_cost;
                total.total_cost += breakdown.total_cost;
                total.hours_running += breakdown.hours_running;
            }
        }

        total
    }

    /// Estimate cost for a sandbox based on configuration
    #[allow(clippy::too_many_arguments)]
    pub fn estimate_cost(
        &self,
        provider_id: &str,
        cpu_cores: f32,
        memory_mb: u32,
        storage_gb: u32,
        gpu_enabled: bool,
        gpu_model: Option<&str>,
        hours: f64,
    ) -> Option<CostBreakdown> {
        let provider = self.registry.get(provider_id)?;

        let mut breakdown = CostBreakdown::new();
        breakdown.hours_running = hours;

        // Base cost
        breakdown.base_cost = provider.pricing.base_cost;

        // Compute cost
        if let Some(per_hour) = provider.pricing.per_hour {
            breakdown.compute_cost = per_hour * hours;
        } else if let Some(per_cpu_hour) = provider.pricing.per_cpu_hour {
            breakdown.compute_cost = per_cpu_hour * cpu_cores as f64 * hours;
        }

        // Memory cost
        let memory_gb = memory_mb as f64 / 1024.0;
        if let Some(per_gb_memory) = provider.pricing.per_gb_memory {
            breakdown.memory_cost = per_gb_memory * memory_gb * hours;
        } else if let Some(per_gb_hour) = provider.pricing.per_gb_hour {
            breakdown.memory_cost = per_gb_hour * memory_gb * hours;
        }

        // Storage cost
        if let Some(per_gb_storage) = provider.pricing.per_gb_storage {
            breakdown.storage_cost = per_gb_storage * storage_gb as f64 * hours;
        }

        // GPU cost
        if gpu_enabled {
            if let Some(gpu_per_hour) = &provider.pricing.gpu_per_hour {
                if let Some(model) = gpu_model {
                    if let Some(gpu_cost) = gpu_per_hour.get(model) {
                        breakdown.gpu_cost = gpu_cost * hours;
                    }
                }
            }
        }

        // Calculate total
        breakdown.total_cost = breakdown.base_cost
            + breakdown.compute_cost
            + breakdown.memory_cost
            + breakdown.storage_cost
            + breakdown.network_cost
            + breakdown.gpu_cost;

        Some(breakdown)
    }

    /// Check if cost is within limit
    pub fn is_within_limit(&self, cost: f64, limit: f64) -> bool {
        cost <= limit
    }

    /// Calculate cost alert threshold reached percentage
    pub fn cost_threshold_percentage(&self, current_cost: f64, limit: f64) -> f64 {
        if limit == 0.0 {
            return 0.0;
        }
        (current_cost / limit) * 100.0
    }

    /// Helper to calculate runtime hours between two timestamps
    fn calculate_runtime_hours(
        &self,
        started_at: &DateTime<Utc>,
        ended_at: Option<&DateTime<Utc>>,
    ) -> f64 {
        let now = Utc::now();
        let end = ended_at.unwrap_or(&now);
        let duration = end.signed_duration_since(*started_at);
        duration.num_seconds() as f64 / 3600.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Provider, ProviderCapabilities, ProviderLimits, ProviderPricing};
    use chrono::Duration;
    use std::collections::HashMap;

    fn create_test_provider() -> Provider {
        Provider {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
            display_name: "Test".to_string(),
            description: "Test provider".to_string(),
            provider_type: "test".to_string(),
            capabilities: ProviderCapabilities {
                gpu: true,
                persistent_storage: true,
                public_urls: false,
                ssh_access: false,
                auto_scaling: false,
                regions: vec![],
            },
            pricing: ProviderPricing {
                base_cost: 0.0,
                per_hour: Some(0.10),
                per_gb_memory: Some(0.01),
                per_vcpu: None,
                gpu_per_hour: Some({
                    let mut map = HashMap::new();
                    map.insert("T4".to_string(), 0.65);
                    map
                }),
                per_million_requests: None,
                per_gb_bandwidth: None,
                included_requests: None,
                included_bandwidth_gb: None,
                per_cpu_hour: None,
                per_gb_hour: None,
                per_execution: None,
                per_gb_storage: Some(0.10),
            },
            limits: ProviderLimits {
                max_memory_gb: Some(32),
                max_vcpus: Some(16),
                max_storage_gb: Some(100),
                max_runtime_hours: None,
                max_memory_mb: None,
                max_execution_time_ms: None,
                max_script_size_kb: None,
                max_runtime_seconds: None,
                max_file_size_mb: None,
            },
            default_config: serde_json::json!({}),
            is_available: true,
            requires_auth: false,
            auth_fields: None,
        }
    }

    #[test]
    fn test_estimate_cost() {
        let mut registry = ProviderRegistry::new().unwrap();
        let calculator = CostCalculator::new(registry);

        // Test estimation (simplified - would need to add test provider to registry)
        // This is a placeholder test
    }

    #[test]
    fn test_cost_threshold_percentage() {
        let registry = ProviderRegistry::new().unwrap();
        let calculator = CostCalculator::new(registry);

        assert_eq!(calculator.cost_threshold_percentage(5.0, 10.0), 50.0);
        assert_eq!(calculator.cost_threshold_percentage(10.0, 10.0), 100.0);
        assert_eq!(calculator.cost_threshold_percentage(15.0, 10.0), 150.0);
    }

    #[test]
    fn test_is_within_limit() {
        let registry = ProviderRegistry::new().unwrap();
        let calculator = CostCalculator::new(registry);

        assert!(calculator.is_within_limit(5.0, 10.0));
        assert!(calculator.is_within_limit(10.0, 10.0));
        assert!(!calculator.is_within_limit(15.0, 10.0));
    }
}
