-- Cloud sync state tracking schema
-- This migration adds tables to support cloud synchronization

-- Cloud sync configuration and state
CREATE TABLE cloud_sync_state (
    id INTEGER PRIMARY KEY,
    provider_name TEXT NOT NULL,
    provider_type TEXT NOT NULL,
    enabled BOOLEAN DEFAULT 1,
    
    -- Last sync information
    last_sync_at TEXT,
    last_successful_sync_at TEXT,
    last_snapshot_id TEXT,
    
    -- Current state
    sync_in_progress BOOLEAN DEFAULT 0,
    current_operation TEXT CHECK(current_operation IN ('backup', 'restore', 'full_sync', 'incremental_sync')),
    
    -- Error tracking
    error_count INTEGER DEFAULT 0,
    last_error_message TEXT,
    last_error_at TEXT,
    
    -- Configuration
    auto_sync_enabled BOOLEAN DEFAULT 0,
    sync_interval_minutes INTEGER DEFAULT 1440, -- 24 hours
    max_snapshots INTEGER DEFAULT 30,
    encryption_enabled BOOLEAN DEFAULT 1,
    compression_enabled BOOLEAN DEFAULT 1,
    
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    UNIQUE(provider_name)
);

-- Cloud snapshots metadata (local cache)
CREATE TABLE cloud_snapshots (
    id TEXT PRIMARY KEY,
    provider_name TEXT NOT NULL,
    snapshot_id TEXT NOT NULL,
    
    -- Metadata
    created_at TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    compressed_size_bytes INTEGER NOT NULL,
    project_count INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    checksum TEXT,
    encrypted BOOLEAN DEFAULT 0,
    
    -- Storage information
    storage_path TEXT,
    etag TEXT,
    last_accessed_at TEXT,
    
    -- Sync tracking
    uploaded_at TEXT,
    download_count INTEGER DEFAULT 0,
    last_downloaded_at TEXT,
    
    -- Local state
    locally_deleted BOOLEAN DEFAULT 0,
    deletion_scheduled_at TEXT,
    
    -- JSON metadata for extensibility
    metadata_json TEXT,
    tags_json TEXT,
    
    FOREIGN KEY (provider_name) REFERENCES cloud_sync_state(provider_name) ON DELETE CASCADE,
    UNIQUE(provider_name, snapshot_id)
);

-- Sync conflicts tracking
CREATE TABLE sync_conflicts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_name TEXT NOT NULL,
    snapshot_id TEXT,
    project_id TEXT NOT NULL,
    
    -- Conflict details
    detected_at TEXT NOT NULL DEFAULT (datetime('now')),
    conflict_type TEXT NOT NULL CHECK(conflict_type IN ('duplicate_name', 'duplicate_path', 'version_conflict', 'data_conflict')),
    
    -- Conflict data
    local_value TEXT,
    remote_value TEXT,
    local_version INTEGER,
    remote_version INTEGER,
    
    -- Resolution
    resolution_status TEXT DEFAULT 'pending' CHECK(resolution_status IN ('pending', 'resolved', 'ignored', 'merged')),
    resolution_strategy TEXT CHECK(resolution_strategy IN ('use_local', 'use_remote', 'merge', 'manual')),
    resolved_at TEXT,
    resolved_by TEXT,
    resolution_notes TEXT,
    
    FOREIGN KEY (provider_name) REFERENCES cloud_sync_state(provider_name) ON DELETE CASCADE
    -- Note: project_id is a foreign key to projects.id but we can't create the constraint 
    -- since projects table is in a different database/package to avoid circular dependencies
);

-- Sync operations log for audit trail
CREATE TABLE sync_operations_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_name TEXT NOT NULL,
    operation_type TEXT NOT NULL CHECK(operation_type IN ('backup', 'restore', 'upload', 'download', 'delete', 'list')),
    operation_id TEXT, -- UUID for tracking related operations
    
    -- Operation details
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    status TEXT NOT NULL DEFAULT 'started' CHECK(status IN ('started', 'completed', 'failed', 'cancelled')),
    
    -- Data tracking
    snapshot_id TEXT,
    projects_affected INTEGER DEFAULT 0,
    bytes_transferred INTEGER DEFAULT 0,
    
    -- Results
    error_message TEXT,
    warning_messages TEXT, -- JSON array of warnings
    
    -- Performance metrics
    duration_seconds INTEGER,
    network_time_seconds INTEGER,
    processing_time_seconds INTEGER,
    
    -- Context information
    initiated_by TEXT DEFAULT 'system', -- 'user', 'system', 'scheduler'
    user_agent TEXT,
    client_version TEXT,
    
    -- Additional metadata
    metadata_json TEXT,
    
    FOREIGN KEY (provider_name) REFERENCES cloud_sync_state(provider_name) ON DELETE CASCADE
);

-- Sync statistics for monitoring and optimization
CREATE TABLE sync_statistics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_name TEXT NOT NULL,
    statistic_date TEXT NOT NULL, -- Date in YYYY-MM-DD format
    
    -- Daily statistics
    backup_count INTEGER DEFAULT 0,
    restore_count INTEGER DEFAULT 0,
    successful_syncs INTEGER DEFAULT 0,
    failed_syncs INTEGER DEFAULT 0,
    
    -- Data statistics
    total_bytes_uploaded INTEGER DEFAULT 0,
    total_bytes_downloaded INTEGER DEFAULT 0,
    average_snapshot_size INTEGER DEFAULT 0,
    
    -- Performance statistics
    average_upload_speed_bps INTEGER DEFAULT 0,
    average_download_speed_bps INTEGER DEFAULT 0,
    average_operation_duration_seconds INTEGER DEFAULT 0,
    
    -- Error statistics
    error_count INTEGER DEFAULT 0,
    most_common_error TEXT,
    
    -- Created/updated tracking
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    FOREIGN KEY (provider_name) REFERENCES cloud_sync_state(provider_name) ON DELETE CASCADE,
    UNIQUE(provider_name, statistic_date)
);

-- Indexes for performance
CREATE INDEX idx_cloud_snapshots_provider_created ON cloud_snapshots(provider_name, created_at DESC);
CREATE INDEX idx_cloud_snapshots_size ON cloud_snapshots(size_bytes);
CREATE INDEX idx_cloud_snapshots_project_count ON cloud_snapshots(project_count);
CREATE INDEX idx_sync_conflicts_status ON sync_conflicts(resolution_status, detected_at);
CREATE INDEX idx_sync_conflicts_project ON sync_conflicts(project_id, conflict_type);
CREATE INDEX idx_sync_operations_log_provider_date ON sync_operations_log(provider_name, started_at DESC);
CREATE INDEX idx_sync_operations_log_status ON sync_operations_log(status, operation_type);
CREATE INDEX idx_sync_statistics_provider_date ON sync_statistics(provider_name, statistic_date DESC);

-- Triggers to maintain updated_at timestamps
CREATE TRIGGER update_cloud_sync_state_updated_at 
    AFTER UPDATE ON cloud_sync_state
BEGIN
    UPDATE cloud_sync_state 
    SET updated_at = datetime('now') 
    WHERE id = NEW.id;
END;

CREATE TRIGGER update_sync_statistics_updated_at 
    AFTER UPDATE ON sync_statistics
BEGIN
    UPDATE sync_statistics 
    SET updated_at = datetime('now') 
    WHERE id = NEW.id;
END;

-- Views for common queries
CREATE VIEW active_providers AS
SELECT 
    provider_name,
    provider_type,
    enabled,
    last_successful_sync_at,
    auto_sync_enabled,
    sync_interval_minutes,
    error_count,
    last_error_message
FROM cloud_sync_state 
WHERE enabled = 1;

CREATE VIEW recent_snapshots AS
SELECT 
    cs.snapshot_id,
    cs.provider_name,
    cs.created_at,
    cs.size_bytes,
    cs.project_count,
    cs.encrypted,
    cst.provider_type
FROM cloud_snapshots cs
JOIN cloud_sync_state cst ON cs.provider_name = cst.provider_name
WHERE cs.locally_deleted = 0
ORDER BY cs.created_at DESC;

CREATE VIEW sync_health_summary AS
SELECT 
    css.provider_name,
    css.provider_type,
    css.enabled,
    css.auto_sync_enabled,
    css.last_successful_sync_at,
    css.error_count,
    COUNT(cs.id) as snapshot_count,
    COUNT(CASE WHEN sc.resolution_status = 'pending' THEN 1 END) as pending_conflicts,
    MAX(cs.created_at) as latest_snapshot_at
FROM cloud_sync_state css
LEFT JOIN cloud_snapshots cs ON css.provider_name = cs.provider_name AND cs.locally_deleted = 0
LEFT JOIN sync_conflicts sc ON css.provider_name = sc.provider_name AND sc.resolution_status = 'pending'
GROUP BY css.provider_name;

-- Initial data for development/testing
-- This will be populated by the application when providers are configured