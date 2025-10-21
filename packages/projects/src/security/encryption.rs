// ABOUTME: API key encryption using ChaCha20-Poly1305 AEAD
// ABOUTME: Supports machine-based (default) and password-based key derivation
//
// SECURITY MODEL:
//
// Machine-Based Encryption (Default):
// - Derives encryption key from machine ID + application salt
// - Provides "transport encryption" - protects data during backup/sync
// - Does NOT provide true "at-rest encryption" on the local machine
// - Anyone with local file access can decrypt API keys on the same machine
// - Key is deterministic and reproducible based on machine ID
//
// Password-Based Encryption (Opt-in):
// - Derives encryption key from user-provided password using Argon2id
// - Provides true "at-rest encryption" - data cannot be decrypted without password
// - User must enter password when starting Orkee or accessing encrypted data
// - Much stronger security model for sensitive environments
//
// Migration: Use `orkee security set-password` to upgrade to password-based encryption

use argon2::{Argon2, ParamsBuilder, Version};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ring::{
    aead::{self, Nonce, UnboundKey},
    error::Unspecified,
    rand::{SecureRandom, SystemRandom},
};
use std::sync::Arc;

/// Application salt for key derivation (constant, not secret)
const APP_SALT: &[u8] = b"orkee-api-key-encryption-v1-20250120";

/// Nonce size for ChaCha20-Poly1305
const NONCE_SIZE: usize = 12;

/// Encryption mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMode {
    /// Machine-based encryption (transport encryption only)
    Machine,
    /// Password-based encryption (at-rest encryption)
    Password,
}

impl std::fmt::Display for EncryptionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionMode::Machine => write!(f, "machine"),
            EncryptionMode::Password => write!(f, "password"),
        }
    }
}

impl std::str::FromStr for EncryptionMode {
    type Err = EncryptionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "machine" => Ok(EncryptionMode::Machine),
            "password" => Ok(EncryptionMode::Password),
            _ => Err(EncryptionError::InvalidMode(s.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Failed to generate random data: {0}")]
    RandomGeneration(String),

    #[error("Failed to encrypt data: {0}")]
    Encryption(String),

    #[error("Failed to decrypt data: {0}")]
    Decryption(String),

    #[error("Failed to derive encryption key: {0}")]
    KeyDerivation(String),

    #[error("Invalid encrypted data format")]
    InvalidFormat,

    #[error("Invalid encryption mode: {0}")]
    InvalidMode(String),

    #[error("Password required for password-based encryption")]
    PasswordRequired,

    #[error("Invalid password")]
    InvalidPassword,
}

impl From<Unspecified> for EncryptionError {
    fn from(_: Unspecified) -> Self {
        EncryptionError::Encryption("Cryptographic operation failed".to_string())
    }
}

/// API key encryption service
#[derive(Debug)]
pub struct ApiKeyEncryption {
    rng: Arc<SystemRandom>,
    encryption_key: Vec<u8>,
    mode: EncryptionMode,
}

impl ApiKeyEncryption {
    /// Create new encryption service with machine-derived key (backward compatible)
    pub fn new() -> Result<Self, EncryptionError> {
        Self::with_machine_key()
    }

    /// Create encryption service with machine-derived key (transport encryption)
    pub fn with_machine_key() -> Result<Self, EncryptionError> {
        let machine_id = machine_uid::get().map_err(|e| {
            EncryptionError::KeyDerivation(format!("Failed to get machine ID: {}", e))
        })?;

        // Derive 256-bit key from machine ID + app salt using HKDF
        let mut key_material = Vec::with_capacity(machine_id.len() + APP_SALT.len());
        key_material.extend_from_slice(machine_id.as_bytes());
        key_material.extend_from_slice(APP_SALT);

        use ring::hkdf;
        let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, b"orkee-encryption-salt");
        let prk = salt.extract(&key_material);

        let mut encryption_key = vec![0u8; 32]; // 256-bit key
        prk.expand(&[b"api-key-encryption"], hkdf::HKDF_SHA256)
            .map_err(|_| EncryptionError::KeyDerivation("HKDF expansion failed".to_string()))?
            .fill(&mut encryption_key)
            .map_err(|_| EncryptionError::KeyDerivation("Key fill failed".to_string()))?;

        Ok(Self {
            rng: Arc::new(SystemRandom::new()),
            encryption_key,
            mode: EncryptionMode::Machine,
        })
    }

    /// Create encryption service with password-derived key (at-rest encryption)
    pub fn with_password(password: &str, salt: &[u8]) -> Result<Self, EncryptionError> {
        if password.is_empty() {
            return Err(EncryptionError::PasswordRequired);
        }

        if salt.len() != 32 {
            return Err(EncryptionError::KeyDerivation(
                "Salt must be 32 bytes".to_string(),
            ));
        }

        // Derive encryption key using Argon2id with recommended parameters
        let mut encryption_key = vec![0u8; 32]; // 256-bit key

        // Use Argon2id with moderate parameters (balance security/performance)
        // Memory: 64 MB, Iterations: 3, Parallelism: 4
        let params = ParamsBuilder::new()
            .m_cost(65536) // 64 MB
            .t_cost(3)     // 3 iterations
            .p_cost(4)     // 4 parallel threads
            .output_len(32) // 256-bit key
            .build()
            .map_err(|e| EncryptionError::KeyDerivation(format!("Invalid Argon2 params: {}", e)))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        argon2
            .hash_password_into(password.as_bytes(), salt, &mut encryption_key)
            .map_err(|e| EncryptionError::KeyDerivation(format!("Argon2 derivation failed: {}", e)))?;

        Ok(Self {
            rng: Arc::new(SystemRandom::new()),
            encryption_key,
            mode: EncryptionMode::Password,
        })
    }

    /// Get the encryption mode
    pub fn mode(&self) -> EncryptionMode {
        self.mode
    }

    /// Generate a random salt for password-based encryption
    pub fn generate_salt() -> Result<Vec<u8>, EncryptionError> {
        let mut salt = vec![0u8; 32];
        let rng = SystemRandom::new();
        rng.fill(&mut salt)
            .map_err(|_| EncryptionError::RandomGeneration("Failed to generate salt".to_string()))?;
        Ok(salt)
    }

    /// Hash a password for verification (NOT the encryption key)
    pub fn hash_password_for_verification(
        password: &str,
        salt: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        if password.is_empty() {
            return Err(EncryptionError::PasswordRequired);
        }

        if salt.len() != 32 {
            return Err(EncryptionError::KeyDerivation(
                "Salt must be 32 bytes".to_string(),
            ));
        }

        // Use a different input to ensure verification hash != encryption key
        let mut password_hash = vec![0u8; 32];

        let params = ParamsBuilder::new()
            .m_cost(65536)
            .t_cost(3)
            .p_cost(4)
            .output_len(32)
            .build()
            .map_err(|e| EncryptionError::KeyDerivation(format!("Invalid Argon2 params: {}", e)))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        // Add context suffix to password to differentiate verification from encryption
        let password_with_context = format!("{}|verification", password);
        argon2
            .hash_password_into(password_with_context.as_bytes(), salt, &mut password_hash)
            .map_err(|e| EncryptionError::KeyDerivation(format!("Argon2 derivation failed: {}", e)))?;

        Ok(password_hash)
    }

    /// Verify a password against a stored hash
    pub fn verify_password(
        password: &str,
        salt: &[u8],
        stored_hash: &[u8],
    ) -> Result<bool, EncryptionError> {
        let computed_hash = Self::hash_password_for_verification(password, salt)?;
        Ok(computed_hash == stored_hash)
    }

    /// Encrypt an API key
    /// Returns base64-encoded: nonce || ciphertext || tag
    pub fn encrypt(&self, plaintext: &str) -> Result<String, EncryptionError> {
        if plaintext.is_empty() {
            return Ok(String::new());
        }

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        self.rng.fill(&mut nonce_bytes).map_err(|_| {
            EncryptionError::RandomGeneration("Failed to generate nonce".to_string())
        })?;

        let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)?;

        // Create sealing key
        let unbound_key = UnboundKey::new(&aead::CHACHA20_POLY1305, &self.encryption_key)?;
        let sealing_key = aead::LessSafeKey::new(unbound_key);

        // Prepare data for encryption
        let mut in_out = plaintext.as_bytes().to_vec();

        // Encrypt in place
        sealing_key
            .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| EncryptionError::Encryption("Seal operation failed".to_string()))?;

        // Combine nonce + ciphertext+tag
        let mut result = Vec::with_capacity(NONCE_SIZE + in_out.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&in_out);

        Ok(BASE64.encode(&result))
    }

    /// Decrypt an API key
    /// Expects base64-encoded: nonce || ciphertext || tag
    pub fn decrypt(&self, ciphertext: &str) -> Result<String, EncryptionError> {
        if ciphertext.is_empty() {
            return Ok(String::new());
        }

        // Decode from base64
        let encrypted_data = BASE64
            .decode(ciphertext)
            .map_err(|_| EncryptionError::InvalidFormat)?;

        if encrypted_data.len() < NONCE_SIZE + aead::CHACHA20_POLY1305.tag_len() {
            return Err(EncryptionError::InvalidFormat);
        }

        // Extract nonce
        let (nonce_bytes, ciphertext_and_tag) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes)?;

        // Create opening key
        let unbound_key = UnboundKey::new(&aead::CHACHA20_POLY1305, &self.encryption_key)?;
        let opening_key = aead::LessSafeKey::new(unbound_key);

        // Decrypt in place
        let mut in_out = ciphertext_and_tag.to_vec();
        let plaintext = opening_key
            .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| EncryptionError::Decryption("Open operation failed".to_string()))?;

        String::from_utf8(plaintext.to_vec())
            .map_err(|_| EncryptionError::Decryption("Invalid UTF-8 in decrypted data".to_string()))
    }

    /// Check if a value is encrypted (base64 with sufficient length)
    pub fn is_encrypted(value: &str) -> bool {
        if value.is_empty() {
            return false;
        }

        // Try to decode as base64 and check minimum length
        if let Ok(decoded) = BASE64.decode(value) {
            decoded.len() >= NONCE_SIZE + aead::CHACHA20_POLY1305.tag_len()
        } else {
            false
        }
    }
}

impl Default for ApiKeyEncryption {
    fn default() -> Self {
        Self::new().expect("Failed to initialize API key encryption")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let encryption = ApiKeyEncryption::new().unwrap();
        let plaintext = "sk-test-1234567890abcdef";

        let encrypted = encryption.encrypt(plaintext).unwrap();
        assert!(!encrypted.is_empty());
        assert_ne!(encrypted, plaintext);

        let decrypted = encryption.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_empty_string() {
        let encryption = ApiKeyEncryption::new().unwrap();
        let encrypted = encryption.encrypt("").unwrap();
        assert_eq!(encrypted, "");

        let decrypted = encryption.decrypt("").unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_different_nonces() {
        let encryption = ApiKeyEncryption::new().unwrap();
        let plaintext = "sk-test-key";

        let encrypted1 = encryption.encrypt(plaintext).unwrap();
        let encrypted2 = encryption.encrypt(plaintext).unwrap();

        // Same plaintext should produce different ciphertext (different nonces)
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to same plaintext
        assert_eq!(encryption.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(encryption.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_is_encrypted() {
        let encryption = ApiKeyEncryption::new().unwrap();
        let plaintext = "sk-test-key";
        let encrypted = encryption.encrypt(plaintext).unwrap();

        assert!(ApiKeyEncryption::is_encrypted(&encrypted));
        assert!(!ApiKeyEncryption::is_encrypted(plaintext));
        assert!(!ApiKeyEncryption::is_encrypted(""));
        assert!(!ApiKeyEncryption::is_encrypted("not-base64!@#"));
    }

    #[test]
    fn test_decrypt_invalid_data() {
        let encryption = ApiKeyEncryption::new().unwrap();

        // Invalid base64
        assert!(encryption.decrypt("not-valid-base64!@#").is_err());

        // Valid base64 but too short
        assert!(encryption.decrypt(&BASE64.encode(b"short")).is_err());

        // Valid base64 but wrong data
        let wrong_data = BASE64.encode(&vec![0u8; 50]);
        assert!(encryption.decrypt(&wrong_data).is_err());
    }

    #[test]
    fn test_machine_specific_keys() {
        // Keys should be derived from machine ID, so they should be consistent
        let encryption1 = ApiKeyEncryption::new().unwrap();
        let encryption2 = ApiKeyEncryption::new().unwrap();

        assert_eq!(encryption1.encryption_key, encryption2.encryption_key);
        assert_eq!(encryption1.mode(), EncryptionMode::Machine);
    }

    // Password-based encryption tests

    #[test]
    fn test_password_encryption_roundtrip() {
        let password = "my-secure-password-12345";
        let salt = ApiKeyEncryption::generate_salt().unwrap();

        let encryption = ApiKeyEncryption::with_password(password, &salt).unwrap();
        assert_eq!(encryption.mode(), EncryptionMode::Password);

        let plaintext = "sk-test-password-encrypted-key";
        let encrypted = encryption.encrypt(plaintext).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_password_encryption_same_salt_same_key() {
        let password = "my-password";
        let salt = ApiKeyEncryption::generate_salt().unwrap();

        let encryption1 = ApiKeyEncryption::with_password(password, &salt).unwrap();
        let encryption2 = ApiKeyEncryption::with_password(password, &salt).unwrap();

        // Same password + same salt = same encryption key
        assert_eq!(encryption1.encryption_key, encryption2.encryption_key);
    }

    #[test]
    fn test_password_encryption_different_salt_different_key() {
        let password = "my-password";
        let salt1 = ApiKeyEncryption::generate_salt().unwrap();
        let salt2 = ApiKeyEncryption::generate_salt().unwrap();

        let encryption1 = ApiKeyEncryption::with_password(password, &salt1).unwrap();
        let encryption2 = ApiKeyEncryption::with_password(password, &salt2).unwrap();

        // Same password + different salt = different encryption key
        assert_ne!(encryption1.encryption_key, encryption2.encryption_key);
    }

    #[test]
    fn test_password_encryption_wrong_password_fails_decrypt() {
        let salt = ApiKeyEncryption::generate_salt().unwrap();
        let plaintext = "sk-test-key";

        let encryption1 = ApiKeyEncryption::with_password("correct-password", &salt).unwrap();
        let encrypted = encryption1.encrypt(plaintext).unwrap();

        let encryption2 = ApiKeyEncryption::with_password("wrong-password", &salt).unwrap();
        let result = encryption2.decrypt(&encrypted);

        // Decryption should fail with wrong password
        assert!(result.is_err());
    }

    #[test]
    fn test_password_verification() {
        let password = "test-password-123";
        let salt = ApiKeyEncryption::generate_salt().unwrap();

        let hash = ApiKeyEncryption::hash_password_for_verification(password, &salt).unwrap();

        // Correct password should verify
        assert!(ApiKeyEncryption::verify_password(password, &salt, &hash).unwrap());

        // Wrong password should not verify
        assert!(!ApiKeyEncryption::verify_password("wrong-password", &salt, &hash).unwrap());
    }

    #[test]
    fn test_password_hash_not_encryption_key() {
        let password = "test-password";
        let salt = ApiKeyEncryption::generate_salt().unwrap();

        let verification_hash = ApiKeyEncryption::hash_password_for_verification(password, &salt).unwrap();
        let encryption = ApiKeyEncryption::with_password(password, &salt).unwrap();

        // Verification hash should be different from encryption key
        assert_ne!(verification_hash, encryption.encryption_key);
    }

    #[test]
    fn test_empty_password_fails() {
        let salt = ApiKeyEncryption::generate_salt().unwrap();
        let result = ApiKeyEncryption::with_password("", &salt);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EncryptionError::PasswordRequired));
    }

    #[test]
    fn test_invalid_salt_length_fails() {
        let password = "test-password";
        let short_salt = vec![0u8; 16]; // Only 16 bytes instead of 32

        let result = ApiKeyEncryption::with_password(password, &short_salt);
        assert!(result.is_err());
    }

    #[test]
    fn test_salt_generation_unique() {
        let salt1 = ApiKeyEncryption::generate_salt().unwrap();
        let salt2 = ApiKeyEncryption::generate_salt().unwrap();

        assert_eq!(salt1.len(), 32);
        assert_eq!(salt2.len(), 32);
        assert_ne!(salt1, salt2); // Salts should be unique
    }

    #[test]
    fn test_encryption_mode_display() {
        assert_eq!(EncryptionMode::Machine.to_string(), "machine");
        assert_eq!(EncryptionMode::Password.to_string(), "password");
    }

    #[test]
    fn test_encryption_mode_from_str() {
        assert_eq!("machine".parse::<EncryptionMode>().unwrap(), EncryptionMode::Machine);
        assert_eq!("password".parse::<EncryptionMode>().unwrap(), EncryptionMode::Password);
        assert_eq!("MACHINE".parse::<EncryptionMode>().unwrap(), EncryptionMode::Machine);
        assert_eq!("Password".parse::<EncryptionMode>().unwrap(), EncryptionMode::Password);

        assert!("invalid".parse::<EncryptionMode>().is_err());
    }

    #[test]
    fn test_machine_and_password_keys_different() {
        let machine_encryption = ApiKeyEncryption::with_machine_key().unwrap();

        let salt = ApiKeyEncryption::generate_salt().unwrap();
        let password_encryption = ApiKeyEncryption::with_password("test-password", &salt).unwrap();

        // Machine and password-based keys should be different
        assert_ne!(machine_encryption.encryption_key, password_encryption.encryption_key);
        assert_eq!(machine_encryption.mode(), EncryptionMode::Machine);
        assert_eq!(password_encryption.mode(), EncryptionMode::Password);
    }
}
