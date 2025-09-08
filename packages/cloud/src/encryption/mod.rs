use super::{CloudError, CloudResult};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use ring::aead::{Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub key_derivation: KeyDerivationConfig,
    pub compression_before_encryption: bool,
    pub include_integrity_check: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_derivation: KeyDerivationConfig::default(),
            compression_before_encryption: true,
            include_integrity_check: true,
        }
    }
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
}

/// Key derivation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationConfig {
    pub iterations: u32,
    pub memory_cost: u32,
    pub parallelism: u32,
    pub salt_length: usize,
}

impl Default for KeyDerivationConfig {
    fn default() -> Self {
        Self {
            iterations: 3,         // Argon2 time cost
            memory_cost: 65536,    // 64 MB memory cost
            parallelism: 4,        // 4 threads
            salt_length: 32,       // 32 byte salt
        }
    }
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub algorithm: EncryptionAlgorithm,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
    pub encrypted_data: Vec<u8>,
    pub metadata: EncryptionMetadata,
}

/// Encryption metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    pub version: u32,
    pub original_size: usize,
    pub encrypted_at: chrono::DateTime<chrono::Utc>,
    pub key_derivation_params: KeyDerivationConfig,
    pub checksum: Option<String>,
}

/// Encryption key manager
pub struct EncryptionKeyManager {
    config: EncryptionConfig,
    rng: SystemRandom,
}

impl EncryptionKeyManager {
    pub fn new(config: EncryptionConfig) -> Self {
        Self {
            config,
            rng: SystemRandom::new(),
        }
    }

    /// Derive encryption key from passphrase using Argon2
    pub fn derive_key_from_passphrase(
        &self,
        passphrase: &str,
        salt: Option<&[u8]>,
    ) -> CloudResult<(Vec<u8>, Vec<u8>)> {
        let salt = match salt {
            Some(s) => s.to_vec(),
            None => {
                let mut salt_bytes = vec![0u8; self.config.key_derivation.salt_length];
                self.rng.fill(&mut salt_bytes)
                    .map_err(|e| CloudError::Encryption(format!("Failed to generate salt: {:?}", e)))?;
                salt_bytes
            }
        };

        let salt_string = SaltString::encode_b64(&salt)
            .map_err(|e| CloudError::Encryption(format!("Failed to encode salt: {}", e)))?;

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                self.config.key_derivation.memory_cost,
                self.config.key_derivation.iterations,
                self.config.key_derivation.parallelism,
                Some(32), // 32-byte output for AES-256
            ).map_err(|e| CloudError::Encryption(format!("Invalid Argon2 params: {}", e)))?,
        );

        let password_hash = argon2
            .hash_password(passphrase.as_bytes(), &salt_string)
            .map_err(|e| CloudError::Encryption(format!("Failed to hash password: {}", e)))?;

        let hash_binding = password_hash.hash.unwrap();
        let key_bytes = hash_binding.as_bytes();
        Ok((key_bytes.to_vec(), salt))
    }

    /// Generate a new random encryption key
    pub fn generate_random_key(&self) -> CloudResult<Vec<u8>> {
        let mut key = vec![0u8; 32]; // 32 bytes for AES-256
        self.rng.fill(&mut key)
            .map_err(|e| CloudError::Encryption(format!("Failed to generate key: {:?}", e)))?;
        Ok(key)
    }

    /// Create a nonce for encryption
    fn generate_nonce(&self) -> CloudResult<Vec<u8>> {
        let mut nonce = vec![0u8; 12]; // 96-bit nonce for GCM
        self.rng.fill(&mut nonce)
            .map_err(|e| CloudError::Encryption(format!("Failed to generate nonce: {:?}", e)))?;
        Ok(nonce)
    }
}

/// Snapshot encryptor
pub struct SnapshotEncryptor {
    key_manager: EncryptionKeyManager,
}

impl SnapshotEncryptor {
    pub fn new(config: EncryptionConfig) -> Self {
        Self {
            key_manager: EncryptionKeyManager::new(config),
        }
    }

    /// Encrypt snapshot data using a passphrase
    pub fn encrypt_with_passphrase(
        &self,
        data: &[u8],
        passphrase: &str,
    ) -> CloudResult<EncryptedData> {
        let (key, salt) = self.key_manager.derive_key_from_passphrase(passphrase, None)?;
        self.encrypt_with_key(data, &key, Some(salt))
    }

    /// Encrypt snapshot data using a raw key
    pub fn encrypt_with_key(
        &self,
        data: &[u8],
        key: &[u8],
        salt: Option<Vec<u8>>,
    ) -> CloudResult<EncryptedData> {
        let nonce_bytes = self.key_manager.generate_nonce()?;
        let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
            .map_err(|_| CloudError::Encryption("Invalid nonce".to_string()))?;

        // Create sealing key
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|e| CloudError::Encryption(format!("Failed to create key: {:?}", e)))?;
        let mut sealing_key = SealingKey::new(unbound_key, SingleUseNonce(Some(nonce)));

        // Encrypt the data
        let mut encrypted_data = data.to_vec();
        sealing_key.seal_in_place_append_tag(Aad::empty(), &mut encrypted_data)
            .map_err(|e| CloudError::Encryption(format!("Encryption failed: {:?}", e)))?;

        // Calculate checksum of original data
        let checksum = if self.key_manager.config.include_integrity_check {
            Some(calculate_sha256_checksum(data))
        } else {
            None
        };

        let metadata = EncryptionMetadata {
            version: 1,
            original_size: data.len(),
            encrypted_at: chrono::Utc::now(),
            key_derivation_params: self.key_manager.config.key_derivation.clone(),
            checksum,
        };

        Ok(EncryptedData {
            algorithm: self.key_manager.config.algorithm.clone(),
            nonce: nonce_bytes,
            salt: salt.unwrap_or_default(),
            encrypted_data,
            metadata,
        })
    }

    /// Decrypt snapshot data using a passphrase
    pub fn decrypt_with_passphrase(
        &self,
        encrypted_data: &EncryptedData,
        passphrase: &str,
    ) -> CloudResult<Vec<u8>> {
        let (key, _) = self.key_manager.derive_key_from_passphrase(
            passphrase,
            Some(&encrypted_data.salt),
        )?;
        self.decrypt_with_key(encrypted_data, &key)
    }

    /// Decrypt snapshot data using a raw key
    pub fn decrypt_with_key(
        &self,
        encrypted_data: &EncryptedData,
        key: &[u8],
    ) -> CloudResult<Vec<u8>> {
        // Verify algorithm compatibility
        match encrypted_data.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                // Continue with AES-256-GCM decryption
            }
        }

        let nonce = Nonce::try_assume_unique_for_key(&encrypted_data.nonce)
            .map_err(|_| CloudError::Encryption("Invalid nonce in encrypted data".to_string()))?;

        // Create opening key
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|e| CloudError::Encryption(format!("Failed to create key: {:?}", e)))?;
        let mut opening_key = OpeningKey::new(unbound_key, SingleUseNonce(Some(nonce)));

        // Decrypt the data
        let mut data_to_decrypt = encrypted_data.encrypted_data.clone();
        let decrypted_data = opening_key
            .open_in_place(Aad::empty(), &mut data_to_decrypt)
            .map_err(|e| CloudError::Encryption(format!("Decryption failed: {:?}", e)))?;

        // Verify integrity if checksum is available
        if let Some(expected_checksum) = &encrypted_data.metadata.checksum {
            let actual_checksum = calculate_sha256_checksum(decrypted_data);
            if &actual_checksum != expected_checksum {
                return Err(CloudError::Encryption(
                    "Integrity check failed: checksum mismatch".to_string(),
                ));
            }
        }

        // Verify decrypted size
        if decrypted_data.len() != encrypted_data.metadata.original_size {
            return Err(CloudError::Encryption(format!(
                "Size mismatch: expected {}, got {}",
                encrypted_data.metadata.original_size,
                decrypted_data.len()
            )));
        }

        Ok(decrypted_data.to_vec())
    }

    /// Serialize encrypted data to bytes
    pub fn serialize_encrypted_data(&self, encrypted_data: &EncryptedData) -> CloudResult<Vec<u8>> {
        serde_json::to_vec(encrypted_data)
            .map_err(|e| CloudError::Encryption(format!("Failed to serialize encrypted data: {}", e)))
    }

    /// Deserialize encrypted data from bytes
    pub fn deserialize_encrypted_data(&self, data: &[u8]) -> CloudResult<EncryptedData> {
        serde_json::from_slice(data)
            .map_err(|e| CloudError::Encryption(format!("Failed to deserialize encrypted data: {}", e)))
    }
}

/// Single-use nonce sequence for AEAD
struct SingleUseNonce(Option<Nonce>);

impl NonceSequence for SingleUseNonce {
    fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
        self.0.take().ok_or(ring::error::Unspecified)
    }
}

/// Key management for encrypted snapshots
pub struct EncryptedSnapshotManager {
    encryptor: SnapshotEncryptor,
    key_cache: HashMap<String, Vec<u8>>, // In production, use secure storage
}

impl EncryptedSnapshotManager {
    pub fn new(config: EncryptionConfig) -> Self {
        Self {
            encryptor: SnapshotEncryptor::new(config),
            key_cache: HashMap::new(),
        }
    }

    /// Add a key to the cache (use with caution in production)
    pub fn cache_key(&mut self, key_id: String, key: Vec<u8>) {
        self.key_cache.insert(key_id, key);
    }

    /// Remove a key from the cache
    pub fn remove_cached_key(&mut self, key_id: &str) {
        self.key_cache.remove(key_id);
    }

    /// Clear all cached keys
    pub fn clear_key_cache(&mut self) {
        self.key_cache.clear();
    }

    /// Encrypt a snapshot with automatic key management
    pub fn encrypt_snapshot(
        &self,
        data: &[u8],
        key_source: KeySource,
    ) -> CloudResult<EncryptedData> {
        match key_source {
            KeySource::Passphrase(passphrase) => {
                self.encryptor.encrypt_with_passphrase(data, &passphrase)
            }
            KeySource::RawKey(key) => {
                self.encryptor.encrypt_with_key(data, &key, None)
            }
            KeySource::CachedKey(key_id) => {
                let key = self.key_cache.get(&key_id)
                    .ok_or_else(|| CloudError::Encryption(format!("Key '{}' not found in cache", key_id)))?;
                self.encryptor.encrypt_with_key(data, key, None)
            }
        }
    }

    /// Decrypt a snapshot with automatic key management
    pub fn decrypt_snapshot(
        &self,
        encrypted_data: &EncryptedData,
        key_source: KeySource,
    ) -> CloudResult<Vec<u8>> {
        match key_source {
            KeySource::Passphrase(passphrase) => {
                self.encryptor.decrypt_with_passphrase(encrypted_data, &passphrase)
            }
            KeySource::RawKey(key) => {
                self.encryptor.decrypt_with_key(encrypted_data, &key)
            }
            KeySource::CachedKey(key_id) => {
                let key = self.key_cache.get(&key_id)
                    .ok_or_else(|| CloudError::Encryption(format!("Key '{}' not found in cache", key_id)))?;
                self.encryptor.decrypt_with_key(encrypted_data, key)
            }
        }
    }

    /// Get information about an encrypted snapshot without decrypting
    pub fn get_encryption_info(&self, encrypted_data: &EncryptedData) -> EncryptionInfo {
        EncryptionInfo {
            algorithm: encrypted_data.algorithm.clone(),
            version: encrypted_data.metadata.version,
            original_size: encrypted_data.metadata.original_size,
            encrypted_size: encrypted_data.encrypted_data.len(),
            encrypted_at: encrypted_data.metadata.encrypted_at,
            has_integrity_check: encrypted_data.metadata.checksum.is_some(),
            key_derivation_params: encrypted_data.metadata.key_derivation_params.clone(),
        }
    }
}

/// Key source for encryption/decryption operations
pub enum KeySource {
    Passphrase(String),
    RawKey(Vec<u8>),
    CachedKey(String),
}

/// Information about encrypted data
#[derive(Debug)]
pub struct EncryptionInfo {
    pub algorithm: EncryptionAlgorithm,
    pub version: u32,
    pub original_size: usize,
    pub encrypted_size: usize,
    pub encrypted_at: chrono::DateTime<chrono::Utc>,
    pub has_integrity_check: bool,
    pub key_derivation_params: KeyDerivationConfig,
}

impl std::fmt::Display for EncryptionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Encryption Information:")?;
        writeln!(f, "  Algorithm: {:?}", self.algorithm)?;
        writeln!(f, "  Version: {}", self.version)?;
        writeln!(f, "  Original Size: {} bytes", self.original_size)?;
        writeln!(f, "  Encrypted Size: {} bytes", self.encrypted_size)?;
        writeln!(f, "  Encryption Overhead: {} bytes ({:.1}%)", 
            self.encrypted_size.saturating_sub(self.original_size),
            (self.encrypted_size as f64 / self.original_size.max(1) as f64 - 1.0) * 100.0
        )?;
        writeln!(f, "  Encrypted At: {}", self.encrypted_at)?;
        writeln!(f, "  Integrity Check: {}", if self.has_integrity_check { "Yes" } else { "No" })?;
        writeln!(f, "  Key Derivation:")?;
        writeln!(f, "    Iterations: {}", self.key_derivation_params.iterations)?;
        writeln!(f, "    Memory Cost: {} KB", self.key_derivation_params.memory_cost)?;
        writeln!(f, "    Parallelism: {}", self.key_derivation_params.parallelism)?;
        
        Ok(())
    }
}

/// Utility function to calculate SHA-256 checksum
fn calculate_sha256_checksum(data: &[u8]) -> String {
    use ring::digest::{Context, SHA256};
    
    let mut context = Context::new(&SHA256);
    context.update(data);
    let digest = context.finish();
    hex::encode(digest.as_ref())
}

/// Password strength checker
pub struct PasswordStrengthChecker;

impl PasswordStrengthChecker {
    /// Check password strength and provide recommendations
    pub fn check_strength(password: &str) -> PasswordStrength {
        let mut score = 0u32;
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Length check
        if password.len() >= 12 {
            score += 25;
        } else if password.len() >= 8 {
            score += 15;
        } else {
            issues.push("Password is too short".to_string());
            recommendations.push("Use at least 12 characters".to_string());
        }

        // Character variety
        let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digits = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let char_variety = [has_lowercase, has_uppercase, has_digits, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        match char_variety {
            4 => score += 25,
            3 => score += 15,
            2 => score += 5,
            _ => {
                issues.push("Limited character variety".to_string());
                recommendations.push("Include uppercase, lowercase, digits, and special characters".to_string());
            }
        }

        // Common patterns check (simplified)
        let common_patterns = [
            "123", "password", "admin", "user", "test", "qwerty", "abc"
        ];
        
        let lower_password = password.to_lowercase();
        if common_patterns.iter().any(|&pattern| lower_password.contains(pattern)) {
            issues.push("Contains common patterns".to_string());
            recommendations.push("Avoid common words and sequences".to_string());
            score = score.saturating_sub(20);
        } else {
            score += 25;
        }

        // Repetition check
        if password.len() > 0 {
            let unique_chars = password.chars().collect::<std::collections::HashSet<_>>().len();
            let repetition_ratio = unique_chars as f64 / password.len() as f64;
            
            if repetition_ratio > 0.7 {
                score += 25;
            } else if repetition_ratio > 0.5 {
                score += 10;
            } else {
                issues.push("Too many repeated characters".to_string());
                recommendations.push("Use more varied characters".to_string());
            }
        }

        let strength_level = match score {
            90..=100 => StrengthLevel::Excellent,
            70..=89 => StrengthLevel::Good,
            50..=69 => StrengthLevel::Fair,
            30..=49 => StrengthLevel::Weak,
            _ => StrengthLevel::VeryWeak,
        };

        PasswordStrength {
            score,
            level: strength_level,
            issues,
            recommendations,
        }
    }
}

/// Password strength assessment result
#[derive(Debug)]
pub struct PasswordStrength {
    pub score: u32,
    pub level: StrengthLevel,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum StrengthLevel {
    VeryWeak,
    Weak,
    Fair,
    Good,
    Excellent,
}

impl std::fmt::Display for StrengthLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrengthLevel::VeryWeak => write!(f, "Very Weak"),
            StrengthLevel::Weak => write!(f, "Weak"),
            StrengthLevel::Fair => write!(f, "Fair"),
            StrengthLevel::Good => write!(f, "Good"),
            StrengthLevel::Excellent => write!(f, "Excellent"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_config_default() {
        let config = EncryptionConfig::default();
        assert!(matches!(config.algorithm, EncryptionAlgorithm::Aes256Gcm));
        assert!(config.compression_before_encryption);
        assert!(config.include_integrity_check);
    }

    #[test]
    fn test_key_derivation() {
        let config = EncryptionConfig::default();
        let key_manager = EncryptionKeyManager::new(config);
        
        let (key1, salt1) = key_manager.derive_key_from_passphrase("test_password", None).unwrap();
        let (key2, _) = key_manager.derive_key_from_passphrase("test_password", Some(&salt1)).unwrap();
        
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32); // AES-256 key size
    }

    #[tokio::test]
    async fn test_encryption_roundtrip() {
        let config = EncryptionConfig::default();
        let encryptor = SnapshotEncryptor::new(config);
        
        let original_data = b"Hello, encrypted world! This is test data.";
        let passphrase = "super_secure_password_123";
        
        // Encrypt
        let encrypted = encryptor.encrypt_with_passphrase(original_data, passphrase).unwrap();
        assert_ne!(encrypted.encrypted_data, original_data);
        assert_eq!(encrypted.metadata.original_size, original_data.len());
        
        // Decrypt
        let decrypted = encryptor.decrypt_with_passphrase(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_password_strength_checker() {
        let weak = PasswordStrengthChecker::check_strength("123");
        assert_eq!(weak.level, StrengthLevel::VeryWeak);
        assert!(!weak.issues.is_empty());
        
        let strong = PasswordStrengthChecker::check_strength("MyVeryStrongP@ssw0rd123!");
        assert!(matches!(strong.level, StrengthLevel::Good | StrengthLevel::Excellent));
        
        let common = PasswordStrengthChecker::check_strength("password123");
        assert!(!common.issues.is_empty());
        assert!(common.issues.iter().any(|issue| issue.contains("common")));
    }

    #[test]
    fn test_encrypted_snapshot_manager() {
        let config = EncryptionConfig::default();
        let manager = EncryptedSnapshotManager::new(config);
        
        let data = b"Test snapshot data for encryption";
        let passphrase = "test_passphrase";
        
        // Encrypt with passphrase
        let encrypted = manager.encrypt_snapshot(data, KeySource::Passphrase(passphrase.to_string())).unwrap();
        
        // Decrypt with passphrase
        let decrypted = manager.decrypt_snapshot(&encrypted, KeySource::Passphrase(passphrase.to_string())).unwrap();
        assert_eq!(decrypted, data);
        
        // Test encryption info
        let info = manager.get_encryption_info(&encrypted);
        assert_eq!(info.original_size, data.len());
        assert!(info.encrypted_size > data.len()); // Should be larger due to metadata
    }

    #[test]
    fn test_checksum_calculation() {
        let data = b"test data";
        let checksum1 = calculate_sha256_checksum(data);
        let checksum2 = calculate_sha256_checksum(data);
        assert_eq!(checksum1, checksum2);
        
        let different_data = b"different test data";
        let checksum3 = calculate_sha256_checksum(different_data);
        assert_ne!(checksum1, checksum3);
    }
}