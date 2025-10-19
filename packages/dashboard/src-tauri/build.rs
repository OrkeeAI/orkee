use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Copy the orkee CLI binary to the binaries folder before building
    if let Err(e) = copy_orkee_binary() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    tauri_build::build()
}

fn copy_orkee_binary() -> Result<(), String> {
    // Get the workspace root (3 levels up from src-tauri/)
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| "CARGO_MANIFEST_DIR not set (this should never happen)".to_string())?,
    );

    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or_else(|| {
            format!(
                "Failed to find workspace root from manifest directory: {}",
                manifest_dir.display()
            )
        })?;

    // Determine the target architecture
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| {
        // Fallback to current platform
        if cfg!(target_arch = "aarch64") {
            "aarch64".to_string()
        } else if cfg!(target_arch = "x86_64") {
            "x86_64".to_string()
        } else {
            panic!("Unsupported architecture - please file a bug report");
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
            panic!("Unsupported OS - please file a bug report");
        }
    });

    // Source: workspace target/release/orkee
    let source = workspace_root.join("target").join("release").join("orkee");

    // Check if source exists
    if !source.exists() {
        return Err(format!(
            "Orkee CLI binary not found at: {}\n\
             \n\
             Please build the CLI first:\n\
             \n\
             From workspace root:\n\
               cargo build --release --package orkee-cli\n\
             \n\
             Or use the helper script:\n\
               cd packages/dashboard && ./rebuild-desktop.sh",
            source.display()
        ));
    }

    // Destination: src-tauri/binaries/orkee-{arch}-{os}
    let binary_name = format!("orkee-{}-{}", target_arch, target_os);
    let binaries_dir = manifest_dir.join("binaries");
    let dest = binaries_dir.join(&binary_name);

    // Create binaries directory if it doesn't exist
    if !binaries_dir.exists() {
        fs::create_dir_all(&binaries_dir).map_err(|e| {
            format!(
                "Failed to create binaries directory at {}: {}",
                binaries_dir.display(),
                e
            )
        })?;
    }

    // Copy the binary
    println!(
        "cargo:warning=Copying orkee binary from {} to {}",
        source.display(),
        dest.display()
    );
    fs::copy(&source, &dest).map_err(|e| {
        format!(
            "Failed to copy orkee binary from {} to {}: {}",
            source.display(),
            dest.display(),
            e
        )
    })?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest)
            .map_err(|e| format!("Failed to read permissions for {}: {}", dest.display(), e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest, perms).map_err(|e| {
            format!(
                "Failed to set executable permissions on {}: {}",
                dest.display(),
                e
            )
        })?;
    }

    // Tell cargo to rerun if the orkee binary changes
    println!("cargo:rerun-if-changed={}", source.display());

    Ok(())
}
