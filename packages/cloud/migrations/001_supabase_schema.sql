-- Orkee Cloud Supabase Schema
-- This schema is for the PostgreSQL database in Supabase

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users extended by Supabase Auth (automatic)
-- auth.users table already exists

-- Subscription tiers and user limits
CREATE TYPE subscription_tier AS ENUM ('free', 'starter', 'pro', 'enterprise');

CREATE TABLE subscriptions (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE UNIQUE,
    tier subscription_tier DEFAULT 'free',
    project_limit INTEGER DEFAULT 2,        -- Free tier gets 2 projects
    storage_limit_mb INTEGER DEFAULT 100,   -- Free tier gets 100MB
    
    -- Feature flags for different tiers
    allow_auto_sync BOOLEAN DEFAULT FALSE,
    allow_realtime BOOLEAN DEFAULT FALSE,
    allow_collaboration BOOLEAN DEFAULT FALSE,
    
    -- Stripe integration
    stripe_customer_id TEXT UNIQUE,
    stripe_subscription_id TEXT UNIQUE,
    stripe_price_id TEXT,
    
    -- Subscription lifecycle
    trial_ends_at TIMESTAMPTZ,
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN DEFAULT FALSE,
    canceled_at TIMESTAMPTZ,
    
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Projects table (mirrors local SQLite schema)
CREATE TABLE projects (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE NOT NULL,
    
    -- Core fields (match SQLite)
    name TEXT NOT NULL,
    project_root TEXT,
    description TEXT,
    status TEXT DEFAULT 'active' CHECK (status IN ('active', 'archived', 'deleted')),
    priority TEXT DEFAULT 'medium' CHECK (priority IN ('high', 'medium', 'low')),
    
    -- Scripts
    setup_script TEXT,
    dev_script TEXT,
    cleanup_script TEXT,
    
    -- Sync metadata
    local_machine_id TEXT,
    local_version INTEGER DEFAULT 1,
    cloud_version INTEGER DEFAULT 1,
    sync_status TEXT DEFAULT 'synced' CHECK (sync_status IN ('synced', 'pending', 'conflict')),
    last_synced_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ, -- Soft delete for sync
    
    -- Constraints
    UNIQUE(user_id, name)
);

-- Create indexes
CREATE INDEX idx_user_projects ON projects (user_id, deleted_at);
CREATE INDEX idx_sync_status ON projects (user_id, sync_status);

-- Project snapshots metadata
CREATE TABLE project_snapshots (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    
    -- Snapshot details
    version INTEGER NOT NULL,
    storage_path TEXT NOT NULL, -- Path in Supabase Storage
    size_bytes BIGINT,
    checksum TEXT, -- SHA-256
    encrypted BOOLEAN DEFAULT TRUE,
    
    -- Metadata
    machine_id TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(project_id, version)
);

-- Create index for snapshot queries
CREATE INDEX idx_project_snapshots ON project_snapshots (project_id, version DESC);

-- Sync conflicts tracking
CREATE TABLE sync_conflicts (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    
    -- Conflict details
    local_version INTEGER,
    cloud_version INTEGER,
    local_data JSONB,
    cloud_data JSONB,
    
    -- Resolution
    resolution_strategy TEXT CHECK (resolution_strategy IN ('local', 'cloud', 'merge', 'manual')),
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES auth.users(id),
    resolved_data JSONB,
    
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit log for compliance
CREATE TABLE audit_log (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID REFERENCES auth.users(id) ON DELETE SET NULL,
    action TEXT NOT NULL,
    resource_type TEXT,
    resource_id TEXT,
    metadata JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for audit log
CREATE INDEX idx_audit_user ON audit_log (user_id, created_at DESC);
CREATE INDEX idx_audit_resource ON audit_log (resource_type, resource_id);

-- Usage tracking for billing
CREATE TABLE usage_metrics (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    metric_type TEXT NOT NULL CHECK (metric_type IN ('storage', 'bandwidth', 'api_calls', 'projects')),
    value NUMERIC NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Prevent duplicates
    UNIQUE(user_id, metric_type, period_start)
);

-- Free tier user engagement tracking
CREATE TABLE user_events (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL CHECK (event_type IN (
        'signup', 'first_project', 'first_sync', 'project_limit_hit', 
        'storage_limit_hit', 'upgrade_prompt_shown', 'upgrade_clicked'
    )),
    event_data JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for analytics
CREATE INDEX idx_user_events_type ON user_events (event_type, created_at);
CREATE INDEX idx_user_events_user ON user_events (user_id, created_at);

-- Tier limits configuration
CREATE TABLE tier_limits (
    tier subscription_tier PRIMARY KEY,
    project_limit INTEGER NOT NULL,
    storage_limit_mb INTEGER NOT NULL,
    allow_auto_sync BOOLEAN DEFAULT FALSE,
    allow_realtime BOOLEAN DEFAULT FALSE,
    allow_collaboration BOOLEAN DEFAULT FALSE,
    sync_frequency_minutes INTEGER DEFAULT 60, -- Minimum time between syncs
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default tier limits
INSERT INTO tier_limits (tier, project_limit, storage_limit_mb, allow_auto_sync, allow_realtime, allow_collaboration, sync_frequency_minutes) VALUES
    ('free', 2, 100, FALSE, FALSE, FALSE, 1),           -- Free: 2 projects, 100MB, manual sync only (1 min between)
    ('starter', 10, 5120, TRUE, FALSE, FALSE, 15),      -- Starter: 10 projects, 5GB, auto-sync every 15 min
    ('pro', -1, 51200, TRUE, TRUE, TRUE, 1),            -- Pro: unlimited projects, 50GB, real-time
    ('enterprise', -1, -1, TRUE, TRUE, TRUE, 1);        -- Enterprise: unlimited everything

-- RLS Policies
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE project_snapshots ENABLE ROW LEVEL SECURITY;
ALTER TABLE sync_conflicts ENABLE ROW LEVEL SECURITY;
ALTER TABLE subscriptions ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE usage_metrics ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_events ENABLE ROW LEVEL SECURITY;

-- Projects RLS
CREATE POLICY "Users manage own projects"
    ON projects FOR ALL
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

-- Snapshots RLS
CREATE POLICY "Users manage own snapshots"
    ON project_snapshots FOR ALL
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

-- Conflicts RLS
CREATE POLICY "Users manage own conflicts"
    ON sync_conflicts FOR ALL
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

-- Subscriptions RLS
CREATE POLICY "Users can view own subscription"
    ON subscriptions FOR SELECT
    USING (auth.uid() = user_id);

-- Audit log RLS (read-only for users)
CREATE POLICY "Users can view own audit log"
    ON audit_log FOR SELECT
    USING (auth.uid() = user_id);

-- Usage metrics RLS
CREATE POLICY "Users can view own usage"
    ON usage_metrics FOR SELECT
    USING (auth.uid() = user_id);

-- User events RLS
CREATE POLICY "Users can view own events"
    ON user_events FOR SELECT
    USING (auth.uid() = user_id);

-- Functions
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers
CREATE TRIGGER update_projects_updated_at
    BEFORE UPDATE ON projects
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_subscriptions_updated_at
    BEFORE UPDATE ON subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();