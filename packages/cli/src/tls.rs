use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum_server::tls_rustls::RustlsConfig;
use rcgen::{Certificate as RcgenCertificate, CertificateParams, DistinguishedName};
use rustls::{ServerConfig, Certificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};
use thiserror::Error;
use tracing::{info, warn, error, debug};

use crate::error::AppError;

/// TLS configuration and certificate management
#[derive(Clone)]
pub struct TlsManager {
    config: TlsConfig,
}

/// TLS configuration settings
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub auto_generate: bool,
}

/// TLS-related errors
#[derive(Debug, Error)]
pub enum TlsError {
    #[error("Certificate file not found: {0}")]
    CertificateNotFound(String),
    
    #[error("Private key file not found: {0}")]
    PrivateKeyNotFound(String),
    
    #[error("Invalid certificate format: {0}")]
    InvalidCertificate(String),
    
    #[error("Invalid private key format: {0}")]
    InvalidPrivateKey(String),
    
    #[error("Certificate generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("Certificate expired")]
    CertificateExpired,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("TLS configuration error: {0}")]
    ConfigError(String),
}

impl From<TlsError> for AppError {
    fn from(err: TlsError) -> Self {
        AppError::Internal(anyhow::anyhow!("TLS error: {}", err))
    }
}

impl TlsManager {
    /// Create a new TLS manager with the given configuration
    pub fn new(config: TlsConfig) -> Self {
        Self { config }
    }
    
    /// Initialize TLS configuration, generating certificates if needed
    pub async fn initialize(&self) -> Result<RustlsConfig, TlsError> {
        if !self.config.enabled {
            return Err(TlsError::ConfigError("TLS is not enabled".to_string()));
        }
        
        info!("Initializing TLS configuration");
        debug!("Certificate path: {}", self.config.cert_path.display());
        debug!("Private key path: {}", self.config.key_path.display());
        
        // Check if certificates exist and are valid
        if self.should_generate_certificate()? {
            if self.config.auto_generate {
                info!("Generating self-signed certificate for development");
                self.generate_self_signed_certificate().await?;
            } else {
                return Err(TlsError::CertificateNotFound(
                    self.config.cert_path.display().to_string()
                ));
            }
        }
        
        // Load the certificates
        let rustls_config = self.load_certificates().await?;
        
        info!("TLS configuration initialized successfully");
        Ok(rustls_config)
    }
    
    /// Check if we should generate a new certificate
    fn should_generate_certificate(&self) -> Result<bool, TlsError> {
        // Check if certificate files exist
        if !self.config.cert_path.exists() || !self.config.key_path.exists() {
            debug!("Certificate or key file missing, need to generate");
            return Ok(true);
        }
        
        // Check if certificate is expired or expiring soon
        match self.check_certificate_validity() {
            Ok(true) => {
                debug!("Certificate is valid, no need to generate");
                Ok(false)
            }
            Ok(false) => {
                info!("Certificate is expired or expiring soon, need to regenerate");
                Ok(true)
            }
            Err(e) => {
                warn!("Failed to check certificate validity: {}, will regenerate", e);
                Ok(true)
            }
        }
    }
    
    /// Check if the certificate is valid and not expiring soon (within 30 days)
    fn check_certificate_validity(&self) -> Result<bool, TlsError> {
        let cert_data = fs::read(&self.config.cert_path)?;
        
        // Parse the certificate to check expiry
        let mut cursor = std::io::Cursor::new(cert_data);
        let certs = certs(&mut cursor)
            .map_err(|e| TlsError::InvalidCertificate(e.to_string()))?;
        
        if certs.is_empty() {
            return Err(TlsError::InvalidCertificate("No certificates found".to_string()));
        }
        
        // For self-signed certificates, we'll do a basic check
        // In a real implementation, you might want to parse the X.509 certificate
        // and check the actual expiry date
        
        // For now, assume certificates older than 30 days should be regenerated
        let metadata = fs::metadata(&self.config.cert_path)?;
        let created = metadata.created().or_else(|_| metadata.modified())?;
        let age = created.elapsed().unwrap_or(Duration::from_secs(0));
        
        // Consider certificates older than 335 days (30 days before 365 day expiry) as needing renewal
        const RENEWAL_THRESHOLD: Duration = Duration::from_secs(335 * 24 * 60 * 60);
        
        Ok(age < RENEWAL_THRESHOLD)
    }
    
    /// Generate a self-signed certificate for development use
    async fn generate_self_signed_certificate(&self) -> Result<(), TlsError> {
        // Create certificate directory if it doesn't exist
        if let Some(parent) = self.config.cert_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Generate certificate
        let mut params = CertificateParams::new(vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "::1".to_string(),
            "orkee.local".to_string(),
            "dev.orkee.local".to_string(),
        ]);
        
        params.distinguished_name = DistinguishedName::new();
        params.distinguished_name.push(rcgen::DnType::CommonName, "Orkee Development Certificate");
        params.distinguished_name.push(rcgen::DnType::OrganizationName, "Orkee");
        params.distinguished_name.push(rcgen::DnType::CountryName, "US");
        
        // Set validity period to 1 year
        params.not_before = rcgen::date_time_ymd(2024, 1, 1);
        params.not_after = rcgen::date_time_ymd(2025, 12, 31);
        
        let cert = RcgenCertificate::from_params(params)
            .map_err(|e| TlsError::GenerationFailed(e.to_string()))?;
        
        // Write certificate to file
        let cert_pem = cert.serialize_pem()
            .map_err(|e| TlsError::GenerationFailed(e.to_string()))?;
        fs::write(&self.config.cert_path, cert_pem)?;
        
        // Write private key to file
        let key_pem = cert.serialize_private_key_pem();
        fs::write(&self.config.key_path, key_pem)?;
        
        // Set appropriate permissions on the key file (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.config.key_path)?.permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&self.config.key_path, perms)?;
        }
        
        info!("Generated self-signed certificate for development use");
        info!("Certificate: {}", self.config.cert_path.display());
        info!("Private key: {}", self.config.key_path.display());
        warn!("⚠️  Self-signed certificate - browsers will show security warnings");
        warn!("   Add security exception or use a reverse proxy with proper certificates");
        
        Ok(())
    }
    
    /// Load certificates from files and create Rustls configuration
    async fn load_certificates(&self) -> Result<RustlsConfig, TlsError> {
        // Load certificate file
        let cert_file = fs::File::open(&self.config.cert_path)?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_data = certs(&mut cert_reader)
            .map_err(|e| TlsError::InvalidCertificate(e.to_string()))?;
        let cert_chain: Vec<Certificate> = cert_data
            .into_iter()
            .map(Certificate)
            .collect();
        
        if cert_chain.is_empty() {
            return Err(TlsError::InvalidCertificate("No certificates found in file".to_string()));
        }
        
        // Load private key file
        let key_file = fs::File::open(&self.config.key_path)?;
        let mut key_reader = BufReader::new(key_file);
        let key_data = pkcs8_private_keys(&mut key_reader)
            .map_err(|e| TlsError::InvalidPrivateKey(e.to_string()))?;
        let mut keys: Vec<PrivateKey> = key_data
            .into_iter()
            .map(PrivateKey)
            .collect();
        
        if keys.is_empty() {
            return Err(TlsError::InvalidPrivateKey("No private keys found in file".to_string()));
        }
        
        // Use the first key
        let private_key = keys.remove(0);
        
        // Create Rustls configuration with rustls 0.21 API
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)
            .map_err(|e| TlsError::ConfigError(e.to_string()))?;
        
        debug!("Loaded certificate(s) and private key successfully");
        
        Ok(RustlsConfig::from_config(Arc::new(config)))
    }
    
    /// Get the default certificate directory
    pub fn default_cert_dir() -> PathBuf {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();
        
        if home_dir.is_empty() {
            PathBuf::from(".orkee/certs")
        } else {
            PathBuf::from(home_dir).join(".orkee").join("certs")
        }
    }
    
    /// Get TLS configuration info for logging
    pub fn get_config_info(&self) -> TlsConfigInfo {
        TlsConfigInfo {
            enabled: self.config.enabled,
            cert_path: self.config.cert_path.clone(),
            key_path: self.config.key_path.clone(),
            auto_generate: self.config.auto_generate,
            cert_exists: self.config.cert_path.exists(),
            key_exists: self.config.key_path.exists(),
        }
    }
}

/// Information about TLS configuration for logging/debugging
#[derive(Debug)]
pub struct TlsConfigInfo {
    pub enabled: bool,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub auto_generate: bool,
    pub cert_exists: bool,
    pub key_exists: bool,
}

impl std::fmt::Display for TlsConfigInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TLS enabled: {}, auto-generate: {}, cert exists: {}, key exists: {}", 
               self.enabled, self.auto_generate, self.cert_exists, self.key_exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_default_cert_dir() {
        let cert_dir = TlsManager::default_cert_dir();
        assert!(cert_dir.ends_with(".orkee/certs"));
    }
    
    #[test]
    fn test_tls_config_creation() {
        let temp_dir = tempdir().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");
        
        let config = TlsConfig {
            enabled: true,
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            auto_generate: true,
        };
        
        let manager = TlsManager::new(config);
        let info = manager.get_config_info();
        
        assert!(info.enabled);
        assert!(info.auto_generate);
        assert_eq!(info.cert_path, cert_path);
        assert_eq!(info.key_path, key_path);
        assert!(!info.cert_exists);
        assert!(!info.key_exists);
    }
    
    #[tokio::test]
    async fn test_self_signed_generation() {
        let temp_dir = tempdir().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");
        
        let config = TlsConfig {
            enabled: true,
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            auto_generate: true,
        };
        
        let manager = TlsManager::new(config);
        
        // Generate certificate
        manager.generate_self_signed_certificate().await.unwrap();
        
        // Check that files were created
        assert!(cert_path.exists());
        assert!(key_path.exists());
        
        // Check that files contain valid PEM data
        let cert_content = fs::read_to_string(&cert_path).unwrap();
        assert!(cert_content.contains("-----BEGIN CERTIFICATE-----"));
        assert!(cert_content.contains("-----END CERTIFICATE-----"));
        
        let key_content = fs::read_to_string(&key_path).unwrap();
        assert!(key_content.contains("-----BEGIN PRIVATE KEY-----"));
        assert!(key_content.contains("-----END PRIVATE KEY-----"));
    }
    
    #[tokio::test]
    async fn test_certificate_loading() {
        let temp_dir = tempdir().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");
        
        let config = TlsConfig {
            enabled: true,
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            auto_generate: true,
        };
        
        let manager = TlsManager::new(config);
        
        // Generate and load certificate
        manager.generate_self_signed_certificate().await.unwrap();
        let _rustls_config = manager.load_certificates().await.unwrap();
        
        // Verify we got a valid configuration
        // If we got here without error, the certificate loading was successful
        // RustlsConfig doesn't expose much for testing, but successful creation means certificates are valid
        assert!(true); // This test passes if we get here without panicking
    }

    #[tokio::test]
    async fn test_tls_initialization_workflow() {
        let temp_dir = tempdir().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");
        
        let config = TlsConfig {
            enabled: true,
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            auto_generate: true,
        };
        
        let manager = TlsManager::new(config);
        
        // Initialize TLS (should auto-generate certificates)
        let _rustls_config = manager.initialize().await.unwrap();
        
        // Verify certificates were created
        assert!(cert_path.exists());
        assert!(key_path.exists());
        
        // Verify configuration is valid
        // If we got here without error, the TLS initialization was successful
        assert!(true); // This test passes if we get here without panicking
    }

    #[tokio::test]
    async fn test_certificate_validity_checking() {
        let temp_dir = tempdir().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");
        
        let config = TlsConfig {
            enabled: true,
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            auto_generate: true,
        };
        
        let manager = TlsManager::new(config);
        
        // Generate certificate
        manager.generate_self_signed_certificate().await.unwrap();
        
        // Check validity (should be valid for fresh certificate)
        let is_valid = manager.check_certificate_validity().unwrap();
        assert!(is_valid, "Fresh certificate should be valid");
        
        // Test should_generate_certificate logic
        let should_generate = manager.should_generate_certificate().unwrap();
        assert!(!should_generate, "Should not regenerate valid certificate");
    }

    #[tokio::test]
    async fn test_missing_certificate_files() {
        let temp_dir = tempdir().unwrap();
        let cert_path = temp_dir.path().join("nonexistent_cert.pem");
        let key_path = temp_dir.path().join("nonexistent_key.pem");
        
        let config = TlsConfig {
            enabled: true,
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            auto_generate: false, // Disable auto-generation
        };
        
        let manager = TlsManager::new(config);
        
        // Initialize should fail when files don't exist and auto-generation is disabled
        let result = manager.initialize().await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            TlsError::CertificateNotFound(_) => {}, // Expected error
            other => panic!("Expected CertificateNotFound error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_invalid_certificate_content() {
        let temp_dir = tempdir().unwrap();
        let cert_path = temp_dir.path().join("invalid_cert.pem");
        let key_path = temp_dir.path().join("invalid_key.pem");
        
        // Write invalid certificate content
        fs::write(&cert_path, "invalid certificate content").unwrap();
        fs::write(&key_path, "invalid key content").unwrap();
        
        let config = TlsConfig {
            enabled: true,
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            auto_generate: false,
        };
        
        let manager = TlsManager::new(config);
        
        // Loading invalid certificates should fail
        let result = manager.load_certificates().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tls_disabled_error() {
        let temp_dir = tempdir().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");
        
        let config = TlsConfig {
            enabled: false, // TLS disabled
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            auto_generate: true,
        };
        
        let manager = TlsManager::new(config);
        
        // Initialize should fail when TLS is disabled
        let result = manager.initialize().await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            TlsError::ConfigError(msg) => {
                assert!(msg.contains("TLS is not enabled"));
            },
            other => panic!("Expected ConfigError, got: {:?}", other),
        }
    }

    #[test]
    fn test_tls_config_info_display() {
        let temp_dir = tempdir().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");
        
        let config = TlsConfig {
            enabled: true,
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            auto_generate: true,
        };
        
        let manager = TlsManager::new(config);
        let info = manager.get_config_info();
        
        let display_str = format!("{}", info);
        assert!(display_str.contains("TLS enabled: true"));
        assert!(display_str.contains("auto-generate: true"));
        assert!(display_str.contains("cert exists: false"));
        assert!(display_str.contains("key exists: false"));
    }

    #[test]
    fn test_tls_error_conversions() {
        use std::io::{Error as IoError, ErrorKind};
        
        // Test IO error conversion
        let io_error = IoError::new(ErrorKind::PermissionDenied, "Permission denied");
        let tls_error = TlsError::IoError(io_error);
        let app_error: crate::error::AppError = tls_error.into();
        
        // Should convert to AppError::Internal
        match app_error {
            crate::error::AppError::Internal(_) => {}, // Expected
            other => panic!("Expected AppError::Internal, got: {:?}", other),
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_private_key_permissions() {
        let temp_dir = tempdir().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");
        
        let config = TlsConfig {
            enabled: true,
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
            auto_generate: true,
        };
        
        let manager = TlsManager::new(config);
        
        // Generate certificate
        manager.generate_self_signed_certificate().await.unwrap();
        
        // Check that private key has correct permissions (0o600)
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&key_path).unwrap();
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o600, "Private key should have 600 permissions");
    }
}