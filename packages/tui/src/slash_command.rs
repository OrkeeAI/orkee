use strum_macros::{EnumString, EnumIter, AsRefStr, IntoStaticStr};
use strum::IntoEnumIterator;

/// Slash commands available in the TUI
#[derive(Debug, Clone, PartialEq, EnumString, EnumIter, AsRefStr, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    /// Show available commands and help
    Help,
    /// Exit the application
    Quit,
    /// Clear the chat history
    Clear,
    /// List all projects
    Projects,
    /// Show current application status
    Status,
    /// Switch to dashboard screen
    Dashboard,
    /// Switch to settings screen  
    Settings,
}

impl SlashCommand {
    /// Get user-friendly description for the command
    pub fn description(&self) -> &'static str {
        match self {
            Self::Help => "Show available commands and usage information",
            Self::Quit => "Exit the application",
            Self::Clear => "Clear the chat history",
            Self::Projects => "Open interactive projects screen",
            Self::Status => "Show current application status and information",
            Self::Dashboard => "Switch to dashboard screen",
            Self::Settings => "Switch to settings screen",
        }
    }
    
    /// Get usage example for the command
    pub fn usage(&self) -> &'static str {
        match self {
            Self::Help => "/help",
            Self::Quit => "/quit",
            Self::Clear => "/clear", 
            Self::Projects => "/projects",
            Self::Status => "/status",
            Self::Dashboard => "/dashboard",
            Self::Settings => "/settings",
        }
    }
    
    /// Check if the command requires arguments
    pub fn requires_args(&self) -> bool {
        false // No commands require arguments anymore
    }
    
    /// Check if command is available during active task execution
    pub fn available_during_task(&self) -> bool {
        // For now, all commands are available. This could be restricted later.
        true
    }
    
    /// Get all built-in slash commands for UI display
    pub fn built_in_commands() -> Vec<Self> {
        Self::iter().collect()
    }
    
    /// Parse command from input string, extracting command and arguments
    pub fn parse_from_input(input: &str) -> Result<(Self, Vec<String>), String> {
        let trimmed = input.trim();
        
        // Remove leading slash
        let without_slash = trimmed.strip_prefix('/')
            .ok_or_else(|| "Input must start with '/'")?;
        
        // Split into parts
        let parts: Vec<&str> = without_slash.split_whitespace().collect();
        
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }
        
        // Parse command name
        let command_str = parts[0];
        let command = Self::try_from(command_str)
            .map_err(|_| format!("Unknown command: /{}", command_str))?;
        
        // Extract arguments
        let args: Vec<String> = parts.into_iter()
            .skip(1)
            .map(|s| s.to_string())
            .collect();
        
        // Validate arguments
        match command {
            Self::Help | Self::Quit | Self::Clear | Self::Projects | Self::Status | Self::Dashboard | Self::Settings if !args.is_empty() => {
                Err(format!("Command /{} does not accept arguments", command.as_ref()))
            }
            _ => Ok((command, args))
        }
    }
    
    /// Get the command name as it appears after the slash
    pub fn command_name(&self) -> &str {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_parsing_valid() {
        let (cmd, args) = SlashCommand::parse_from_input("/help").unwrap();
        assert_eq!(cmd, SlashCommand::Help);
        assert!(args.is_empty());
        
        let (cmd, args) = SlashCommand::parse_from_input("/project test").unwrap();
        assert_eq!(cmd, SlashCommand::Project);
        assert_eq!(args, vec!["test"]);
        
        let (cmd, args) = SlashCommand::parse_from_input("/projects").unwrap();
        assert_eq!(cmd, SlashCommand::Projects);
        assert!(args.is_empty());
    }
    
    #[test]
    fn test_command_parsing_errors() {
        assert!(SlashCommand::parse_from_input("help").is_err()); // No slash
        assert!(SlashCommand::parse_from_input("/").is_err()); // Empty command
        assert!(SlashCommand::parse_from_input("/unknown").is_err()); // Unknown command
        assert!(SlashCommand::parse_from_input("/project").is_err()); // Missing required arg
        assert!(SlashCommand::parse_from_input("/help extra").is_err()); // Unexpected args
    }
    
    #[test]
    fn test_command_metadata() {
        let help = SlashCommand::Help;
        assert_eq!(help.usage(), "/help");
        assert!(!help.requires_args());
        assert!(help.available_during_task());
        assert!(!help.description().is_empty());
        
        let project = SlashCommand::Project;
        assert_eq!(project.usage(), "/project <name>");
        assert!(project.requires_args());
    }
    
    #[test]
    fn test_built_in_commands() {
        let commands = SlashCommand::built_in_commands();
        assert!(!commands.is_empty());
        assert!(commands.contains(&SlashCommand::Help));
        assert!(commands.contains(&SlashCommand::Projects));
        assert!(commands.contains(&SlashCommand::Project));
    }
    
    #[test]
    fn test_enum_string_conversion() {
        assert_eq!("help", SlashCommand::Help.as_ref());
        assert_eq!("projects", SlashCommand::Projects.as_ref());
        assert_eq!("project", SlashCommand::Project.as_ref());
        
        // Test parsing from string
        assert_eq!(Ok(SlashCommand::Help), "help".parse());
        assert_eq!(Ok(SlashCommand::Projects), "projects".parse());
        assert_eq!(Ok(SlashCommand::Project), "project".parse());
    }
}