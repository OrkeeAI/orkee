// ABOUTME: API key encryption using ChaCha20-Poly1305 AEAD
// ABOUTME: Derives encryption key from machine ID + application salt

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
}

impl From<Unspecified> for EncryptionError {
    fn from(_: Unspecified) -> Self {
        EncryptionError::Encryption("Cryptographic operation failed".to_string())
    }
}

/// API key encryption service
pub struct ApiKeyEncryption {
    rng: Arc<SystemRandom>,
    encryption_key: Vec<u8>,
}

impl ApiKeyEncryption {
    /// Create new encryption service with machine-derived key
    pub fn new() -> Result<Self, EncryptionError> {
        let machine_id = machine_uid::get()
            .map_err(|e| EncryptionError::KeyDerivation(format!("Failed to get machine ID: {}", e)))?;

        // Derive 256-bit key from machine ID + app salt using BLAKE2b
        let mut key_material = Vec::with_capacity(machine_id.len() + APP_SALT.len());
        key_material.extend_from_slice(machine_id.as_bytes());
        key_material.extend_from_slice(APP_SALT);

        // Use HKDF to derive a proper encryption key
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
        })
    }

    /// Encrypt an API key
    /// Returns base64-encoded: nonce || ciphertext || tag
    pub fn encrypt(&self, plaintext: &str) -> Result<String, EncryptionError> {
        if plaintext.is_empty() {
            return Ok(String::new());
        }

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        self.rng
            .fill(&mut nonce_bytes)
            .map_err(|_| EncryptionError::RandomGeneration("Failed to generate nonce".to_string()))?;

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
    }
}
