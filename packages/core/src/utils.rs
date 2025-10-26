// ABOUTME: Shared utility functions for Orkee
// ABOUTME: ID generation, path operations, compression utilities

use std::path::Path;
use tokio::fs;

/// Generate a unique project ID (8-character format for cloud compatibility)
pub fn generate_project_id() -> String {
    // Generate 8-character ID like nanoid
    use rand::Rng;
    const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Checks if a path exists
pub async fn path_exists(path: impl AsRef<Path>) -> bool {
    fs::metadata(path).await.is_ok()
}

/// Compress data using gzip
pub fn compress_data(data: &[u8]) -> Result<Vec<u8>, String> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| format!("Compression failed: {}", e))?;
    encoder
        .finish()
        .map_err(|e| format!("Compression finish failed: {}", e))
}

/// Decompress gzipped data
pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>, String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("Decompression failed: {}", e))?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_project_id() {
        let id1 = generate_project_id();
        let id2 = generate_project_id();

        // IDs are 8 characters long (cloud-compatible format)
        assert_eq!(id1.len(), 8);
        assert_eq!(id2.len(), 8);
        assert_ne!(id1, id2);

        // Should be alphanumeric characters only
        assert!(id1.chars().all(|c| c.is_ascii_alphanumeric()));
        assert!(id2.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[tokio::test]
    async fn test_path_exists() {
        assert!(!path_exists("/nonexistent/path").await);
        assert!(path_exists("/tmp").await);
    }

    #[test]
    fn test_compress_decompress() {
        // Use a longer, more repetitive message for effective compression
        let original = b"Hello, world! This is a test message. Hello, world! This is a test message. Hello, world! This is a test message. Hello, world! This is a test message.";
        let compressed = compress_data(original).unwrap();
        let decompressed = decompress_data(&compressed).unwrap();

        assert_eq!(original.as_slice(), decompressed.as_slice());
        assert!(compressed.len() < original.len());
    }
}
