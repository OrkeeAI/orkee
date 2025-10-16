// ABOUTME: Build script for embedding environment variables at compile time
// ABOUTME: Captures POSTHOG_API_KEY during build for secure key management

fn main() {
    // Capture POSTHOG_API_KEY at build time if available
    // This allows the key to be embedded in the binary during CI/CD builds
    // while still allowing runtime override via environment variable
    if let Ok(key) = std::env::var("POSTHOG_API_KEY") {
        println!("cargo:rustc-env=POSTHOG_API_KEY={}", key);
    }

    // Re-run build script if POSTHOG_API_KEY changes
    println!("cargo:rerun-if-env-changed=POSTHOG_API_KEY");
}
