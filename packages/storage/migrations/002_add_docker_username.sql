-- Add docker_username field to sandbox_settings
-- This stores the Docker Hub username for building and pushing images

ALTER TABLE sandbox_settings ADD COLUMN docker_username TEXT;
