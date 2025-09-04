use orkee_projects::Project;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

/// Types of items that can be mentioned
#[derive(Debug, Clone, PartialEq)]
pub enum MentionTarget {
    Projects,
    Files,
    // Future: Users, etc.
}

/// Represents a mentionable item for display in the popup
#[derive(Debug, Clone)]
pub struct MentionItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub target_type: MentionTarget,
}

impl MentionItem {
    /// Create a new mention item from a project
    pub fn from_project(project: &Project) -> Self {
        Self {
            id: project.id.clone(),
            name: project.name.clone(),
            path: project.project_root.clone(),
            description: project.description.clone(),
            target_type: MentionTarget::Projects,
        }
    }
    
    /// Get the display text for this mention item
    pub fn display_text(&self) -> String {
        match self.target_type {
            MentionTarget::Projects => {
                if let Some(desc) = &self.description {
                    format!("{} - {} ({})", self.name, self.path, desc)
                } else {
                    format!("{} - {}", self.name, self.path)
                }
            }
            MentionTarget::Files => {
                format!("{} ({})", self.name, self.path)
            }
        }
    }
    
    /// Get the text that should be inserted when this item is selected
    pub fn insertion_text(&self) -> String {
        match self.target_type {
            MentionTarget::Projects => self.name.clone(),
            MentionTarget::Files => self.path.clone(),
        }
    }
}

/// Filtered mention result with match information
#[derive(Debug, Clone)]
pub struct MentionMatch {
    pub item: MentionItem,
    pub score: i64,
    pub match_indices: Vec<usize>,
}

/// Mention popup that provides fuzzy matching and selection for @ mentions
pub struct MentionPopup {
    /// All available mention items
    items: Vec<MentionItem>,
    /// Current filtered results with match info
    filtered: Vec<MentionMatch>,
    /// Currently selected index in filtered results
    selected_index: usize,
    /// Current filter text (text after @)
    filter: String,
    /// Fuzzy matcher for item filtering
    matcher: SkimMatcherV2,
    /// Maximum number of items to display
    max_display_items: usize,
    /// The position in the input where @ was typed
    mention_start_position: usize,
}

impl std::fmt::Debug for MentionPopup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MentionPopup")
            .field("items", &self.items)
            .field("filtered", &self.filtered)
            .field("selected_index", &self.selected_index)
            .field("filter", &self.filter)
            .field("matcher", &"<SkimMatcherV2>")
            .field("max_display_items", &self.max_display_items)
            .field("mention_start_position", &self.mention_start_position)
            .finish()
    }
}

impl Default for MentionPopup {
    fn default() -> Self {
        Self::new(vec![], 0)
    }
}

impl MentionPopup {
    /// Create a new mention popup with available items
    pub fn new(items: Vec<MentionItem>, mention_start_position: usize) -> Self {
        let mut popup = Self {
            items,
            filtered: Vec::new(),
            selected_index: 0,
            filter: String::new(),
            matcher: SkimMatcherV2::default(),
            max_display_items: 6, // Reasonable default for UI
            mention_start_position,
        };
        
        // Initialize with all items visible
        popup.update_filter("");
        popup
    }
    
    /// Create a new mention popup from projects
    pub fn from_projects(projects: &[Project], mention_start_position: usize) -> Self {
        let items = projects
            .iter()
            .map(MentionItem::from_project)
            .collect();
        
        Self::new(items, mention_start_position)
    }
    
    /// Update the filter text and refresh the filtered results
    pub fn update_filter(&mut self, text: &str) {
        self.filter = text.to_string();
        self.filtered.clear();
        
        if text.is_empty() {
            // Show all items when no filter
            self.filtered = self.items
                .iter()
                .cloned()
                .map(|item| MentionMatch {
                    item,
                    score: 0,
                    match_indices: vec![],
                })
                .collect();
        } else {
            // Perform fuzzy matching on both name and path
            for item in &self.items {
                // Try to match against name first, then display text
                let name_match = self.matcher.fuzzy_indices(&item.name, text);
                let display_match = self.matcher.fuzzy_indices(&item.display_text(), text);
                
                // Use the better match or name match if both exist
                if let Some((score, indices)) = name_match.or(display_match) {
                    self.filtered.push(MentionMatch {
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
    
    /// Get the currently selected mention item, if any
    pub fn selected_item(&self) -> Option<&MentionItem> {
        self.filtered
            .get(self.selected_index)
            .map(|mention_match| &mention_match.item)
    }
    
    /// Get all filtered mention matches for UI rendering
    pub fn filtered_matches(&self) -> &[MentionMatch] {
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
    
    /// Get the position in the input where @ was typed
    pub fn mention_start_position(&self) -> usize {
        self.mention_start_position
    }
    
    /// Set the maximum number of items to display
    pub fn set_max_display_items(&mut self, max: usize) {
        self.max_display_items = max;
    }
    
    /// Reset the popup to show all items
    pub fn reset(&mut self) {
        self.update_filter("");
    }
    
    /// Get the text range that should be replaced when an item is selected
    /// Returns (start_position, end_position) in the input buffer
    pub fn replacement_range(&self, current_cursor_position: usize) -> (usize, usize) {
        (self.mention_start_position, current_cursor_position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orkee_projects::Project;

    fn create_test_project(id: &str, name: &str, path: &str) -> Project {
        Project {
            id: id.to_string(),
            name: name.to_string(),
            project_root: path.to_string(),
            status: orkee_projects::ProjectStatus::Active,
            priority: orkee_projects::Priority::Medium,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: None,
            description: None,
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            git_repository: None,
            rank: None,
            task_source: None,
            manual_tasks: None,
            mcp_servers: None,
        }
    }

    #[test]
    fn test_mention_item_from_project() {
        let project = create_test_project("1", "test-project", "/path/to/project");
        let item = MentionItem::from_project(&project);
        
        assert_eq!(item.id, "1");
        assert_eq!(item.name, "test-project");
        assert_eq!(item.path, "/path/to/project");
        assert_eq!(item.target_type, MentionTarget::Projects);
    }

    #[test]
    fn test_mention_popup_creation() {
        let projects = vec![
            create_test_project("1", "project-one", "/path/one"),
            create_test_project("2", "project-two", "/path/two"),
        ];
        
        let popup = MentionPopup::from_projects(&projects, 5);
        
        assert_eq!(popup.items.len(), 2);
        assert_eq!(popup.mention_start_position, 5);
        assert!(popup.has_results());
    }

    #[test]
    fn test_filter_empty() {
        let projects = vec![
            create_test_project("1", "project-one", "/path/one"),
            create_test_project("2", "project-two", "/path/two"),
        ];
        
        let mut popup = MentionPopup::from_projects(&projects, 0);
        popup.update_filter("");
        
        // Should show all items
        assert_eq!(popup.result_count(), 2);
        assert!(popup.has_results());
    }

    #[test]
    fn test_fuzzy_matching() {
        let projects = vec![
            create_test_project("1", "project-one", "/path/one"),
            create_test_project("2", "another-project", "/path/two"),
            create_test_project("3", "third-proj", "/path/three"),
        ];
        
        let mut popup = MentionPopup::from_projects(&projects, 0);
        
        // Test fuzzy match "proj" should match all projects
        popup.update_filter("proj");
        assert!(popup.result_count() > 0);
        
        // Test specific match "one" should match "project-one"
        popup.update_filter("one");
        let matches = popup.filtered_matches();
        assert!(matches.iter().any(|m| m.item.name.contains("one")));
    }

    #[test]
    fn test_selection_navigation() {
        let projects = vec![
            create_test_project("1", "project-one", "/path/one"),
            create_test_project("2", "project-two", "/path/two"),
            create_test_project("3", "project-three", "/path/three"),
        ];
        
        let mut popup = MentionPopup::from_projects(&projects, 0);
        popup.update_filter(""); // Show all
        
        assert_eq!(popup.selected_index(), 0);
        
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
    fn test_selected_item() {
        let projects = vec![
            create_test_project("1", "project-one", "/path/one"),
        ];
        
        let mut popup = MentionPopup::from_projects(&projects, 0);
        popup.update_filter("");
        
        let selected = popup.selected_item();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "project-one");
    }

    #[test]
    fn test_replacement_range() {
        let popup = MentionPopup::from_projects(&[], 10);
        let (start, end) = popup.replacement_range(15);
        
        assert_eq!(start, 10); // mention_start_position
        assert_eq!(end, 15);   // current_cursor_position
    }
}