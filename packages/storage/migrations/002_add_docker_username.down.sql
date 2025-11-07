-- Remove docker_username field from sandbox_settings

ALTER TABLE sandbox_settings DROP COLUMN docker_username;
