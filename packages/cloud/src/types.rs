use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Progress information for sync operations
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub items_processed: usize,
    pub total_items: usize,
    pub percentage: f32,
    pub elapsed_seconds: u64,
    pub estimated_remaining_seconds: Option<u64>,
    pub current_item: Option<String>,
}

impl ProgressInfo {
    pub fn new(items_processed: usize, total_items: usize, elapsed_seconds: u64) -> Self {
        let percentage = if total_items > 0 {
            (items_processed as f32 / total_items as f32) * 100.0
        } else {
            0.0
        };

        let estimated_remaining_seconds = if items_processed > 0 && elapsed_seconds > 0 {
            let rate = items_processed as f64 / elapsed_seconds as f64;
            let remaining_items = total_items.saturating_sub(items_processed);
            Some((remaining_items as f64 / rate) as u64)
        } else {
            None
        };

        Self {
            items_processed,
            total_items,
            percentage,
            elapsed_seconds,
            estimated_remaining_seconds,
            current_item: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.items_processed >= self.total_items
    }

    pub fn with_current_item(mut self, item: String) -> Self {
        self.current_item = Some(item);
        self
    }
}

/// Callback type for progress updates
pub type ProgressCallback = Box<dyn Fn(ProgressInfo) + Send + Sync>;

/// Cloud operation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetadata {
    pub operation_id: String,
    pub operation_type: OperationType,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub user_id: String,
    pub machine_id: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Type of cloud operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Sync,
    Restore,
    Backup,
    Delete,
    Enable,
    Disable,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Sync => write!(f, "Sync"),
            OperationType::Restore => write!(f, "Restore"),
            OperationType::Backup => write!(f, "Backup"),
            OperationType::Delete => write!(f, "Delete"),
            OperationType::Enable => write!(f, "Enable"),
            OperationType::Disable => write!(f, "Disable"),
        }
    }
}

/// Machine identifier for tracking sync sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineId {
    pub id: String,
    pub hostname: String,
    pub platform: String,
}

impl MachineId {
    /// Generate a machine ID for this system
    pub fn current() -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let platform = std::env::consts::OS.to_string();

        Self {
            id,
            hostname,
            platform,
        }
    }
}

/// Usage event for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub machine_id: String,
}

impl UsageEvent {
    /// Create a new usage event
    pub fn new(event_type: String, event_data: serde_json::Value, user_id: String) -> Self {
        Self {
            event_type,
            event_data,
            timestamp: Utc::now(),
            user_id,
            machine_id: MachineId::current().id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_calculation() {
        let progress = ProgressInfo::new(50, 100, 10);
        assert_eq!(progress.percentage, 50.0);
        assert!(!progress.is_complete());

        let complete = ProgressInfo::new(100, 100, 20);
        assert_eq!(complete.percentage, 100.0);
        assert!(complete.is_complete());
    }

    #[test]
    fn test_machine_id_generation() {
        let machine_id = MachineId::current();
        assert!(!machine_id.id.is_empty());
        assert!(!machine_id.hostname.is_empty());
        assert!(!machine_id.platform.is_empty());
    }
}
