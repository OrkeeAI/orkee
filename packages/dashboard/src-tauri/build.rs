use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Copy the orkee CLI binary to the binaries folder before building
    copy_orkee_binary();

    tauri_build::build()
}

fn copy_orkee_binary() {
    // Get the workspace root (3 levels up from src-tauri/)
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("Failed to find workspace root");

    // Determine the target architecture
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| {
        // Fallback to current platform
        if cfg!(target_arch = "aarch64") {
            "aarch64".to_string()
        } else if cfg!(target_arch = "x86_64") {
            "x86_64".to_string()
        } else {
            panic!("Unsupported architecture");
        }
    });

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| {
        if cfg!(target_os = "macos") {
            "apple-darwin".to_string()
        } else if cfg!(target_os = "linux") {
            "unknown-linux-gnu".to_string()
        } else if cfg!(target_os = "windows") {
            "pc-windows-msvc".to_string()
        } else {
            panic!("Unsupported OS");
        }
    });

    // Source: workspace target/release/orkee
    let source = workspace_root.join("target").join("release").join("orkee");

    // Destination: src-tauri/binaries/orkee-{arch}-{os}
    let binary_name = format!("orkee-{}-{}", target_arch, target_os);
    let dest = manifest_dir.join("binaries").join(&binary_name);

    // Only copy if source exists
    if source.exists() {
        println!("cargo:warning=Copying orkee binary from {:?} to {:?}", source, dest);
        fs::copy(&source, &dest).expect("Failed to copy orkee binary");

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dest).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest, perms).unwrap();
        }
    } else {
        println!("cargo:warning=Orkee binary not found at {:?} - skipping copy. Build the CLI first with: cargo build --release --package orkee-cli", source);
    }

    // Tell cargo to rerun if the orkee binary changes
    println!("cargo:rerun-if-changed={}", source.display());
}
