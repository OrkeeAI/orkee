use clap::Subcommand;
use colored::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use inquire::Confirm;
use orkee_projects::openspec::{
    archive_change_cli, export_specs, import_specs, list_changes, show_change, validate_change_cli,
};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum SpecCommand {
    /// List active changes or specifications
    List {
        /// Filter by project ID
        #[arg(long)]
        project: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show details of a change
    Show {
        /// Change ID to show
        change_id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show only deltas
        #[arg(long)]
        deltas_only: bool,
    },

    /// Validate changes
    Validate {
        /// Change ID to validate (or validate all if not specified)
        change_id: Option<String>,

        /// Use strict validation
        #[arg(long)]
        strict: bool,

        /// Project ID (for validating all changes in a project)
        #[arg(long)]
        project: Option<String>,
    },

    /// Archive a completed change
    Archive {
        /// Change ID to archive
        change_id: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,

        /// Skip updating specs (for tooling-only changes)
        #[arg(long)]
        skip_specs: bool,
    },

    /// Export specs to filesystem
    Export {
        /// Project ID (required for export)
        #[arg(long)]
        project: String,

        /// Path to export to
        #[arg(long, default_value = "./")]
        path: PathBuf,
    },

    /// Import specs from filesystem
    Import {
        /// Project ID (required for import)
        #[arg(long)]
        project: String,

        /// Path to import from
        #[arg(long, default_value = "./")]
        path: PathBuf,

        /// Overwrite existing data (use PreferRemote strategy)
        #[arg(long)]
        force: bool,
    },
}

pub async fn handle_spec_command(cmd: SpecCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        SpecCommand::List { project, json } => {
            list_changes_cmd(project.as_deref(), json).await
        }

        SpecCommand::Show {
            change_id,
            json,
            deltas_only,
        } => show_change_cmd(&change_id, json, deltas_only).await,

        SpecCommand::Validate {
            change_id,
            strict,
            project,
        } => validate_cmd(change_id.as_deref(), strict, project.as_deref()).await,

        SpecCommand::Archive {
            change_id,
            yes,
            skip_specs,
        } => archive_cmd(&change_id, yes, !skip_specs).await,

        SpecCommand::Export { project, path } => export_cmd(&project, &path).await,

        SpecCommand::Import { project, path, force } => import_cmd(&project, &path, force).await,
    }
}

async fn list_changes_cmd(
    project_id: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let changes = list_changes(project_id).await?;

    if changes.is_empty() {
        println!("{}", "No changes found".yellow());
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&changes)?);
    } else {
        println!("{}", "📝 OpenSpec Changes".blue().bold());
        println!();

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec!["ID", "Status", "Project", "Created"]);

        for change in &changes {
            let status_text = format!("{:?}", change.status);
            let created = change.created_at.format("%Y-%m-%d %H:%M").to_string();

            table.add_row(vec![
                &change.id[..8],
                &status_text,
                &change.project_id[..8],
                &created,
            ]);
        }

        println!("{table}");
        println!();
        println!("{} {} changes", "Total:".dimmed(), changes.len());
    }

    Ok(())
}

async fn show_change_cmd(
    change_id: &str,
    json: bool,
    deltas_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (change, deltas) = show_change(change_id).await?;

    if json {
        let output = serde_json::json!({
            "change": change,
            "deltas": deltas,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if !deltas_only {
            println!("{} {}", "Change:".blue().bold(), change.id);
            println!("{} {:?}", "Status:".cyan(), change.status);
            println!("{} {:?}", "Validation:".cyan(), change.validation_status);
            println!("{} {}", "Project:".cyan(), change.project_id);
            println!("{} {}", "Created by:".cyan(), change.created_by);
            println!(
                "{} {}",
                "Created at:".cyan(),
                change.created_at.format("%Y-%m-%d %H:%M")
            );
            println!();
            println!("{}", "Proposal:".yellow().bold());
            println!("{}", change.proposal_markdown);
            println!();
            println!("{}", "Tasks:".yellow().bold());
            println!("{}", change.tasks_markdown);
            println!();
        }

        println!("{} ({} deltas)", "Deltas:".yellow().bold(), deltas.len());
        for delta in &deltas {
            println!();
            println!(
                "{} {} - {:?}",
                "  •".cyan(),
                delta.capability_name,
                delta.delta_type
            );
            if deltas_only {
                println!("    {}", delta.delta_markdown.lines().take(5).collect::<Vec<_>>().join("\n    "));
                if delta.delta_markdown.lines().count() > 5 {
                    println!("    {}", "...".dimmed());
                }
            }
        }
    }

    Ok(())
}

async fn validate_cmd(
    change_id: Option<&str>,
    strict: bool,
    project_id: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(id) = change_id {
        // Validate single change
        let result = validate_change_cli(id, strict).await?;

        if result.is_valid {
            println!("{} Change {} is valid", "✓".green(), id);
        } else {
            println!("{} Change {} has validation errors:", "✗".red(), id);
            for error in &result.errors {
                println!("  • {}", error);
            }
            return Err("Validation failed".into());
        }
    } else {
        // Validate all changes (optionally filtered by project)
        let changes = list_changes(project_id).await?;

        let mut all_valid = true;
        for change in changes {
            let result = validate_change_cli(&change.id, strict).await?;

            if result.is_valid {
                println!("{} {}", "✓".green(), change.id);
            } else {
                println!("{} {} - {} errors", "✗".red(), change.id, result.errors.len());
                all_valid = false;
            }
        }

        if !all_valid {
            return Err("Some changes have validation errors".into());
        }
    }

    Ok(())
}

async fn archive_cmd(
    change_id: &str,
    yes: bool,
    apply_specs: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !yes {
        let apply_msg = if apply_specs {
            "and apply deltas"
        } else {
            "without applying deltas"
        };

        let confirmed = Confirm::new(&format!(
            "Archive change {} {}?",
            change_id, apply_msg
        ))
        .with_default(false)
        .prompt()?;

        if !confirmed {
            println!("{}", "Archive cancelled".yellow());
            return Ok(());
        }
    }

    archive_change_cli(change_id, apply_specs).await?;

    let apply_msg = if apply_specs {
        "and deltas applied"
    } else {
        "(no deltas applied)"
    };

    println!(
        "{} Change {} archived successfully {}",
        "✓".green(),
        change_id,
        apply_msg
    );

    Ok(())
}

async fn export_cmd(project_id: &str, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{} Exporting OpenSpec structure for project {}...",
        "📦".cyan(),
        project_id
    );

    export_specs(project_id, path).await?;

    println!(
        "{} Exported to {}",
        "✓".green(),
        path.display()
    );

    Ok(())
}

async fn import_cmd(
    project_id: &str,
    path: &PathBuf,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{} Importing OpenSpec structure for project {} from {}...",
        "📥".cyan(),
        project_id,
        path.display()
    );

    if force {
        println!(
            "{} Force mode enabled - existing data will be overwritten",
            "⚠️".yellow()
        );
    } else {
        println!(
            "{} Existing data will be preserved (use --force to overwrite)",
            "ℹ️".blue()
        );
    }

    let report = import_specs(project_id, path, force).await?;

    println!();
    println!("{}", "Import Summary:".green().bold());
    println!("  {} capabilities imported", report.capabilities_imported);
    println!("  {} capabilities skipped", report.capabilities_skipped);
    println!("  {} requirements imported", report.requirements_imported);
    println!("  {} changes imported", report.changes_imported);
    println!();
    println!(
        "{} Import completed successfully",
        "✓".green()
    );

    Ok(())
}
