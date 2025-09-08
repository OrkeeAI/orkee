use super::{CloudCredentials, CloudError, CloudResult, CloudProvider};
use chrono::{DateTime, Utc};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service name for keyring storage
const KEYRING_SERVICE: &str = "orkee-cloud-sync";

/// Credential store for secure storage of cloud provider credentials
pub struct CredentialStore;

impl CredentialStore {
    /// Store credentials securely in the OS keyring
    pub fn store_credentials(provider: &str, credentials: &CloudCredentials) -> CloudResult<()> {
        let entry = Entry::new(KEYRING_SERVICE, provider)
            .map_err(|e| CloudError::Configuration(format!("Failed to access keyring: {}", e)))?;

        let serialized = serde_json::to_string(credentials)
            .map_err(|e| CloudError::Serialization(e))?;

        entry
            .set_password(&serialized)
            .map_err(|e| CloudError::Configuration(format!("Failed to store credentials: {}", e)))?;

        tracing::info!("Credentials stored successfully for provider: {}", provider);
        Ok(())
    }

    /// Retrieve credentials from the OS keyring
    pub fn get_credentials(provider: &str) -> CloudResult<CloudCredentials> {
        let entry = Entry::new(KEYRING_SERVICE, provider)
            .map_err(|e| CloudError::Configuration(format!("Failed to access keyring: {}", e)))?;

        let serialized = entry
            .get_password()
            .map_err(|e| CloudError::Authentication(format!("Failed to retrieve credentials: {}", e)))?;

        let credentials = serde_json::from_str(&serialized)
            .map_err(|e| CloudError::Serialization(e))?;

        Ok(credentials)
    }

    /// Remove credentials from the OS keyring
    pub fn remove_credentials(provider: &str) -> CloudResult<()> {
        let entry = Entry::new(KEYRING_SERVICE, provider)
            .map_err(|e| CloudError::Configuration(format!("Failed to access keyring: {}", e)))?;

        entry
            .delete_credential()
            .map_err(|e| CloudError::Configuration(format!("Failed to remove credentials: {}", e)))?;

        tracing::info!("Credentials removed successfully for provider: {}", provider);
        Ok(())
    }

    /// Check if credentials exist for a provider
    pub fn has_credentials(provider: &str) -> bool {
        let entry = match Entry::new(KEYRING_SERVICE, provider) {
            Ok(entry) => entry,
            Err(_) => return false,
        };

        entry.get_password().is_ok()
    }

    /// List all providers with stored credentials
    pub fn list_providers() -> CloudResult<Vec<String>> {
        // Unfortunately, keyring doesn't support listing entries
        // So we'll check for known providers
        let known_providers = vec!["s3", "r2", "azure", "gcs"];
        let mut found_providers = Vec::new();

        for provider in known_providers {
            if Self::has_credentials(provider) {
                found_providers.push(provider.to_string());
            }
        }

        Ok(found_providers)
    }
}

/// Fallback credential provider that reads from environment variables
pub struct EnvironmentCredentialProvider;

impl EnvironmentCredentialProvider {
    /// Get AWS credentials from environment variables
    pub fn get_aws_credentials(region: Option<String>) -> CloudResult<CloudCredentials> {
        let access_key_id = std::env::var("AWS_ACCESS_KEY_ID")
            .map_err(|_| CloudError::Authentication("AWS_ACCESS_KEY_ID not found".to_string()))?;

        let secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY")
            .map_err(|_| CloudError::Authentication("AWS_SECRET_ACCESS_KEY not found".to_string()))?;

        let session_token = std::env::var("AWS_SESSION_TOKEN").ok();

        let region = region
            .or_else(|| std::env::var("AWS_REGION").ok())
            .or_else(|| std::env::var("AWS_DEFAULT_REGION").ok())
            .unwrap_or_else(|| "us-east-1".to_string());

        Ok(CloudCredentials::AwsCredentials {
            access_key_id,
            secret_access_key,
            session_token,
            region,
        })
    }

    /// Get API key credentials from environment
    pub fn get_api_key_credentials(key_env: &str, secret_env: Option<&str>) -> CloudResult<CloudCredentials> {
        let key = std::env::var(key_env)
            .map_err(|_| CloudError::Authentication(format!("{} not found", key_env)))?;

        let secret = if let Some(secret_env) = secret_env {
            std::env::var(secret_env).ok()
        } else {
            None
        };

        Ok(CloudCredentials::ApiKey { key, secret })
    }

    /// Get OAuth2 credentials from environment
    pub fn get_oauth2_credentials(
        access_token_env: &str,
        refresh_token_env: Option<&str>,
    ) -> CloudResult<CloudCredentials> {
        let access_token = std::env::var(access_token_env)
            .map_err(|_| CloudError::Authentication(format!("{} not found", access_token_env)))?;

        let refresh_token = if let Some(refresh_env) = refresh_token_env {
            std::env::var(refresh_env).ok()
        } else {
            None
        };

        Ok(CloudCredentials::OAuth2 {
            access_token,
            refresh_token,
            expires_at: None,
        })
    }
}

/// Composite credential provider that tries multiple sources
pub struct CredentialProvider {
    provider_name: String,
}

impl CredentialProvider {
    pub fn new(provider_name: String) -> Self {
        Self { provider_name }
    }

    /// Get credentials using the following precedence:
    /// 1. OS Keyring (most secure)
    /// 2. Environment variables (fallback)
    pub fn get_credentials(&self) -> CloudResult<CloudCredentials> {
        // Try keyring first
        if CredentialStore::has_credentials(&self.provider_name) {
            tracing::debug!("Loading credentials from keyring for provider: {}", self.provider_name);
            return CredentialStore::get_credentials(&self.provider_name);
        }

        // Fallback to environment variables
        tracing::debug!("Loading credentials from environment for provider: {}", self.provider_name);
        match self.provider_name.as_str() {
            "s3" | "aws" => {
                EnvironmentCredentialProvider::get_aws_credentials(None)
            }
            "r2" => {
                // Cloudflare R2 uses S3-compatible API
                let mut creds = EnvironmentCredentialProvider::get_aws_credentials(None)
                    .or_else(|_| -> CloudResult<CloudCredentials> {
                        // Try R2-specific env vars
                        let access_key_id = std::env::var("R2_ACCESS_KEY_ID")
                            .map_err(|_| CloudError::Authentication("R2_ACCESS_KEY_ID not found".to_string()))?;
                        let secret_access_key = std::env::var("R2_SECRET_ACCESS_KEY")
                            .map_err(|_| CloudError::Authentication("R2_SECRET_ACCESS_KEY not found".to_string()))?;
                        
                        Ok(CloudCredentials::AwsCredentials {
                            access_key_id,
                            secret_access_key,
                            session_token: None,
                            region: "auto".to_string(),
                        })
                    })?;

                // Override region for R2
                if let CloudCredentials::AwsCredentials { region, .. } = &mut creds {
                    *region = "auto".to_string();
                }

                Ok(creds)
            }
            _ => Err(CloudError::Configuration(
                format!("Unknown provider: {}", self.provider_name)
            )),
        }
    }

    /// Store credentials securely
    pub fn store_credentials(&self, credentials: &CloudCredentials) -> CloudResult<()> {
        CredentialStore::store_credentials(&self.provider_name, credentials)
    }

    /// Remove stored credentials
    pub fn remove_credentials(&self) -> CloudResult<()> {
        CredentialStore::remove_credentials(&self.provider_name)
    }

    /// Test if credentials are valid
    pub async fn validate_credentials(&self) -> CloudResult<bool> {
        let credentials = self.get_credentials()?;
        
        match self.provider_name.as_str() {
            "s3" | "aws" => {
                // Create a minimal S3 provider to test credentials
                let provider = super::s3::S3Provider::new(
                    "test-bucket".to_string(),
                    "us-east-1".to_string(),
                );
                
                match provider.authenticate(&credentials).await {
                    Ok(_) => Ok(true),
                    Err(CloudError::Authentication(_)) => Ok(false),
                    Err(e) => Err(e),
                }
            }
            _ => Err(CloudError::Configuration(
                format!("Validation not implemented for provider: {}", self.provider_name)
            )),
        }
    }
}

/// Credential rotation manager
pub struct CredentialRotationManager {
    provider: String,
    rotation_interval_days: u32,
}

impl CredentialRotationManager {
    pub fn new(provider: String, rotation_interval_days: u32) -> Self {
        Self {
            provider,
            rotation_interval_days,
        }
    }

    /// Check if credentials need rotation
    pub fn needs_rotation(&self) -> CloudResult<bool> {
        // This is a placeholder - in a real implementation, you'd track
        // when credentials were last rotated
        let metadata = self.get_credential_metadata()?;
        
        if let Some(last_rotation) = metadata.last_rotated {
            let days_since_rotation = (Utc::now() - last_rotation).num_days();
            Ok(days_since_rotation >= self.rotation_interval_days as i64)
        } else {
            // Never rotated, so rotation is needed
            Ok(true)
        }
    }

    /// Rotate credentials (placeholder implementation)
    pub async fn rotate_credentials(&self) -> CloudResult<()> {
        tracing::info!("Starting credential rotation for provider: {}", self.provider);
        
        // In a real implementation, this would:
        // 1. Create new credentials with the cloud provider
        // 2. Test the new credentials
        // 3. Replace the old credentials
        // 4. Update rotation metadata
        // 5. Optionally revoke old credentials
        
        let mut metadata = self.get_credential_metadata().unwrap_or_default();
        metadata.last_rotated = Some(Utc::now());
        metadata.rotation_count += 1;
        
        self.save_credential_metadata(&metadata)?;
        
        tracing::info!("Credential rotation completed for provider: {}", self.provider);
        Ok(())
    }

    fn get_credential_metadata(&self) -> CloudResult<CredentialMetadata> {
        // In a real implementation, this would load from a secure location
        Ok(CredentialMetadata::default())
    }

    fn save_credential_metadata(&self, _metadata: &CredentialMetadata) -> CloudResult<()> {
        // In a real implementation, this would save to a secure location
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CredentialMetadata {
    last_rotated: Option<DateTime<Utc>>,
    rotation_count: u32,
    created_at: Option<DateTime<Utc>>,
}

/// Interactive credential setup helper
pub struct CredentialSetup;

impl CredentialSetup {
    /// Interactive setup for AWS credentials
    pub fn setup_aws_credentials() -> CloudResult<CloudCredentials> {
        use std::io::{self, Write};

        print!("Enter AWS Access Key ID: ");
        io::stdout().flush().unwrap();
        let mut access_key_id = String::new();
        io::stdin().read_line(&mut access_key_id).unwrap();
        let access_key_id = access_key_id.trim().to_string();

        print!("Enter AWS Secret Access Key: ");
        io::stdout().flush().unwrap();
        let mut secret_access_key = String::new();
        io::stdin().read_line(&mut secret_access_key).unwrap();
        let secret_access_key = secret_access_key.trim().to_string();

        print!("Enter AWS Region (default: us-east-1): ");
        io::stdout().flush().unwrap();
        let mut region = String::new();
        io::stdin().read_line(&mut region).unwrap();
        let region = region.trim();
        let region = if region.is_empty() {
            "us-east-1".to_string()
        } else {
            region.to_string()
        };

        print!("Enter AWS Session Token (optional, press enter to skip): ");
        io::stdout().flush().unwrap();
        let mut session_token = String::new();
        io::stdin().read_line(&mut session_token).unwrap();
        let session_token = session_token.trim();
        let session_token = if session_token.is_empty() {
            None
        } else {
            Some(session_token.to_string())
        };

        Ok(CloudCredentials::AwsCredentials {
            access_key_id,
            secret_access_key,
            session_token,
            region,
        })
    }

    /// Validate and store credentials interactively
    pub async fn setup_and_validate_credentials(provider: &str) -> CloudResult<()> {
        let credentials = match provider {
            "s3" | "aws" => Self::setup_aws_credentials()?,
            _ => {
                return Err(CloudError::Configuration(
                    format!("Interactive setup not available for provider: {}", provider)
                ));
            }
        };

        let credential_provider = CredentialProvider::new(provider.to_string());
        
        // Test credentials before storing
        println!("Testing credentials...");
        
        // For S3, we need to test with a real bucket name
        print!("Enter S3 bucket name to test with: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        let mut bucket = String::new();
        io::stdin().read_line(&mut bucket).unwrap();
        let bucket = bucket.trim().to_string();

        let s3_provider = super::s3::S3Provider::new(bucket, "us-east-1".to_string());
        match s3_provider.authenticate(&credentials).await {
            Ok(_) => {
                println!("✅ Credentials are valid!");
                credential_provider.store_credentials(&credentials)?;
                println!("✅ Credentials stored securely!");
                Ok(())
            }
            Err(e) => {
                println!("❌ Credential validation failed: {}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_provider_creation() {
        let provider = CredentialProvider::new("s3".to_string());
        assert_eq!(provider.provider_name, "s3");
    }

    #[test]
    fn test_credential_metadata_default() {
        let metadata = CredentialMetadata::default();
        assert!(metadata.last_rotated.is_none());
        assert_eq!(metadata.rotation_count, 0);
        assert!(metadata.created_at.is_none());
    }

    #[tokio::test]
    async fn test_rotation_manager() {
        let manager = CredentialRotationManager::new("s3".to_string(), 30);
        
        // Should need rotation for new credentials
        assert!(manager.needs_rotation().unwrap_or(false));
    }
}