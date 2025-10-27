-- Migration Rollback: 008_ideate_templates
-- Description: Remove default PRD quickstart templates
-- Created: 2025-01-27

-- Delete default system templates
DELETE FROM prd_quickstart_templates WHERE is_system = 1;
