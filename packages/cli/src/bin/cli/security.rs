// ABOUTME: CLI commands for managing API key encryption security
// ABOUTME: Supports password-based encryption upgrade/downgrade and status checking

use clap::Subcommand;
use colored::*;
use inquire::{Password, PasswordDisplayMode};
use orkee_projects::security::encryption::{ApiKeyEncryption, EncryptionMode};
use orkee_projects::ProjectManager;
use std::process;

#[derive(Subcommand)]
pub enum SecurityCommands {
    /// Set a password to enable password-based encryption (upgrades from machine-based)
    SetPassword,
    /// Change the encryption password
    ChangePassword,
    /// Remove password-based encryption (downgrades to machine-based)
    RemovePassword,
    /// Show current encryption mode and security status
    Status,
}

impl SecurityCommands {
    pub async fn execute(&self) {
        match self {
            SecurityCommands::SetPassword => set_password_command().await,
            SecurityCommands::ChangePassword => change_password_command().await,
            SecurityCommands::RemovePassword => remove_password_command().await,
            SecurityCommands::Status => status_command().await,
        }
    }
}

async fn set_password_command() {
    println!(
        "{}",
        "Setting up password-based encryption...".bold().cyan()
    );
    println!();

    // Initialize project manager to access database
    let manager = match ProjectManager::new().await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{} Failed to initialize: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    // Check current encryption mode
    match manager.get_encryption_mode().await {
        Ok(Some(EncryptionMode::Password)) => {
            eprintln!(
                "{} Password-based encryption is already enabled.",
                "✗".red().bold()
            );
            eprintln!(
                "  Use {} to change the password.",
                "orkee security change-password".yellow()
            );
            process::exit(1);
        }
        Ok(Some(EncryptionMode::Machine)) | Ok(None) => {
            // Continue with setup
        }
        Err(e) => {
            eprintln!(
                "{} Failed to check encryption mode: {}",
                "✗".red().bold(),
                e
            );
            process::exit(1);
        }
    }

    println!("{}", "⚠ SECURITY WARNING:".yellow().bold());
    println!("  • Machine-based encryption only protects data during transfer");
    println!("  • Password-based encryption provides true at-rest security");
    println!("  • You will need this password to access encrypted API keys");
    println!("  • If you forget your password, encrypted keys cannot be recovered");
    println!();

    // Prompt for password
    let password = match Password::new("Enter new password:")
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_help_message("Minimum 8 characters recommended")
        .prompt()
    {
        Ok(p) => p,
        Err(_) => {
            eprintln!("{} Password input cancelled", "✗".red().bold());
            process::exit(1);
        }
    };

    if password.len() < 8 {
        eprintln!(
            "{} Password must be at least 8 characters",
            "✗".red().bold()
        );
        process::exit(1);
    }

    // Confirm password
    let password_confirm = match Password::new("Confirm password:")
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt()
    {
        Ok(p) => p,
        Err(_) => {
            eprintln!("{} Password confirmation cancelled", "✗".red().bold());
            process::exit(1);
        }
    };

    if password != password_confirm {
        eprintln!("{} Passwords do not match", "✗".red().bold());
        process::exit(1);
    }

    // Generate salt
    let salt = match ApiKeyEncryption::generate_salt() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} Failed to generate salt: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    // Generate verification hash
    let verification_hash = match ApiKeyEncryption::hash_password_for_verification(&password, &salt)
    {
        Ok(h) => h,
        Err(e) => {
            eprintln!("{} Failed to hash password: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    // Save encryption settings to database
    match manager
        .set_encryption_mode(
            EncryptionMode::Password,
            Some(&salt),
            Some(&verification_hash),
        )
        .await
    {
        Ok(_) => {
            println!();
            println!(
                "{} Password-based encryption enabled successfully!",
                "✓".green().bold()
            );
            println!();
            println!("{}", "Next steps:".bold());
            println!("  • Your API keys are now encrypted with your password");
            println!("  • Keep your password secure - it cannot be recovered if lost");
            println!(
                "  • Use {} to check encryption status",
                "orkee security status".cyan()
            );
            println!();
        }
        Err(e) => {
            eprintln!(
                "{} Failed to enable password-based encryption: {}",
                "✗".red().bold(),
                e
            );
            process::exit(1);
        }
    }
}

async fn change_password_command() {
    println!("{}", "Changing encryption password...".bold().cyan());
    println!();

    let manager = match ProjectManager::new().await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{} Failed to initialize: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    // Check current encryption mode
    let (current_mode, current_salt, current_hash) = match manager.get_encryption_settings().await {
        Ok(Some((mode, salt, hash))) => (mode, salt, hash),
        Ok(None) => {
            eprintln!("{} No encryption settings found", "✗".red().bold());
            process::exit(1);
        }
        Err(e) => {
            eprintln!(
                "{} Failed to get encryption settings: {}",
                "✗".red().bold(),
                e
            );
            process::exit(1);
        }
    };

    if current_mode != EncryptionMode::Password {
        eprintln!(
            "{} Password-based encryption is not enabled.",
            "✗".red().bold()
        );
        eprintln!(
            "  Use {} to enable it first.",
            "orkee security set-password".yellow()
        );
        process::exit(1);
    }

    let salt = match current_salt {
        Some(s) => s,
        None => {
            eprintln!("{} No salt found in encryption settings", "✗".red().bold());
            process::exit(1);
        }
    };

    let hash = match current_hash {
        Some(h) => h,
        None => {
            eprintln!("{} No hash found in encryption settings", "✗".red().bold());
            process::exit(1);
        }
    };

    // Check if account is locked due to too many failed attempts
    if let Err(e) = manager.check_password_lockout().await {
        eprintln!("{} {}", "✗".red().bold(), e);
        process::exit(1);
    }

    // Verify current password
    let current_password = match Password::new("Enter current password:")
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt()
    {
        Ok(p) => p,
        Err(_) => {
            eprintln!("{} Password input cancelled", "✗".red().bold());
            process::exit(1);
        }
    };

    match ApiKeyEncryption::verify_password(&current_password, &salt, &hash) {
        Ok(true) => {
            // Password correct, reset attempt counter
            if let Err(e) = manager.reset_password_attempts().await {
                eprintln!(
                    "{} Warning: Failed to reset password attempts: {}",
                    "⚠".yellow().bold(),
                    e
                );
                // Continue anyway - this is not critical
            }
        }
        Ok(false) => {
            // Record failed attempt
            if let Err(e) = manager.record_failed_password_attempt().await {
                eprintln!(
                    "{} Warning: Failed to record attempt: {}",
                    "⚠".yellow().bold(),
                    e
                );
                // Continue to show error anyway
            }
            eprintln!("{} Current password is incorrect", "✗".red().bold());
            process::exit(1);
        }
        Err(e) => {
            eprintln!("{} Password verification failed: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    }

    // Prompt for new password
    let new_password = match Password::new("Enter new password:")
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_help_message("Minimum 8 characters recommended")
        .prompt()
    {
        Ok(p) => p,
        Err(_) => {
            eprintln!("{} Password input cancelled", "✗".red().bold());
            process::exit(1);
        }
    };

    if new_password.len() < 8 {
        eprintln!(
            "{} Password must be at least 8 characters",
            "✗".red().bold()
        );
        process::exit(1);
    }

    // Confirm new password
    let new_password_confirm = match Password::new("Confirm new password:")
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt()
    {
        Ok(p) => p,
        Err(_) => {
            eprintln!("{} Password confirmation cancelled", "✗".red().bold());
            process::exit(1);
        }
    };

    if new_password != new_password_confirm {
        eprintln!("{} Passwords do not match", "✗".red().bold());
        process::exit(1);
    }

    // Generate new salt (best practice)
    let new_salt = match ApiKeyEncryption::generate_salt() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} Failed to generate salt: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    // Generate new verification hash
    let new_hash = match ApiKeyEncryption::hash_password_for_verification(&new_password, &new_salt)
    {
        Ok(h) => h,
        Err(e) => {
            eprintln!("{} Failed to hash password: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    // Initialize database access to rotate API keys
    use orkee_projects::DbState;
    let db_state = match DbState::init().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("{} Failed to initialize database: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    // Create encryption instances for key rotation
    let old_encryption = match ApiKeyEncryption::with_password(&current_password, &salt) {
        Ok(enc) => enc,
        Err(e) => {
            eprintln!(
                "{} Failed to create old encryption: {}",
                "✗".red().bold(),
                e
            );
            process::exit(1);
        }
    };

    let new_encryption = match ApiKeyEncryption::with_password(&new_password, &new_salt) {
        Ok(enc) => enc,
        Err(e) => {
            eprintln!(
                "{} Failed to create new encryption: {}",
                "✗".red().bold(),
                e
            );
            process::exit(1);
        }
    };

    // Atomically rotate encryption keys AND update encryption settings
    println!();
    println!("{}", "Changing password (rotating keys and updating settings)...".cyan());
    match db_state
        .change_encryption_password_atomic(
            "default-user",
            &old_encryption,
            &new_encryption,
            EncryptionMode::Password,
            &new_salt,
            &new_hash,
        )
        .await
    {
        Ok(_) => {
            println!("{} Password changed successfully!", "✓".green().bold());
            println!();
            println!("{}", "Security status:".bold());
            println!("  • All encrypted API keys have been re-encrypted with the new password");
            println!("  • Encryption settings updated");
            println!("  • You will need the new password to access encrypted API keys");
            println!();
        }
        Err(e) => {
            eprintln!(
                "{} Failed to change password: {}",
                "✗".red().bold(),
                e
            );
            eprintln!("  No changes were made - your existing password still works");
            process::exit(1);
        }
    }
}

async fn remove_password_command() {
    println!("{}", "Removing password-based encryption...".bold().cyan());
    println!();

    let manager = match ProjectManager::new().await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{} Failed to initialize: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    // Check current encryption mode
    match manager.get_encryption_mode().await {
        Ok(Some(EncryptionMode::Machine)) | Ok(None) => {
            eprintln!(
                "{} Password-based encryption is not enabled.",
                "✗".red().bold()
            );
            process::exit(1);
        }
        Ok(Some(EncryptionMode::Password)) => {
            // Continue with removal
        }
        Err(e) => {
            eprintln!(
                "{} Failed to check encryption mode: {}",
                "✗".red().bold(),
                e
            );
            process::exit(1);
        }
    }

    println!("{}", "⚠ SECURITY WARNING:".yellow().bold());
    println!("  • This will downgrade to machine-based encryption");
    println!("  • Machine-based encryption only protects data during transfer");
    println!("  • Anyone with local file access can decrypt API keys");
    println!();

    // Confirm removal
    let confirm =
        match inquire::Confirm::new("Are you sure you want to remove password-based encryption?")
            .with_default(false)
            .prompt()
        {
            Ok(c) => c,
            Err(_) => {
                eprintln!("{} Operation cancelled", "✗".red().bold());
                process::exit(1);
            }
        };

    if !confirm {
        println!("Operation cancelled");
        process::exit(0);
    }

    // Downgrade to machine-based encryption
    match manager
        .set_encryption_mode(EncryptionMode::Machine, None, None)
        .await
    {
        Ok(_) => {
            println!();
            println!("{} Password-based encryption removed", "✓".green().bold());
            println!("  Reverted to machine-based encryption");
            println!();
        }
        Err(e) => {
            eprintln!(
                "{} Failed to remove password-based encryption: {}",
                "✗".red().bold(),
                e
            );
            process::exit(1);
        }
    }
}

async fn status_command() {
    let manager = match ProjectManager::new().await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{} Failed to initialize: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    let mode = match manager.get_encryption_mode().await {
        Ok(Some(m)) => m,
        Ok(None) => EncryptionMode::Machine, // Default
        Err(e) => {
            eprintln!("{} Failed to get encryption mode: {}", "✗".red().bold(), e);
            process::exit(1);
        }
    };

    println!();
    println!("{}", "Encryption Security Status".bold().cyan());
    println!("{}", "═══════════════════════════".cyan());
    println!();

    match mode {
        EncryptionMode::Machine => {
            println!(
                "{} {}",
                "Current Mode:".bold(),
                "Machine-Based Encryption".yellow()
            );
            println!();
            println!("{}", "Security Level:".bold());
            println!("  • {} Transport encryption only", "⚠".yellow());
            println!("  • Protects data during backup/sync");
            println!("  • Does NOT protect at-rest on local machine");
            println!("  • Anyone with local database access can decrypt keys");
            println!();
            println!("{}", "Recommendation:".bold());
            println!(
                "  Use {} for stronger security",
                "orkee security set-password".green()
            );
            println!();
        }
        EncryptionMode::Password => {
            println!(
                "{} {}",
                "Current Mode:".bold(),
                "Password-Based Encryption".green()
            );
            println!();
            println!("{}", "Security Level:".bold());
            println!("  • {} True at-rest encryption", "✓".green());
            println!("  • Data cannot be decrypted without password");
            println!("  • Suitable for shared machines and sensitive environments");
            println!();
            println!("{}", "Management:".bold());
            println!(
                "  • Change password: {}",
                "orkee security change-password".cyan()
            );
            println!(
                "  • Remove protection: {}",
                "orkee security remove-password".cyan()
            );
            println!();
        }
    }
}
