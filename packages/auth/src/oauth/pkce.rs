// ABOUTME: PKCE (Proof Key for Code Exchange) implementation for OAuth 2.0
// ABOUTME: Generates code verifiers and SHA256 challenges for secure OAuth flows

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};

use crate::{
    error::{AuthError, AuthResult},
    oauth::types::PkceChallenge,
};

/// Generate a PKCE challenge for OAuth flow
///
/// This generates a random code verifier and computes the SHA256 challenge
/// according to RFC 7636 (PKCE) specification.
pub fn generate_pkce_challenge() -> AuthResult<PkceChallenge> {
    // Generate code verifier (43-128 characters, alphanumeric)
    let code_verifier = generate_code_verifier()?;

    // Generate code challenge using S256 method
    let code_challenge = generate_code_challenge(&code_verifier)?;

    Ok(PkceChallenge {
        code_verifier,
        code_challenge,
        code_challenge_method: "S256".to_string(),
    })
}

/// Generate a random code verifier (43-128 characters)
fn generate_code_verifier() -> AuthResult<String> {
    let length = 64; // 64 characters is a good middle ground
    let verifier: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();

    if verifier.len() < 43 || verifier.len() > 128 {
        return Err(AuthError::Pkce(format!(
            "Invalid code verifier length: {}",
            verifier.len()
        )));
    }

    Ok(verifier)
}

/// Generate SHA256 code challenge from verifier
fn generate_code_challenge(verifier: &str) -> AuthResult<String> {
    // Compute SHA256 hash
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();

    // Base64 URL-safe encode (without padding)
    let challenge = URL_SAFE_NO_PAD.encode(hash);

    Ok(challenge)
}

/// Verify that a code verifier matches a code challenge
///
/// This is typically done by the authorization server, but can be useful
/// for testing and validation.
pub fn verify_pkce_challenge(verifier: &str, challenge: &str) -> bool {
    match generate_code_challenge(verifier) {
        Ok(computed_challenge) => computed_challenge == challenge,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code_verifier() {
        let verifier = generate_code_verifier().unwrap();
        assert!(verifier.len() >= 43 && verifier.len() <= 128);
        assert!(verifier.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_code_challenge() {
        let verifier = "test_verifier_1234567890_abcdefghijklmnopqrstuvwxyz";
        let challenge = generate_code_challenge(verifier).unwrap();

        // Challenge should be base64 URL-safe encoded (no padding)
        assert!(!challenge.contains('='));
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
    }

    #[test]
    fn test_verify_pkce_challenge() {
        let verifier = "test_verifier_1234567890_abcdefghijklmnopqrstuvwxyz";
        let challenge = generate_code_challenge(verifier).unwrap();

        assert!(verify_pkce_challenge(verifier, &challenge));
        assert!(!verify_pkce_challenge("wrong_verifier", &challenge));
    }

    #[test]
    fn test_generate_pkce_challenge() {
        let pkce = generate_pkce_challenge().unwrap();

        // Verify properties
        assert!(pkce.code_verifier.len() >= 43 && pkce.code_verifier.len() <= 128);
        assert_eq!(pkce.code_challenge_method, "S256");
        assert!(verify_pkce_challenge(
            &pkce.code_verifier,
            &pkce.code_challenge
        ));
    }

    #[test]
    fn test_pkce_deterministic() {
        // Same verifier should always produce same challenge
        let verifier = "test_verifier_constant";
        let challenge1 = generate_code_challenge(verifier).unwrap();
        let challenge2 = generate_code_challenge(verifier).unwrap();

        assert_eq!(challenge1, challenge2);
    }
}
