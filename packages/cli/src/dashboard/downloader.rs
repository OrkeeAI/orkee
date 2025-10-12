use colored::*;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tar::Archive;

const GITHUB_REPO: &str = "OrkeeAI/orkee";
const DASHBOARD_DIST_ASSET: &str = "orkee-dashboard-dist.tar.gz";
const DASHBOARD_SOURCE_ASSET: &str = "orkee-dashboard-source.tar.gz";

// Security limits for archive extraction
const MAX_FILE_SIZE_BYTES: u64 = 100 * 1024 * 1024; // 100 MB per file
const MAX_TOTAL_SIZE_BYTES: u64 = 500 * 1024 * 1024; // 500 MB total
const MAX_NESTING_DEPTH: usize = 20; // Maximum directory nesting depth

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DashboardMode {
    Dist,   // Pre-built production files
    Source, // Source files requiring build
}

/// Validate that a path is safe and doesn't attempt path traversal
fn validate_safe_path(path: &Path, base_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Canonicalize both paths to resolve any symlinks or relative components
    let canonical_base = base_dir.canonicalize().or_else(|_| {
        // If base doesn't exist yet, just use it as-is
        Ok::<PathBuf, std::io::Error>(base_dir.to_path_buf())
    })?;

    let canonical_path = if path.exists() {
        path.canonicalize()?
    } else {
        // For non-existent paths, construct them relative to canonical_base
        if let Ok(relative) = path.strip_prefix(base_dir) {
            // Path is under base_dir (before canonicalization) - construct relative to canonical base
            canonical_base.join(relative)
        } else if path.is_absolute() {
            // Absolute path not under base_dir - use as-is for validation
            path.to_path_buf()
        } else {
            // Relative path - join with canonical base
            canonical_base.join(path)
        }
    };

    // Check that the path is within the base directory
    if !canonical_path.starts_with(&canonical_base) {
        return Err(format!(
            "Path traversal detected: {} is outside {}",
            canonical_path.display(),
            canonical_base.display()
        )
        .into());
    }

    // Check for dangerous path components
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                return Err("Path contains parent directory (..) component".into());
            }
            std::path::Component::Normal(name) => {
                let name_str = name.to_string_lossy();
                // Block paths that look suspicious
                if name_str.contains("..")
                    || name_str.starts_with('.')
                        && name_str.len() > 1
                        && name_str.chars().nth(1) == Some('.')
                {
                    return Err(format!("Suspicious path component: {}", name_str).into());
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Validate that a file size doesn't exceed the maximum allowed size
fn validate_file_size(size: u64, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if size > MAX_FILE_SIZE_BYTES {
        return Err(format!(
            "File size {} bytes exceeds maximum allowed size of {} bytes for: {}",
            size,
            MAX_FILE_SIZE_BYTES,
            path.display()
        )
        .into());
    }
    Ok(())
}

/// Validate that total extracted size doesn't exceed limits
fn validate_total_size(
    current_total: u64,
    new_size: u64,
) -> Result<u64, Box<dyn std::error::Error>> {
    let new_total = current_total
        .checked_add(new_size)
        .ok_or("Total size overflow during extraction")?;

    if new_total > MAX_TOTAL_SIZE_BYTES {
        return Err(format!(
            "Total extraction size {} bytes exceeds maximum allowed size of {} bytes",
            new_total, MAX_TOTAL_SIZE_BYTES
        )
        .into());
    }

    Ok(new_total)
}

/// Validate that path nesting depth doesn't exceed maximum
fn validate_nesting_depth(path: &Path, base_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let relative_path = path.strip_prefix(base_dir).unwrap_or(path);

    let depth = relative_path.components().count();

    if depth > MAX_NESTING_DEPTH {
        return Err(format!(
            "Path nesting depth {} exceeds maximum allowed depth of {} for: {}",
            depth,
            MAX_NESTING_DEPTH,
            path.display()
        )
        .into());
    }

    Ok(())
}

/// Validate symlinks after extraction to ensure they don't point outside base directory
fn validate_symlink(
    symlink_path: &Path,
    base_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !symlink_path.is_symlink() {
        return Ok(());
    }

    // Read the symlink target
    let target = fs::read_link(symlink_path)?;

    // Resolve the absolute path of the symlink target
    let absolute_target = if target.is_absolute() {
        target.clone()
    } else {
        // For relative symlinks, resolve from the symlink's parent directory
        if let Some(parent) = symlink_path.parent() {
            parent.join(&target)
        } else {
            target.clone()
        }
    };

    // Canonicalize to resolve any .. or . components
    let canonical_target = if absolute_target.exists() {
        absolute_target.canonicalize()?
    } else {
        // For non-existent targets, manually resolve the path
        let canonical_base = base_dir.canonicalize()?;
        if let Ok(relative) = absolute_target.strip_prefix(base_dir) {
            canonical_base.join(relative)
        } else {
            absolute_target
        }
    };

    let canonical_base = base_dir.canonicalize()?;

    // Check if the symlink target is within the base directory
    if !canonical_target.starts_with(&canonical_base) {
        return Err(format!(
            "Symlink points outside base directory: {} -> {}",
            symlink_path.display(),
            canonical_target.display()
        )
        .into());
    }

    Ok(())
}

/// Get the path where dashboard assets should be stored
fn get_dashboard_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or(
        "Could not determine home directory. \
        Please ensure the HOME environment variable is set."
    )?;
    let dashboard_dir = home.join(".orkee").join("dashboard");

    // Validate the dashboard directory path itself
    let orkee_dir = home.join(".orkee");
    if !orkee_dir.exists() {
        fs::create_dir_all(&orkee_dir)?;
    }

    // Ensure the dashboard directory is within .orkee
    let canonical_orkee = orkee_dir.canonicalize()?;
    let dashboard_canonical = if dashboard_dir.exists() {
        dashboard_dir.canonicalize()?
    } else {
        canonical_orkee.join("dashboard")
    };

    if !dashboard_canonical.starts_with(&canonical_orkee) {
        return Err("Dashboard directory path validation failed".into());
    }

    Ok(dashboard_dir)
}

/// Check if dashboard assets are already downloaded and match the current version
pub fn is_dashboard_installed(mode: DashboardMode) -> Result<bool, Box<dyn std::error::Error>> {
    let dashboard_dir = get_dashboard_dir()?;
    let version_file = dashboard_dir.join(".version");
    let mode_file = dashboard_dir.join(".mode");

    if !dashboard_dir.exists() || !version_file.exists() {
        return Ok(false);
    }

    // Check if the mode matches
    if mode_file.exists() {
        if let Ok(installed_mode) = fs::read_to_string(&mode_file) {
            let installed_mode = installed_mode.trim();
            let expected_mode = match mode {
                DashboardMode::Dist => "dist",
                DashboardMode::Source => "source",
            };

            if installed_mode != expected_mode {
                println!(
                    "{} Dashboard mode mismatch (installed: {}, expected: {})",
                    "⚠️".yellow(),
                    installed_mode,
                    expected_mode
                );
                return Ok(false);
            }
        }
    }

    // Read the installed version
    if let Ok(installed_version) = fs::read_to_string(&version_file) {
        let installed_version = installed_version.trim();
        let current_version = env!("CARGO_PKG_VERSION");

        if installed_version == current_version {
            return Ok(true);
        } else {
            println!(
                "{} Dashboard version mismatch (installed: {}, current: {})",
                "⚠️".yellow(),
                installed_version,
                current_version
            );
        }
    }

    Ok(false)
}

/// Install dependencies in the dashboard directory
fn install_dependencies(dashboard_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Installing dashboard dependencies...", "📦".cyan());

    let bun_check = std::process::Command::new("which").arg("bun").output();

    if bun_check.is_ok() && bun_check.unwrap().status.success() {
        // Use production flag if not in dev mode
        let install_args = if std::env::var("ORKEE_DEV_MODE").is_err() {
            vec!["install", "--production"]
        } else {
            vec!["install"]
        };

        let install_result = std::process::Command::new("bun")
            .args(&install_args)
            .current_dir(dashboard_dir)
            .status();

        match install_result {
            Ok(status) if status.success() => {
                println!("{} Dependencies installed successfully!", "✅".green());
                Ok(())
            }
            _ => Err(format!(
                "❌ Failed to install dashboard dependencies\n\n\
                The 'bun install' command failed in: {}\n\n\
                Troubleshooting:\n\
                  1. Try running manually: cd {} && bun install\n\
                  2. Check if bun is working: bun --version\n\
                  3. Clear cache and retry: rm -rf node_modules package-lock.json && bun install\n\
                  4. Check bun logs for specific errors\n\n\
                For production use, consider using the --dev=false flag to download pre-built assets",
                dashboard_dir.display(),
                dashboard_dir.display()
            )
            .into()),
        }
    } else {
        Err(format!(
            "❌ bun package manager not found\n\n\
            Dashboard dependencies require bun to be installed.\n\n\
            Installation:\n\
              • macOS/Linux: curl -fsSL https://bun.sh/install | bash\n\
              • Or visit: https://bun.sh/docs/installation\n\n\
            After installing bun:\n\
              1. Restart your terminal\n\
              2. Verify installation: bun --version\n\
              3. Run manually: cd {} && bun install\n\n\
            Alternative:\n\
              • Use --dev=false flag to download pre-built dashboard (no bun required)",
            dashboard_dir.display()
        )
        .into())
    }
}

/// Download and extract dashboard from GitHub releases
pub async fn download_dashboard(
    mode: DashboardMode,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dashboard_dir = get_dashboard_dir()?;
    let version = env!("CARGO_PKG_VERSION");

    let asset_name = match mode {
        DashboardMode::Dist => DASHBOARD_DIST_ASSET,
        DashboardMode::Source => DASHBOARD_SOURCE_ASSET,
    };

    let mode_name = match mode {
        DashboardMode::Dist => "pre-built dashboard",
        DashboardMode::Source => "dashboard source",
    };

    println!("{}", format!("📦 {} not found locally", mode_name).yellow());
    println!("{} Downloading {} v{}...", "⬇️".cyan(), mode_name, version);

    // Create dashboard directory if it doesn't exist
    fs::create_dir_all(&dashboard_dir)?;

    // Construct download URL
    let download_url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        GITHUB_REPO, version, asset_name
    );

    // Create progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Connecting to GitHub...");

    // Download the file
    let client = reqwest::Client::builder()
        .user_agent("orkee-cli")
        .timeout(Duration::from_secs(300))
        .connect_timeout(Duration::from_secs(30))
        .build()?;

    let response = client.get(&download_url).send().await?;

    if !response.status().is_success() {
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(format!(
                "❌ Dashboard package not found for version {} ({})\n\n\
                Possible causes:\n\
                  • This is a development version without published release assets\n\
                  • The GitHub release doesn't include the {} asset\n\
                  • Network issues prevented finding the release\n\n\
                Troubleshooting:\n\
                  1. Check if release exists: https://github.com/{}/releases/tag/v{}\n\
                  2. For local development, ensure dashboard is in packages/dashboard/\n\
                  3. Try running with --dev flag to use local dashboard\n\
                  4. If building from source, run 'cd packages/dashboard && bun install && bun build'\n\n\
                Download URL attempted: {}",
                version,
                response.status(),
                asset_name,
                GITHUB_REPO,
                version,
                download_url
            )
            .into());
        }

        return Err(format!(
            "❌ Failed to download dashboard from GitHub (HTTP {})\n\n\
            Possible causes:\n\
              • Network connectivity issues\n\
              • GitHub API rate limiting\n\
              • Temporary GitHub service issues\n\n\
            Troubleshooting:\n\
              1. Check your internet connection\n\
              2. Try again in a few minutes\n\
              3. Check GitHub status: https://www.githubstatus.com/\n\
              4. For local development, use --dev flag to bypass download\n\n\
            Download URL: {}",
            response.status(),
            download_url
        )
        .into());
    }

    // Get content length for progress bar
    let total_size = response.content_length().unwrap_or(0);

    if total_size > 0 {
        pb.set_length(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-")
        );
    }

    // Download to temporary file
    let temp_file = dashboard_dir.join("dashboard.tar.gz.tmp");

    // Backup old version and mode files for rollback
    let version_file = dashboard_dir.join(".version");
    let mode_file = dashboard_dir.join(".mode");
    let backup_version = dashboard_dir.join(".version.backup");
    let backup_mode = dashboard_dir.join(".mode.backup");

    // Backup existing files if they exist
    if version_file.exists() {
        let _ = fs::copy(&version_file, &backup_version);
    }
    if mode_file.exists() {
        let _ = fs::copy(&mode_file, &backup_mode);
    }

    // Perform download and extraction with guaranteed cleanup and rollback
    let extraction_result = async {
        let mut file = fs::File::create(&temp_file)?;

        let bytes = response.bytes().await?;
        pb.inc(bytes.len() as u64);
        file.write_all(&bytes)?;
        drop(file); // Ensure file handle is closed before extraction

        pb.finish_with_message("Download complete");

        // Extract the archive
        println!("{} Extracting dashboard source...", "📂".cyan());

        // Extract to a staging directory first to avoid corrupting existing install
        let staging_dir = dashboard_dir.join(".staging");
        if staging_dir.exists() {
            fs::remove_dir_all(&staging_dir)?;
        }
        fs::create_dir_all(&staging_dir)?;

        // Extract tar.gz to staging with security validations
        let tar_gz = fs::File::open(&temp_file)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        // Track total extracted size to prevent disk exhaustion
        let mut total_extracted_size: u64 = 0;

        // Extract each entry with validation
        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let entry_path = entry.path()?;
            let dest_path = staging_dir.join(&entry_path);

            // Validate path safety
            validate_safe_path(&dest_path, &staging_dir)?;

            // Validate nesting depth
            validate_nesting_depth(&dest_path, &staging_dir)?;

            // Validate file size
            let entry_size = entry.size();
            validate_file_size(entry_size, &dest_path)?;

            // Track and validate total size
            total_extracted_size = validate_total_size(total_extracted_size, entry_size)?;

            // Extract the entry
            entry.unpack(&dest_path)?;

            // Validate symlinks after extraction
            validate_symlink(&dest_path, &staging_dir)?;
        }

        // Extraction successful, now replace old files with new ones
        // Clear existing dashboard files (except .version, .mode, node_modules, backups, staging)
        if dashboard_dir.exists() {
            for entry in fs::read_dir(&dashboard_dir)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if !filename_str.starts_with('.') && filename != "node_modules" {
                        if path.is_dir() {
                            fs::remove_dir_all(&path)?;
                        } else {
                            fs::remove_file(&path)?;
                        }
                    }
                }
            }
        }

        // Move files from staging to dashboard directory with path validation
        for entry in fs::read_dir(&staging_dir)? {
            let entry = entry?;
            let source = entry.path();
            if let Some(filename) = source.file_name() {
                let dest = dashboard_dir.join(filename);

                // Validate destination path to prevent path traversal
                validate_safe_path(&dest, &dashboard_dir)?;

                // Remove destination if it exists (except node_modules)
                if dest.exists() && filename != "node_modules" {
                    if dest.is_dir() {
                        fs::remove_dir_all(&dest)?;
                    } else {
                        fs::remove_file(&dest)?;
                    }
                }
                fs::rename(&source, &dest)?;
            }
        }

        // Clean up staging directory
        if staging_dir.exists() {
            let _ = fs::remove_dir_all(&staging_dir);
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;

    // Cleanup: Always remove temp file and backups
    if temp_file.exists() {
        if let Err(e) = fs::remove_file(&temp_file) {
            eprintln!(
                "⚠️  Warning: Failed to remove temporary file {}: {}",
                temp_file.display(),
                e
            );
        }
    }

    // Handle rollback on failure
    if extraction_result.is_err() {
        eprintln!(
            "{} Download/extraction failed, rolling back...",
            "⚠️".yellow()
        );

        // Restore backup files
        if backup_version.exists() {
            if let Err(e) = fs::rename(&backup_version, &version_file) {
                eprintln!("⚠️  Warning: Failed to restore version backup: {}", e);
            } else {
                eprintln!("✓ Restored version file from backup");
            }
        }
        if backup_mode.exists() {
            if let Err(e) = fs::rename(&backup_mode, &mode_file) {
                eprintln!("⚠️  Warning: Failed to restore mode backup: {}", e);
            } else {
                eprintln!("✓ Restored mode file from backup");
            }
        }

        // Clean up staging directory if it exists
        let staging_dir = dashboard_dir.join(".staging");
        if staging_dir.exists() {
            let _ = fs::remove_dir_all(&staging_dir);
        }

        // Propagate the error
        extraction_result?;
        return Err("Download/extraction failed and rollback completed".into());
    }

    // Success - clean up backup files
    if backup_version.exists() {
        let _ = fs::remove_file(&backup_version);
    }
    if backup_mode.exists() {
        let _ = fs::remove_file(&backup_mode);
    }

    // Write version file
    let version_file = dashboard_dir.join(".version");
    fs::write(&version_file, version)?;

    // Write mode file
    let mode_file = dashboard_dir.join(".mode");
    let mode_str = match mode {
        DashboardMode::Dist => "dist",
        DashboardMode::Source => "source",
    };
    fs::write(&mode_file, mode_str)?;

    // Install dependencies only for source mode
    if mode == DashboardMode::Source {
        if let Err(e) = install_dependencies(&dashboard_dir) {
            println!("{} {}", "⚠️".yellow(), e);
        }
    }

    let success_msg = match mode {
        DashboardMode::Dist => "Pre-built dashboard installed successfully!",
        DashboardMode::Source => "Dashboard source installed successfully!",
    };
    println!("{} {}", "✅".green(), success_msg);

    Ok(dashboard_dir)
}

/// Ensure dashboard is installed, downloading if necessary
pub async fn ensure_dashboard(
    dev_mode: bool,
) -> Result<(PathBuf, DashboardMode), Box<dyn std::error::Error>> {
    let dashboard_dir = get_dashboard_dir()?;

    // Determine which mode to use
    let mode = if dev_mode || std::env::var("ORKEE_DEV_MODE").is_ok() {
        DashboardMode::Source
    } else {
        DashboardMode::Dist
    };

    if is_dashboard_installed(mode)? {
        // For source mode, check if node_modules exists
        if mode == DashboardMode::Source {
            let node_modules = dashboard_dir.join("node_modules");
            if !node_modules.exists() {
                println!(
                    "{} Dashboard found but dependencies missing, installing...",
                    "📦".yellow()
                );
                install_dependencies(&dashboard_dir)?;
            }
        }

        let mode_name = match mode {
            DashboardMode::Dist => "pre-built dashboard",
            DashboardMode::Source => "dashboard source",
        };

        println!(
            "{} Using cached {} from {}",
            "📂".cyan(),
            mode_name,
            dashboard_dir.display()
        );
        Ok((dashboard_dir, mode))
    } else {
        // Try to download the requested mode first
        match download_dashboard(mode).await {
            Ok(path) => Ok((path, mode)),
            Err(e) if mode == DashboardMode::Dist => {
                // If dist download fails, fallback to source
                println!(
                    "\n{} Pre-built dashboard not available, attempting fallback to source...",
                    "⚠️".yellow()
                );
                println!("{} Original error: {}", "ℹ️".blue(), e);
                println!(
                    "{} This fallback requires bun to be installed\n",
                    "ℹ️".blue()
                );

                match download_dashboard(DashboardMode::Source).await {
                    Ok(path) => {
                        println!(
                            "{} Successfully fell back to dashboard source mode",
                            "✅".green()
                        );
                        Ok((path, DashboardMode::Source))
                    }
                    Err(fallback_error) => Err(format!(
                        "❌ Failed to download dashboard in both modes\n\n\
                            Pre-built download error:\n{}\n\n\
                            Source fallback error:\n{}\n\n\
                            Troubleshooting:\n\
                              1. Check your internet connection\n\
                              2. Verify the release exists: https://github.com/{}/releases\n\
                              3. For local development, use --dev flag to use packages/dashboard/\n\
                              4. Try running from source: git clone the repo and run locally\n\
                              5. Check GitHub status: https://www.githubstatus.com/",
                        e, fallback_error, GITHUB_REPO
                    )
                    .into()),
                }
            }
            Err(e) => Err(e),
        }
    }
}

/// Get the path to run dashboard dev server from
#[allow(dead_code)]
pub fn get_dashboard_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    get_dashboard_dir()
}

/// Clean up downloaded dashboard assets
#[allow(dead_code)]
pub fn clean_dashboard() -> Result<(), Box<dyn std::error::Error>> {
    let dashboard_dir = get_dashboard_dir()?;
    if dashboard_dir.exists() {
        fs::remove_dir_all(&dashboard_dir)?;
        println!("{} Dashboard cache cleaned", "🧹".green());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_safe_path_allows_safe_paths() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Safe path within base
        let safe_path = base.join("safe_file.txt");
        assert!(validate_safe_path(&safe_path, base).is_ok());

        // Safe nested path
        let safe_nested = base.join("subdir").join("file.txt");
        assert!(validate_safe_path(&safe_nested, base).is_ok());
    }

    #[test]
    fn test_validate_safe_path_blocks_parent_directory_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Path with .. component
        let traversal_path = base.join("..").join("etc").join("passwd");
        assert!(validate_safe_path(&traversal_path, base).is_err());
    }

    #[test]
    fn test_validate_safe_path_blocks_suspicious_components() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Path with suspicious .. in filename
        let suspicious = base.join("file..txt");
        assert!(validate_safe_path(&suspicious, base).is_err());

        // Path starting with ..
        let dot_dot = base.join("..hidden");
        assert!(validate_safe_path(&dot_dot, base).is_err());
    }

    #[test]
    fn test_validate_safe_path_allows_single_dot_files() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Hidden files with single dot are okay
        let hidden_file = base.join(".gitignore");
        assert!(validate_safe_path(&hidden_file, base).is_ok());
    }

    #[test]
    fn test_validate_safe_path_with_existing_directories() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create actual directories
        let subdir = base.join("subdir");
        fs::create_dir(&subdir).unwrap();

        let file_in_subdir = subdir.join("file.txt");
        assert!(validate_safe_path(&file_in_subdir, base).is_ok());
    }

    #[test]
    fn test_validate_safe_path_canonical_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create a subdirectory
        let subdir = base.join("subdir");
        fs::create_dir(&subdir).unwrap();

        // Path with redundant components (should be resolved)
        let redundant = base.join("subdir").join(".").join("file.txt");
        assert!(validate_safe_path(&redundant, base).is_ok());
    }

    #[test]
    fn test_validate_safe_path_blocks_absolute_paths_outside_base() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Absolute path outside base directory
        let outside_path = Path::new("/etc/passwd");
        assert!(validate_safe_path(outside_path, base).is_err());
    }

    #[test]
    fn test_get_dashboard_dir_returns_valid_path() {
        let result = get_dashboard_dir();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_str().unwrap().contains(".orkee"));
        assert!(path.to_str().unwrap().contains("dashboard"));
    }

    #[test]
    fn test_dashboard_mode_equality() {
        assert_eq!(DashboardMode::Dist, DashboardMode::Dist);
        assert_eq!(DashboardMode::Source, DashboardMode::Source);
        assert_ne!(DashboardMode::Dist, DashboardMode::Source);
    }

    #[test]
    fn test_is_dashboard_installed_checks_version_and_mode() {
        // Test that the function checks both version and mode files
        // We can't easily mock HOME, so we test the logic by creating files
        // in the actual ~/.orkee/dashboard directory and cleaning up after

        let dashboard_dir = match get_dashboard_dir() {
            Ok(dir) => dir,
            Err(_) => return, // Skip test if we can't get dashboard dir
        };

        // Save original state
        let version_file = dashboard_dir.join(".version");
        let mode_file = dashboard_dir.join(".mode");
        let backup_version = dashboard_dir.join(".version.test_backup");
        let backup_mode = dashboard_dir.join(".mode.test_backup");

        // Backup existing files if they exist
        if version_file.exists() {
            let _ = fs::copy(&version_file, &backup_version);
        }
        if mode_file.exists() {
            let _ = fs::copy(&mode_file, &backup_mode);
        }

        // Ensure dashboard directory exists
        let _ = fs::create_dir_all(&dashboard_dir);

        // Test 1: No version file - should return false
        let _ = fs::remove_file(&version_file);
        let _ = fs::remove_file(&mode_file);
        let result = is_dashboard_installed(DashboardMode::Dist);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);

        // Test 2: Wrong version - should return false
        fs::write(&version_file, "0.0.0").unwrap();
        fs::write(&mode_file, "dist").unwrap();
        let result = is_dashboard_installed(DashboardMode::Dist);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);

        // Test 3: Correct version, wrong mode - should return false
        let current_version = env!("CARGO_PKG_VERSION");
        fs::write(&version_file, current_version).unwrap();
        fs::write(&mode_file, "source").unwrap();
        let result = is_dashboard_installed(DashboardMode::Dist);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);

        // Test 4: Correct version and mode - should return true
        fs::write(&version_file, current_version).unwrap();
        fs::write(&mode_file, "dist").unwrap();
        let result = is_dashboard_installed(DashboardMode::Dist);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // Test 5: Correct version, no mode file (backwards compat) - should return true
        fs::write(&version_file, current_version).unwrap();
        let _ = fs::remove_file(&mode_file);
        let result = is_dashboard_installed(DashboardMode::Dist);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // Cleanup: restore original state
        if backup_version.exists() {
            let _ = fs::rename(&backup_version, &version_file);
        } else {
            let _ = fs::remove_file(&version_file);
        }

        if backup_mode.exists() {
            let _ = fs::rename(&backup_mode, &mode_file);
        } else {
            let _ = fs::remove_file(&mode_file);
        }
    }

    #[test]
    fn test_validate_file_size_accepts_small_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("small.txt");

        // Small file should be accepted
        assert!(validate_file_size(1024, &file_path).is_ok());
        assert!(validate_file_size(1024 * 1024, &file_path).is_ok());
    }

    #[test]
    fn test_validate_file_size_rejects_large_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");

        // File larger than MAX_FILE_SIZE_BYTES should be rejected
        let too_large = MAX_FILE_SIZE_BYTES + 1;
        assert!(validate_file_size(too_large, &file_path).is_err());
    }

    #[test]
    fn test_validate_total_size_tracks_accumulation() {
        // Should accept sizes within limit
        let result = validate_total_size(0, 1024);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1024);

        // Should accumulate correctly
        let result = validate_total_size(1024, 2048);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3072);
    }

    #[test]
    fn test_validate_total_size_rejects_excessive_total() {
        // Should reject when total exceeds MAX_TOTAL_SIZE_BYTES
        let result = validate_total_size(MAX_TOTAL_SIZE_BYTES - 100, 200);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_nesting_depth_accepts_shallow_paths() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Shallow paths should be accepted
        let shallow = base.join("file.txt");
        assert!(validate_nesting_depth(&shallow, base).is_ok());

        let medium = base.join("a").join("b").join("c").join("file.txt");
        assert!(validate_nesting_depth(&medium, base).is_ok());
    }

    #[test]
    fn test_validate_nesting_depth_rejects_deep_paths() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Build a path deeper than MAX_NESTING_DEPTH
        let mut deep_path = base.to_path_buf();
        for i in 0..MAX_NESTING_DEPTH + 1 {
            deep_path = deep_path.join(format!("dir{}", i));
        }

        assert!(validate_nesting_depth(&deep_path, base).is_err());
    }

    #[test]
    fn test_validate_symlink_accepts_internal_links() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create a file and symlink pointing to it
        let target_file = base.join("target.txt");
        fs::write(&target_file, "content").unwrap();

        let symlink = base.join("link.txt");

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target_file, &symlink).unwrap();
            assert!(validate_symlink(&symlink, base).is_ok());
        }
    }

    #[test]
    fn test_validate_symlink_rejects_external_links() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create symlink pointing outside base directory
        let symlink = base.join("evil_link.txt");
        let external_target = Path::new("/etc/passwd");

        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink(external_target, &symlink);
            if symlink.exists() {
                assert!(validate_symlink(&symlink, base).is_err());
            }
        }
    }

    #[test]
    fn test_validate_symlink_accepts_non_symlinks() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Regular files should pass validation
        let regular_file = base.join("regular.txt");
        fs::write(&regular_file, "content").unwrap();

        assert!(validate_symlink(&regular_file, base).is_ok());
    }
}
