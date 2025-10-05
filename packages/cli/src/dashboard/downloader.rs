use colored::*;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tar::Archive;

const GITHUB_REPO: &str = "OrkeeAI/orkee";
const DASHBOARD_ASSET_NAME: &str = "orkee-dashboard-source.tar.gz";

/// Get the path where dashboard assets should be stored
fn get_dashboard_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".orkee").join("dashboard")
}

/// Check if dashboard assets are already downloaded and match the current version
pub fn is_dashboard_installed() -> bool {
    let dashboard_dir = get_dashboard_dir();
    let version_file = dashboard_dir.join(".version");

    if !dashboard_dir.exists() || !version_file.exists() {
        return false;
    }

    // Read the installed version
    if let Ok(installed_version) = fs::read_to_string(&version_file) {
        let installed_version = installed_version.trim();
        let current_version = env!("CARGO_PKG_VERSION");

        if installed_version == current_version {
            return true;
        } else {
            println!(
                "{} Dashboard version mismatch (installed: {}, current: {})",
                "⚠️".yellow(),
                installed_version,
                current_version
            );
        }
    }

    false
}

/// Install dependencies in the dashboard directory
fn install_dependencies(dashboard_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Installing dashboard dependencies...", "📦".cyan());

    let bun_check = std::process::Command::new("which").arg("bun").output();

    if bun_check.is_ok() && bun_check.unwrap().status.success() {
        let install_result = std::process::Command::new("bun")
            .args(["install"])
            .current_dir(dashboard_dir)
            .status();

        match install_result {
            Ok(status) if status.success() => {
                println!("{} Dependencies installed successfully!", "✅".green());
                Ok(())
            }
            _ => Err(format!(
                "Failed to install dependencies. Run 'bun install' manually in {}",
                dashboard_dir.display()
            )
            .into()),
        }
    } else {
        Err(format!(
            "bun not found. Install bun and run 'bun install' in {}",
            dashboard_dir.display()
        )
        .into())
    }
}

/// Download and extract dashboard source from GitHub releases
pub async fn download_dashboard() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dashboard_dir = get_dashboard_dir();
    let version = env!("CARGO_PKG_VERSION");

    println!("{}", "📦 Dashboard source not found locally".yellow());
    println!(
        "{} Downloading dashboard source v{}...",
        "⬇️".cyan(),
        version
    );

    // Create dashboard directory if it doesn't exist
    fs::create_dir_all(&dashboard_dir)?;

    // Construct download URL
    let download_url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        GITHUB_REPO, version, DASHBOARD_ASSET_NAME
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
                "Dashboard source package not found for version {}. \
                This might be a development version without published assets. \
                Please ensure you have the dashboard source in packages/dashboard",
                version
            )
            .into());
        }
        return Err(format!("Failed to download dashboard: {}", response.status()).into());
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
    let mut file = fs::File::create(&temp_file)?;

    let bytes = response.bytes().await?;
    pb.inc(bytes.len() as u64);
    file.write_all(&bytes)?;

    pb.finish_with_message("Download complete");

    // Extract the archive
    println!("{} Extracting dashboard source...", "📂".cyan());

    // Clear existing dashboard files (except .version and node_modules)
    if dashboard_dir.exists() {
        for entry in fs::read_dir(&dashboard_dir)? {
            let entry = entry?;
            let path = entry.path();
            let filename = path.file_name().unwrap();
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

    // Extract tar.gz
    let tar_gz = fs::File::open(&temp_file)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(&dashboard_dir)?;

    // Remove temporary file
    fs::remove_file(&temp_file)?;

    // Write version file
    let version_file = dashboard_dir.join(".version");
    fs::write(&version_file, version)?;

    // Install dependencies
    if let Err(e) = install_dependencies(&dashboard_dir) {
        println!("{} {}", "⚠️".yellow(), e);
    }

    println!("{} Dashboard source installed successfully!", "✅".green());

    Ok(dashboard_dir)
}

/// Ensure dashboard is installed, downloading if necessary
pub async fn ensure_dashboard() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dashboard_dir = get_dashboard_dir();

    if is_dashboard_installed() {
        // Check if node_modules exists
        let node_modules = dashboard_dir.join("node_modules");
        if !node_modules.exists() {
            println!(
                "{} Dashboard found but dependencies missing, installing...",
                "📦".yellow()
            );
            install_dependencies(&dashboard_dir)?;
        }

        println!(
            "{} Using cached dashboard from {}",
            "📂".cyan(),
            dashboard_dir.display()
        );
        Ok(dashboard_dir)
    } else {
        download_dashboard().await
    }
}

/// Get the path to run dashboard dev server from
#[allow(dead_code)]
pub fn get_dashboard_path() -> PathBuf {
    get_dashboard_dir()
}

/// Clean up downloaded dashboard assets
#[allow(dead_code)]
pub fn clean_dashboard() -> Result<(), Box<dyn std::error::Error>> {
    let dashboard_dir = get_dashboard_dir();
    if dashboard_dir.exists() {
        fs::remove_dir_all(&dashboard_dir)?;
        println!("{} Dashboard cache cleaned", "🧹".green());
    }
    Ok(())
}
