// ABOUTME: OpenSpec sync engine for bidirectional PRD ↔ Spec ↔ Task synchronization
// ABOUTME: Handles change detection, delta generation, and conflict resolution

use super::types::{DeltaType, ParsedCapability, ParsedSpec, SpecCapability};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),

    #[error("Requirement not found: {0}")]
    RequirementNotFound(String),

    #[error("Conflict detected in capability '{0}': {1}")]
    ConflictDetected(String, String),

    #[error("Invalid sync state: {0}")]
    InvalidState(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

pub type SyncResult<T> = Result<T, SyncError>;

/// Represents a detected difference between two spec versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDiff {
    pub capability_name: String,
    pub delta_type: DeltaType,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub changed_requirements: Vec<String>,
}

/// Result of a sync operation
#[derive(Debug, Clone, Default)]
pub struct SyncReport {
    pub added_capabilities: Vec<String>,
    pub modified_capabilities: Vec<String>,
    pub removed_capabilities: Vec<String>,
    pub added_requirements: usize,
    pub modified_requirements: usize,
    pub removed_requirements: usize,
    pub conflicts: Vec<String>,
}

/// Detect changes between an old spec and a new parsed spec
pub fn detect_changes(
    old_capabilities: &[SpecCapability],
    new_spec: &ParsedSpec,
) -> SyncResult<Vec<CapabilityDiff>> {
    let mut changes = Vec::new();

    // Build maps for quick lookup
    let old_map: HashMap<String, &SpecCapability> = old_capabilities
        .iter()
        .map(|c| (c.name.clone(), c))
        .collect();

    let new_map: HashMap<String, &ParsedCapability> = new_spec
        .capabilities
        .iter()
        .map(|c| (c.name.clone(), c))
        .collect();

    // Find added capabilities
    for (name, new_cap) in &new_map {
        if !old_map.contains_key(name) {
            changes.push(CapabilityDiff {
                capability_name: name.clone(),
                delta_type: DeltaType::Added,
                old_content: None,
                new_content: Some(format_capability_content(new_cap)),
                changed_requirements: new_cap
                    .requirements
                    .iter()
                    .map(|r| r.name.clone())
                    .collect(),
            });
        }
    }

    // Find modified capabilities
    for (name, old_cap) in &old_map {
        if let Some(new_cap) = new_map.get(name) {
            if is_capability_modified(old_cap, new_cap) {
                let changed_reqs = detect_requirement_changes(old_cap, new_cap);
                changes.push(CapabilityDiff {
                    capability_name: name.clone(),
                    delta_type: DeltaType::Modified,
                    old_content: Some(old_cap.spec_markdown.clone()),
                    new_content: Some(format_capability_content(new_cap)),
                    changed_requirements: changed_reqs,
                });
            }
        }
    }

    // Find removed capabilities
    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            changes.push(CapabilityDiff {
                capability_name: name.clone(),
                delta_type: DeltaType::Removed,
                old_content: Some(old_map[name].spec_markdown.clone()),
                new_content: None,
                changed_requirements: vec![],
            });
        }
    }

    Ok(changes)
}

/// Check if a capability has been modified
fn is_capability_modified(old: &SpecCapability, new: &ParsedCapability) -> bool {
    // Check if purpose changed
    let old_purpose = old.purpose_markdown.as_deref().unwrap_or("");
    if old_purpose.trim() != new.purpose.trim() {
        return true;
    }

    // Check if requirements changed
    if old.requirement_count != new.requirements.len() as i32 {
        return true;
    }

    // No changes detected
    false
}

/// Detect which requirements changed within a capability
fn detect_requirement_changes(_old: &SpecCapability, new: &ParsedCapability) -> Vec<String> {
    // This is a simplified version - would need database access for full implementation
    new.requirements.iter().map(|r| r.name.clone()).collect()
}

/// Format a parsed capability into markdown content
fn format_capability_content(capability: &ParsedCapability) -> String {
    let mut content = String::new();

    // Add purpose
    if !capability.purpose.is_empty() {
        content.push_str(&capability.purpose);
        content.push_str("\n\n");
    }

    // Add requirements
    for requirement in &capability.requirements {
        content.push_str("### ");
        content.push_str(&requirement.name);
        content.push_str("\n\n");

        if !requirement.description.is_empty() {
            content.push_str(&requirement.description);
            content.push_str("\n\n");
        }

        // Add scenarios
        for scenario in &requirement.scenarios {
            if !scenario.name.is_empty() {
                content.push_str("**Scenario: ");
                content.push_str(&scenario.name);
                content.push_str("**\n");
            }

            content.push_str("WHEN ");
            content.push_str(&scenario.when);
            content.push('\n');

            content.push_str("THEN ");
            content.push_str(&scenario.then);
            content.push('\n');

            for and_clause in &scenario.and {
                content.push_str("AND ");
                content.push_str(and_clause);
                content.push('\n');
            }

            content.push('\n');
        }
    }

    content
}

/// Generate a sync report from detected changes
pub fn generate_sync_report(changes: &[CapabilityDiff]) -> SyncReport {
    let mut report = SyncReport::default();

    for change in changes {
        match change.delta_type {
            DeltaType::Added => {
                report
                    .added_capabilities
                    .push(change.capability_name.clone());
                report.added_requirements += change.changed_requirements.len();
            }
            DeltaType::Modified => {
                report
                    .modified_capabilities
                    .push(change.capability_name.clone());
                report.modified_requirements += change.changed_requirements.len();
            }
            DeltaType::Removed => {
                report
                    .removed_capabilities
                    .push(change.capability_name.clone());
                report.removed_requirements += change.changed_requirements.len();
            }
        }
    }

    report
}

/// Conflict detection for concurrent modifications
#[derive(Debug, Clone)]
pub struct ConflictDetector {
    _last_sync_time: Option<DateTime<Utc>>,
}

impl ConflictDetector {
    pub fn new() -> Self {
        Self {
            _last_sync_time: None,
        }
    }

    pub fn with_last_sync_time(last_sync_time: DateTime<Utc>) -> Self {
        Self {
            _last_sync_time: Some(last_sync_time),
        }
    }

    /// Detect conflicts between local and remote changes
    pub fn detect_conflicts(
        &self,
        local_changes: &[CapabilityDiff],
        remote_changes: &[CapabilityDiff],
    ) -> Vec<String> {
        let mut conflicts = Vec::new();

        // Build maps of changed capabilities
        let local_map: HashSet<&str> = local_changes
            .iter()
            .map(|c| c.capability_name.as_str())
            .collect();

        let remote_map: HashSet<&str> = remote_changes
            .iter()
            .map(|c| c.capability_name.as_str())
            .collect();

        // Find overlapping changes
        for capability_name in local_map.iter() {
            if remote_map.contains(capability_name) {
                conflicts.push(format!(
                    "Capability '{}' modified in both local and remote",
                    capability_name
                ));
            }
        }

        conflicts
    }

    /// Resolve conflicts using a merge strategy
    pub fn resolve_conflicts(
        &self,
        local_changes: &[CapabilityDiff],
        remote_changes: &[CapabilityDiff],
        strategy: MergeStrategy,
    ) -> SyncResult<Vec<CapabilityDiff>> {
        match strategy {
            MergeStrategy::PreferLocal => Ok(local_changes.to_vec()),
            MergeStrategy::PreferRemote => Ok(remote_changes.to_vec()),
            MergeStrategy::Manual => Err(SyncError::ConflictDetected(
                "Manual resolution required".to_string(),
                "Cannot auto-resolve conflicts with Manual strategy".to_string(),
            )),
            MergeStrategy::ThreeWayMerge => {
                // Would need base version for true 3-way merge
                // For now, fall back to prefer local
                Ok(local_changes.to_vec())
            }
        }
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Merge strategy for conflict resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Keep local changes, discard remote
    PreferLocal,
    /// Keep remote changes, discard local
    PreferRemote,
    /// Require manual resolution
    Manual,
    /// Attempt three-way merge (requires base version)
    ThreeWayMerge,
}

/// Sync engine for orchestrating sync operations
pub struct SyncEngine {
    conflict_detector: ConflictDetector,
}

impl SyncEngine {
    pub fn new() -> Self {
        Self {
            conflict_detector: ConflictDetector::new(),
        }
    }

    pub fn with_last_sync_time(last_sync_time: DateTime<Utc>) -> Self {
        Self {
            conflict_detector: ConflictDetector::with_last_sync_time(last_sync_time),
        }
    }

    /// Perform a full sync operation
    pub fn sync(
        &self,
        old_capabilities: &[SpecCapability],
        new_spec: &ParsedSpec,
    ) -> SyncResult<SyncReport> {
        // Detect changes
        let changes = detect_changes(old_capabilities, new_spec)?;

        // Generate report
        let report = generate_sync_report(&changes);

        Ok(report)
    }

    /// Sync with conflict detection
    pub fn sync_with_conflict_detection(
        &self,
        local_capabilities: &[SpecCapability],
        local_spec: &ParsedSpec,
        remote_capabilities: &[SpecCapability],
        remote_spec: &ParsedSpec,
        strategy: MergeStrategy,
    ) -> SyncResult<SyncReport> {
        // Detect local and remote changes
        let local_changes = detect_changes(local_capabilities, local_spec)?;
        let remote_changes = detect_changes(remote_capabilities, remote_spec)?;

        // Check for conflicts
        let conflicts = self
            .conflict_detector
            .detect_conflicts(&local_changes, &remote_changes);

        if !conflicts.is_empty() && strategy == MergeStrategy::Manual {
            return Err(SyncError::ConflictDetected(
                "Multiple capabilities".to_string(),
                conflicts.join(", "),
            ));
        }

        // Resolve conflicts
        let resolved_changes =
            self.conflict_detector
                .resolve_conflicts(&local_changes, &remote_changes, strategy)?;

        // Generate report
        let mut report = generate_sync_report(&resolved_changes);
        report.conflicts = conflicts;

        Ok(report)
    }
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openspec::types::{CapabilityStatus, ParsedRequirement, ParsedScenario};

    fn create_test_capability(name: &str, requirement_count: i32) -> SpecCapability {
        SpecCapability {
            id: "test-id".to_string(),
            project_id: "test-project".to_string(),
            prd_id: None,
            name: name.to_string(),
            purpose_markdown: Some("Test purpose".to_string()),
            spec_markdown: "Test spec".to_string(),
            design_markdown: None,
            requirement_count,
            version: 1,
            status: CapabilityStatus::Active,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_parsed_capability(name: &str, req_count: usize) -> ParsedCapability {
        ParsedCapability {
            name: name.to_string(),
            purpose: "Test purpose".to_string(),
            requirements: (0..req_count)
                .map(|i| ParsedRequirement {
                    name: format!("Req{}", i),
                    description: "Test description".to_string(),
                    scenarios: vec![ParsedScenario {
                        name: "Test scenario".to_string(),
                        when: "test condition".to_string(),
                        then: "test result".to_string(),
                        and: vec![],
                    }],
                })
                .collect(),
        }
    }

    #[test]
    fn test_detect_added_capability() {
        let old_capabilities = vec![];
        let new_spec = ParsedSpec {
            capabilities: vec![create_test_parsed_capability("NewCap", 1)],
            raw_markdown: String::new(),
        };

        let changes = detect_changes(&old_capabilities, &new_spec).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].capability_name, "NewCap");
        assert!(matches!(changes[0].delta_type, DeltaType::Added));
    }

    #[test]
    fn test_detect_removed_capability() {
        let old_capabilities = vec![create_test_capability("OldCap", 1)];
        let new_spec = ParsedSpec {
            capabilities: vec![],
            raw_markdown: String::new(),
        };

        let changes = detect_changes(&old_capabilities, &new_spec).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].capability_name, "OldCap");
        assert!(matches!(changes[0].delta_type, DeltaType::Removed));
    }

    #[test]
    fn test_sync_report_generation() {
        let changes = vec![
            CapabilityDiff {
                capability_name: "Cap1".to_string(),
                delta_type: DeltaType::Added,
                old_content: None,
                new_content: Some("content".to_string()),
                changed_requirements: vec!["Req1".to_string(), "Req2".to_string()],
            },
            CapabilityDiff {
                capability_name: "Cap2".to_string(),
                delta_type: DeltaType::Modified,
                old_content: Some("old".to_string()),
                new_content: Some("new".to_string()),
                changed_requirements: vec!["Req3".to_string()],
            },
        ];

        let report = generate_sync_report(&changes);
        assert_eq!(report.added_capabilities.len(), 1);
        assert_eq!(report.modified_capabilities.len(), 1);
        assert_eq!(report.added_requirements, 2);
        assert_eq!(report.modified_requirements, 1);
    }

    #[test]
    fn test_conflict_detection() {
        let detector = ConflictDetector::new();

        let local_changes = vec![CapabilityDiff {
            capability_name: "SharedCap".to_string(),
            delta_type: DeltaType::Modified,
            old_content: Some("old".to_string()),
            new_content: Some("local new".to_string()),
            changed_requirements: vec![],
        }];

        let remote_changes = vec![CapabilityDiff {
            capability_name: "SharedCap".to_string(),
            delta_type: DeltaType::Modified,
            old_content: Some("old".to_string()),
            new_content: Some("remote new".to_string()),
            changed_requirements: vec![],
        }];

        let conflicts = detector.detect_conflicts(&local_changes, &remote_changes);
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("SharedCap"));
    }

    #[test]
    fn test_sync_engine() {
        let engine = SyncEngine::new();

        let old_capabilities = vec![create_test_capability("OldCap", 2)];
        let new_spec = ParsedSpec {
            capabilities: vec![
                create_test_parsed_capability("OldCap", 3), // Modified
                create_test_parsed_capability("NewCap", 1), // Added
            ],
            raw_markdown: String::new(),
        };

        let report = engine.sync(&old_capabilities, &new_spec).unwrap();
        assert_eq!(report.added_capabilities.len(), 1);
        assert_eq!(report.modified_capabilities.len(), 1);
    }
}
