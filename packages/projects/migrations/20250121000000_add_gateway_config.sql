-- Add Vercel AI Gateway configuration to users table
-- Enables database-backed storage for gateway credentials in Tauri desktop apps

ALTER TABLE users ADD COLUMN ai_gateway_enabled INTEGER DEFAULT 0;
ALTER TABLE users ADD COLUMN ai_gateway_url TEXT;
ALTER TABLE users ADD COLUMN ai_gateway_key TEXT;
