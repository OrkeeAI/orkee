use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Copy the orkee CLI binary to the binaries folder before building
    if let Err(e) = copy_orkee_binary() {
        eprintln!("cargo:warning={}", e);
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

    // Get the Rust target triple from CARGO_CFG_TARGET
    // This gives us the full triple like "x86_64-unknown-linux-gnu"
    let target_triple = env::var("TARGET").unwrap_or_else(|_| {
        // Fallback: construct from CARGO_CFG_TARGET_ARCH and CARGO_CFG_TARGET_OS
        let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| {
            if cfg!(target_arch = "aarch64") {
                "aarch64".to_string()
            } else if cfg!(target_arch = "x86_64") {
                "x86_64".to_string()
            } else {
                panic!("Unsupported architecture");
            }
        });

        let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| {
            if cfg!(target_os = "macos") {
                "macos".to_string()
            } else if cfg!(target_os = "linux") {
                "linux".to_string()
            } else if cfg!(target_os = "windows") {
                "windows".to_string()
            } else {
                panic!("Unsupported OS");
            }
        });

        // Map OS to full target triple suffix
        let suffix = match os.as_str() {
            "macos" => "apple-darwin",
            "linux" => "unknown-linux-gnu",
            "windows" => "pc-windows-msvc",
            _ => panic!("Unsupported OS: {}", os),
        };

        format!("{}-{}", arch, suffix)
    });

    // Check if binary already exists in binaries/ directory
    // (prepare-binaries.sh may have already placed it there)
    let binary_name = format!("orkee-{}", target_triple);
    let binaries_dir = manifest_dir.join("binaries");
    let dest = binaries_dir.join(&binary_name);

    if dest.exists() {
        println!("cargo:warning=Orkee binary already exists at {}, skipping copy", dest.display());
        println!("cargo:rerun-if-changed={}", dest.display());
        return Ok(());
    }

    // Try target-specific directory first (cross-compile), then fall back to default
    let mut source = workspace_root
        .join("target")
        .join(&target_triple)
        .join("release")
        .join("orkee");

    // If not in target-specific directory, check default location
    if !source.exists() {
        source = workspace_root.join("target").join("release").join("orkee");
    }

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
