use crate::slash_command::SlashCommand;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

/// Represents a command item for display in the popup
#[derive(Debug, Clone)]
pub struct CommandItem {
    pub command: SlashCommand,
    pub name: String,
    pub usage: String,
    pub description: String,
}

impl CommandItem {
    /// Create a new command item from a SlashCommand
    pub fn from_command(command: SlashCommand) -> Self {
        Self {
            name: command.command_name().to_string(),
            usage: command.usage().to_string(),
            description: command.description().to_string(),
            command,
        }
    }
}

/// Filtered command result with match information
#[derive(Debug, Clone)]
pub struct CommandMatch {
    pub item: CommandItem,
    pub score: i64,
    pub match_indices: Vec<usize>,
}

/// Command popup that provides fuzzy matching and selection
pub struct CommandPopup {
    /// All available commands
    commands: Vec<CommandItem>,
    /// Current filtered results with match info
    filtered: Vec<CommandMatch>,
    /// Currently selected index in filtered results
    selected_index: usize,
    /// Current filter text
    filter: String,
    /// Fuzzy matcher for command filtering
    matcher: SkimMatcherV2,
    /// Maximum number of items to display
    max_display_items: usize,
}

impl std::fmt::Debug for CommandPopup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandPopup")
            .field("commands", &self.commands)
            .field("filtered", &self.filtered)
            .field("selected_index", &self.selected_index)
            .field("filter", &self.filter)
            .field("matcher", &"<SkimMatcherV2>")
            .field("max_display_items", &self.max_display_items)
            .finish()
    }
}

impl Default for CommandPopup {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPopup {
    /// Create a new command popup with all built-in commands
    pub fn new() -> Self {
        let commands = SlashCommand::built_in_commands()
            .into_iter()
            .map(CommandItem::from_command)
            .collect();

        let mut popup = Self {
            commands,
            filtered: Vec::new(),
            selected_index: 0,
            filter: String::new(),
            matcher: SkimMatcherV2::default(),
            max_display_items: 8, // Reasonable default for UI
        };
        
        // Initialize with all commands visible
        popup.update_filter("");
        popup
    }
    
    /// Update the filter text and refresh the filtered results
    pub fn update_filter(&mut self, text: &str) {
        self.filter = text.to_string();
        self.filtered.clear();
        
        // Remove leading '/' if present for matching
        let search_text = text.trim_start_matches('/');
        
        if search_text.is_empty() {
            // Show all commands when no filter
            self.filtered = self.commands
                .iter()
                .cloned()
                .map(|item| CommandMatch {
                    item,
                    score: 0,
                    match_indices: vec![],
                })
                .collect();
        } else {
            // Perform fuzzy matching
            for item in &self.commands {
                // Try to match against both command name and usage
                let name_match = self.matcher.fuzzy_indices(&item.name, search_text);
                let usage_match = self.matcher.fuzzy_indices(&item.usage, search_text);
                
                // Use the better match or name match if both exist
                if let Some((score, indices)) = name_match.or(usage_match) {
                    self.filtered.push(CommandMatch {
                        item: item.clone(),
                        score,
                        match_indices: indices,
                    });
                }
            }
            
            // Sort by score (highest first) and then by name
            self.filtered.sort_by(|a, b| {
                b.score.cmp(&a.score)
                    .then_with(|| a.item.name.cmp(&b.item.name))
            });
        }
        
        // Limit display items
        self.filtered.truncate(self.max_display_items);
        
        // Reset selection to first item
        self.selected_index = 0;
    }
    
    /// Move selection up in the filtered list
    pub fn move_up(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            // Wrap to bottom
            self.selected_index = self.filtered.len() - 1;
        }
    }
    
    /// Move selection down in the filtered list
    pub fn move_down(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        
        if self.selected_index < self.filtered.len() - 1 {
            self.selected_index += 1;
        } else {
            // Wrap to top
            self.selected_index = 0;
        }
    }
    
    /// Get the currently selected command, if any
    pub fn selected_command(&self) -> Option<&SlashCommand> {
        self.filtered
            .get(self.selected_index)
            .map(|cmd_match| &cmd_match.item.command)
    }
    
    /// Get the currently selected command item, if any
    pub fn selected_item(&self) -> Option<&CommandItem> {
        self.filtered
            .get(self.selected_index)
            .map(|cmd_match| &cmd_match.item)
    }
    
    /// Get all filtered command matches for UI rendering
    pub fn filtered_matches(&self) -> &[CommandMatch] {
        &self.filtered
    }
    
    /// Get the current filter text
    pub fn filter(&self) -> &str {
        &self.filter
    }
    
    /// Get the selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }
    
    /// Check if there are any filtered results
    pub fn has_results(&self) -> bool {
        !self.filtered.is_empty()
    }
    
    /// Get the number of filtered results
    pub fn result_count(&self) -> usize {
        self.filtered.len()
    }
    
    /// Set the maximum number of items to display
    pub fn set_max_display_items(&mut self, max: usize) {
        self.max_display_items = max;
    }
    
    /// Reset the popup to show all commands
    pub fn reset(&mut self) {
        self.update_filter("");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_creation() {
        let popup = CommandPopup::new();
        assert!(!popup.commands.is_empty());
        assert!(popup.has_results());
    }
    
    #[test]
    fn test_filter_empty() {
        let mut popup = CommandPopup::new();
        popup.update_filter("");
        
        // Should show all commands
        assert_eq!(popup.result_count(), popup.commands.len());
        assert!(popup.has_results());
    }
    
    #[test]
    fn test_filter_matching() {
        let mut popup = CommandPopup::new();
        
        // Filter for "help"
        popup.update_filter("help");
        assert!(popup.has_results());
        
        // Should find help command
        let matches = popup.filtered_matches();
        assert!(matches.iter().any(|m| m.item.command == SlashCommand::Help));
    }
    
    #[test]
    fn test_fuzzy_matching() {
        let mut popup = CommandPopup::new();
        
        // Fuzzy match "proj" should match "projects" and "project"
        popup.update_filter("proj");
        let matches = popup.filtered_matches();
        
        assert!(matches.iter().any(|m| m.item.command == SlashCommand::Projects));
        assert!(matches.iter().any(|m| m.item.command == SlashCommand::Project));
    }
    
    #[test]
    fn test_selection_navigation() {
        let mut popup = CommandPopup::new();
        popup.update_filter(""); // Show all commands
        
        let initial_selection = popup.selected_index();
        assert_eq!(initial_selection, 0);
        
        // Move down
        popup.move_down();
        assert_eq!(popup.selected_index(), 1);
        
        // Move up (should wrap to end if at beginning)
        popup.selected_index = 0;
        popup.move_up();
        assert_eq!(popup.selected_index(), popup.result_count() - 1);
        
        // Move down (should wrap to beginning if at end)
        popup.selected_index = popup.result_count() - 1;
        popup.move_down();
        assert_eq!(popup.selected_index(), 0);
    }
    
    #[test]
    fn test_selected_command() {
        let mut popup = CommandPopup::new();
        popup.update_filter("help");
        
        if popup.has_results() {
            let selected = popup.selected_command();
            assert!(selected.is_some());
            
            let item = popup.selected_item();
            assert!(item.is_some());
        }
    }
    
    #[test]
    fn test_slash_prefix_handling() {
        let mut popup = CommandPopup::new();
        
        // Both "/help" and "help" should work the same
        popup.update_filter("/help");
        let with_slash = popup.result_count();
        
        popup.update_filter("help");
        let without_slash = popup.result_count();
        
        assert_eq!(with_slash, without_slash);
    }
    
    #[test]
    fn test_max_display_items() {
        let mut popup = CommandPopup::new();
        popup.set_max_display_items(2);
        
        popup.update_filter(""); // Show all, but limit to 2
        assert!(popup.result_count() <= 2);
    }
}