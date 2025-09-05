use clap::Subcommand;
use colored::*;
use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, ContentArrangement};
use inquire::{Text, Select, Confirm};
use orkee_projects::{
    get_all_projects, get_project, create_project, update_project, delete_project,
    refresh_all_git_info, ProjectCreateInput, ProjectUpdateInput, Project, ProjectStatus, Priority,
};

#[derive(Subcommand)]
pub enum ProjectsCommands {
    /// List all projects
    List,
    /// Show project details
    Show {
        /// Project ID to show
        id: String,
    },
    /// Add a new project
    Add {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
        /// Project root path
        #[arg(short, long)]
        path: Option<String>,
        /// Project description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Edit an existing project
    Edit {
        /// Project ID to edit
        id: String,
    },
    /// Delete a project
    Delete {
        /// Project ID to delete
        id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Refresh git repository information for all projects
    RefreshGit,
}

pub async fn handle_projects_command(command: ProjectsCommands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ProjectsCommands::List => list_projects().await,
        ProjectsCommands::Show { id } => show_project(&id).await,
        ProjectsCommands::Add { name, path, description } => {
            add_project(name, path, description).await
        }
        ProjectsCommands::Edit { id } => edit_project(&id).await,
        ProjectsCommands::Delete { id, yes } => delete_project_cmd(&id, yes).await,
        ProjectsCommands::RefreshGit => refresh_git_info().await,
    }
}

async fn list_projects() -> Result<(), Box<dyn std::error::Error>> {
    let projects = get_all_projects().await?;

    if projects.is_empty() {
        println!("{}", "No projects found".yellow());
        println!("{}", "Use 'orkee projects add' to create your first project".dimmed());
        return Ok(());
    }

    println!("{}", "📂 Orkee Projects".blue().bold());
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec!["ID", "Name", "Repository", "Status", "Priority", "Tags", "Created"]);

    for project in &projects {
        let status_text = match project.status {
            ProjectStatus::Active => "Active",
            ProjectStatus::Archived => "Archived",
        };

        let priority_text = match project.priority {
            Priority::High => "High",
            Priority::Medium => "Medium", 
            Priority::Low => "Low",
        };

        let tags_text = match &project.tags {
            Some(tags) if !tags.is_empty() => tags.join(", "),
            _ => "—".to_string(),
        };

        let created_text = format_date(&project.created_at.to_rfc3339());

        table.add_row(vec![
            project.id.clone(),
            truncate(&project.name, 25),
            extract_repo_name(&project.project_root),
            status_text.to_string(),
            priority_text.to_string(),
            truncate(&tags_text, 20),
            created_text,
        ]);
    }

    println!("{}", table);
    println!("Total: {} projects", projects.len().to_string().cyan());

    Ok(())
}

async fn show_project(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    match get_project(id).await? {
        Some(project) => {
            println!("{}", format!("📂 Project Details - {}", project.name).blue().bold());
            println!();
            
            print_project_details(&project);
        }
        None => {
            eprintln!("{}", format!("Project with ID '{}' not found", id).red());
            return Err("Project not found".into());
        }
    }

    Ok(())
}

async fn add_project(
    name: Option<String>,
    path: Option<String>, 
    description: Option<String>
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "➕ Add New Project".blue().bold());
    println!();

    let name = match name {
        Some(n) => n,
        None => Text::new("Project name:").prompt()?,
    };

    let project_root = match path {
        Some(p) => p,
        None => {
            let current_dir = std::env::current_dir()?
                .to_string_lossy()
                .to_string();
            Text::new("Project root path:")
                .with_default(&current_dir)
                .prompt()?
        }
    };

    let description = match description {
        Some(d) => Some(d),
        None => {
            let desc = Text::new("Description (optional):").prompt()?;
            if desc.trim().is_empty() { None } else { Some(desc) }
        }
    };

    let status = Select::new("Status:", vec![ProjectStatus::Active, ProjectStatus::Archived])
        .prompt()?;

    let priority = Select::new("Priority:", vec![Priority::High, Priority::Medium, Priority::Low])
        .prompt()?;

    let setup_script = Text::new("Setup script (optional):")
        .with_default("npm install")
        .prompt()?;
    let setup_script = if setup_script.trim().is_empty() { None } else { Some(setup_script) };

    let dev_script = Text::new("Development script (optional):")
        .with_default("npm run dev")
        .prompt()?;
    let dev_script = if dev_script.trim().is_empty() { None } else { Some(dev_script) };

    let tags_input = Text::new("Tags (comma-separated, optional):").prompt()?;
    let tags = if tags_input.trim().is_empty() {
        None
    } else {
        Some(tags_input.split(',').map(|s| s.trim().to_string()).collect())
    };

    let project_data = ProjectCreateInput {
        name: name.clone(),
        project_root,
        description,
        status: Some(status),
        priority: Some(priority),
        setup_script,
        dev_script,
        cleanup_script: None,
        tags,
        rank: None,
        task_source: None,
        manual_tasks: None,
        mcp_servers: None,
    };

    match create_project(project_data).await {
        Ok(project) => {
            println!();
            println!("{}", format!("✅ Project '{}' created successfully!", name).green());
            println!("ID: {}", project.id.cyan());
        }
        Err(e) => {
            eprintln!("{}", format!("❌ Failed to create project: {}", e).red());
            return Err(e.into());
        }
    }

    Ok(())
}

async fn edit_project(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project = match get_project(id).await? {
        Some(p) => p,
        None => {
            eprintln!("{}", format!("Project with ID '{}' not found", id).red());
            return Err("Project not found".into());
        }
    };

    println!("{}", format!("📝 Edit Project - {}", project.name).blue().bold());
    println!();

    let name = Text::new("Project name:")
        .with_default(&project.name)
        .prompt()?;

    let project_root = Text::new("Project root path:")
        .with_default(&project.project_root)
        .prompt()?;

    let description = Text::new("Description:")
        .with_default(project.description.as_deref().unwrap_or(""))
        .prompt()?;
    let description = if description.trim().is_empty() { None } else { Some(description) };

    let status_options = vec![ProjectStatus::Active, ProjectStatus::Archived];
    let status = Select::new("Status:", status_options)
        .prompt()?;

    let priority_options = vec![Priority::High, Priority::Medium, Priority::Low];
    let priority = Select::new("Priority:", priority_options)
        .prompt()?;

    let setup_script = Text::new("Setup script:")
        .with_default(project.setup_script.as_deref().unwrap_or(""))
        .prompt()?;
    let setup_script = if setup_script.trim().is_empty() { None } else { Some(setup_script) };

    let dev_script = Text::new("Development script:")
        .with_default(project.dev_script.as_deref().unwrap_or(""))
        .prompt()?;
    let dev_script = if dev_script.trim().is_empty() { None } else { Some(dev_script) };

    let current_tags = project.tags.as_ref().map(|t| t.join(", ")).unwrap_or_default();
    let tags_input = Text::new("Tags (comma-separated):")
        .with_default(&current_tags)
        .prompt()?;
    let tags = if tags_input.trim().is_empty() {
        None
    } else {
        Some(tags_input.split(',').map(|s| s.trim().to_string()).collect())
    };

    let updates = ProjectUpdateInput {
        name: Some(name.clone()),
        project_root: Some(project_root),
        description,
        status: Some(status),
        priority: Some(priority),
        setup_script,
        dev_script,
        cleanup_script: project.cleanup_script.clone(),
        tags,
        rank: project.rank,
        task_source: project.task_source.clone(),
        manual_tasks: project.manual_tasks.clone(),
        mcp_servers: project.mcp_servers.clone(),
    };

    match update_project(id, updates).await {
        Ok(_) => {
            println!();
            println!("{}", format!("✅ Project '{}' updated successfully!", name).green());
        }
        Err(e) => {
            eprintln!("{}", format!("❌ Failed to update project: {}", e).red());
            return Err(e.into());
        }
    }

    Ok(())
}

async fn delete_project_cmd(id: &str, skip_confirmation: bool) -> Result<(), Box<dyn std::error::Error>> {
    let project = match get_project(id).await? {
        Some(p) => p,
        None => {
            eprintln!("{}", format!("Project with ID '{}' not found", id).red());
            return Err("Project not found".into());
        }
    };

    println!("{}", format!("🗑️  Delete Project - {}", project.name).red().bold());
    println!();

    print_project_details(&project);
    println!();

    let confirmed = if skip_confirmation {
        true
    } else {
        Confirm::new(&format!("Are you sure you want to delete '{}'?", project.name))
            .with_default(false)
            .prompt()?
    };

    if confirmed {
        match delete_project(id).await {
            Ok(true) => {
                println!("{}", format!("✅ Project '{}' deleted successfully!", project.name).green());
            }
            Ok(false) => {
                eprintln!("{}", "❌ Project not found".red());
                return Err("Project not found".into());
            }
            Err(e) => {
                eprintln!("{}", format!("❌ Failed to delete project: {}", e).red());
                return Err(e.into());
            }
        }
    } else {
        println!("{}", "❌ Operation cancelled".yellow());
    }

    Ok(())
}

fn print_project_details(project: &Project) {
    println!("{:<15} {}", "ID:".cyan(), project.id);
    println!("{:<15} {}", "Name:".cyan(), project.name);
    println!("{:<15} {}", "Repository:".cyan(), extract_repo_name(&project.project_root));
    println!("{:<15} {}", "Path:".cyan(), project.project_root);
    
    let status_colored = match project.status {
        ProjectStatus::Active => "Active".green(),
        ProjectStatus::Archived => "Archived".yellow(),
    };
    println!("{:<15} {}", "Status:".cyan(), status_colored);
    
    let priority_colored = match project.priority {
        Priority::High => "High".red(),
        Priority::Medium => "Medium".yellow(),
        Priority::Low => "Low".green(),
    };
    println!("{:<15} {}", "Priority:".cyan(), priority_colored);

    if let Some(description) = &project.description {
        if !description.trim().is_empty() {
            println!("{:<15} {}", "Description:".cyan(), description);
        }
    }

    if let Some(tags) = &project.tags {
        if !tags.is_empty() {
            println!("{:<15} {}", "Tags:".cyan(), tags.join(", "));
        }
    }

    if let Some(setup_script) = &project.setup_script {
        if !setup_script.trim().is_empty() {
            println!("{:<15} {}", "Setup Script:".cyan(), setup_script);
        }
    }

    if let Some(dev_script) = &project.dev_script {
        if !dev_script.trim().is_empty() {
            println!("{:<15} {}", "Dev Script:".cyan(), dev_script);
        }
    }

    if let Some(cleanup_script) = &project.cleanup_script {
        if !cleanup_script.trim().is_empty() {
            println!("{:<15} {}", "Cleanup Script:".cyan(), cleanup_script);
        }
    }

    println!("{:<15} {}", "Created:".cyan(), format_date(&project.created_at.to_rfc3339()));
    println!("{:<15} {}", "Updated:".cyan(), format_date(&project.updated_at.to_rfc3339()));
}

fn format_date(date_str: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(date_str) {
        Ok(dt) => dt.format("%-m/%-d/%Y").to_string(),
        Err(_) => date_str.to_string(),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn extract_repo_name(path: &str) -> String {
    // Try to get Git remote origin URL
    if let Ok(repo) = git2::Repository::open(path) {
        if let Ok(remote) = repo.find_remote("origin") {
            if let Some(url) = remote.url() {
                return parse_git_url(url);
            }
        }
    }
    
    // No Git repository or no remote origin
    "No remote repository".to_string()
}

async fn refresh_git_info() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🔄 Refreshing Git Repository Information".blue().bold());
    println!();
    
    match refresh_all_git_info().await {
        Ok(count) => {
            if count > 0 {
                println!("{}", format!("✅ Successfully updated git info for {} project(s)", count).green());
            } else {
                println!("{}", "ℹ️  All projects already have up-to-date git repository information".yellow());
            }
        }
        Err(e) => {
            eprintln!("{}", format!("❌ Failed to refresh git info: {}", e).red());
            return Err(e.into());
        }
    }
    
    Ok(())
}

fn parse_git_url(url: &str) -> String {
    // Handle GitHub SSH URLs: git@github.com:username/repo.git
    if url.starts_with("git@github.com:") {
        let without_prefix = url.strip_prefix("git@github.com:").unwrap_or(url);
        let without_suffix = without_prefix.strip_suffix(".git").unwrap_or(without_prefix);
        return without_suffix.to_string();
    }
    
    // Handle GitHub HTTPS URLs: https://github.com/username/repo.git
    if url.starts_with("https://github.com/") {
        let without_prefix = url.strip_prefix("https://github.com/").unwrap_or(url);
        let without_suffix = without_prefix.strip_suffix(".git").unwrap_or(without_prefix);
        return without_suffix.to_string();
    }
    
    // Handle other Git hosting services or generic URLs
    if let Ok(parsed_url) = url::Url::parse(url) {
        if let Some(path) = parsed_url.path().strip_prefix('/') {
            let without_suffix = path.strip_suffix(".git").unwrap_or(path);
            return without_suffix.to_string();
        }
    }
    
    // If all else fails, try to extract from the URL string
    if let Some(start) = url.find('/').or_else(|| url.find(':')) {
        let remaining = &url[start + 1..];
        let without_suffix = remaining.strip_suffix(".git").unwrap_or(remaining);
        return without_suffix.to_string();
    }
    
    "Unknown".to_string()
}