use colored::*;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tar::Archive;

const GITHUB_REPO: &str = "OrkeeAI/orkee";
const DASHBOARD_DIST_ASSET: &str = "orkee-dashboard-dist.tar.gz";
const DASHBOARD_SOURCE_ASSET: &str = "orkee-dashboard-source.tar.gz";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DashboardMode {
    Dist,   // Pre-built production files
    Source, // Source files requiring build
}

/// Get the path where dashboard assets should be stored
fn get_dashboard_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or_else(|| {
        "Could not determine home directory. \
        Please ensure the HOME environment variable is set."
    })?;
    Ok(home.join(".orkee").join("dashboard"))
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
    let client = reqwest::Client::builder().user_agent("orkee-cli").build()?;

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

    // Perform download and extraction with guaranteed cleanup
    let extraction_result = async {
        let mut file = fs::File::create(&temp_file)?;

        let bytes = response.bytes().await?;
        pb.inc(bytes.len() as u64);
        file.write_all(&bytes)?;
        drop(file); // Ensure file handle is closed before extraction

        pb.finish_with_message("Download complete");

        // Extract the archive
        println!("{} Extracting dashboard source...", "📂".cyan());

        // Clear existing dashboard files (except .version and node_modules)
        if dashboard_dir.exists() {
            for entry in fs::read_dir(&dashboard_dir)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    if filename != ".version"
                        && filename != "dashboard.tar.gz.tmp"
                        && filename != "node_modules"
                    {
                        if path.is_dir() {
                            fs::remove_dir_all(&path)?;
                        } else {
                            fs::remove_file(&path)?;
                        }
                    }
                }
            }
        }

        // Extract tar.gz
        let tar_gz = fs::File::open(&temp_file)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(&dashboard_dir)?;

        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;

    // Always attempt to cleanup temp file, regardless of success or failure
    if temp_file.exists() {
        if let Err(e) = fs::remove_file(&temp_file) {
            eprintln!(
                "⚠️  Warning: Failed to remove temporary file {}: {}",
                temp_file.display(),
                e
            );
        }
    }

    // Propagate any errors from extraction
    extraction_result?;

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
