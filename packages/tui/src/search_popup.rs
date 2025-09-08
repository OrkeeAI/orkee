use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use orkee_projects::{Priority, Project, ProjectStatus};
use std::time::{Duration, Instant};
use tui_input::Input;

/// Which field was matched during search
#[derive(Debug, Clone, PartialEq)]
pub enum MatchedField {
    Name,
    Description,
    Path,
    Tag(String),
}

/// Filtered project result with match information
#[derive(Debug, Clone)]
pub struct ProjectMatch {
    /// Index in the original projects array
    pub project_index: usize,
    /// Reference to the project
    pub project: Project,
    /// Fuzzy match score (higher = better match)
    pub score: i64,
    /// Indices of matched characters for highlighting
    pub match_indices: Vec<usize>,
    /// Which field was matched
    pub matched_field: MatchedField,
}

/// Current search mode
#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    /// Free text fuzzy search
    Text,
    /// Filter by project status
    Status,
    /// Filter by project priority
    Priority,
    /// Filter by tags
    Tags,
}

/// Project search popup with fuzzy matching and filtering
pub struct SearchPopup {
    /// Current search text input
    search_input: Input,
    /// Current search mode
    search_mode: SearchMode,
    /// Active status filter
    filter_status: Option<ProjectStatus>,
    /// Active priority filter  
    filter_priority: Option<Priority>,
    /// Active tag filters (matches any of these tags)
    filter_tags: Vec<String>,
    /// Current filtered results with match info
    filtered_results: Vec<ProjectMatch>,
    /// Currently selected index in filtered results
    selected_index: usize,
    /// Fuzzy matcher for text search
    matcher: SkimMatcherV2,
    /// Maximum number of items to display in popup
    max_display_items: usize,
    /// Cache of last search query to avoid redundant searches
    last_search_query: String,
    /// Whether to use cached results
    use_cache: bool,
    /// Last time search was updated for debouncing
    last_update_time: Option<Instant>,
    /// Debounce duration to prevent excessive search updates
    debounce_duration: Duration,
    /// Pending search update flag
    pending_update: bool,
}

impl std::fmt::Debug for SearchPopup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchPopup")
            .field("search_input", &self.search_input.value())
            .field("search_mode", &self.search_mode)
            .field("filter_status", &self.filter_status)
            .field("filter_priority", &self.filter_priority)
            .field("filter_tags", &self.filter_tags)
            .field("filtered_results", &self.filtered_results.len())
            .field("selected_index", &self.selected_index)
            .field("max_display_items", &self.max_display_items)
            .field("debounce_duration", &self.debounce_duration)
            .field("pending_update", &self.pending_update)
            .finish()
    }
}

impl Default for SearchPopup {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchPopup {
    /// Create a new search popup
    pub fn new() -> Self {
        Self {
            search_input: Input::default(),
            search_mode: SearchMode::Text,
            filter_status: None,
            filter_priority: None,
            filter_tags: Vec::new(),
            filtered_results: Vec::new(),
            selected_index: 0,
            matcher: SkimMatcherV2::default(),
            max_display_items: 10,
            last_search_query: String::new(),
            use_cache: false,
            last_update_time: None,
            debounce_duration: Duration::from_millis(100), // 100ms debounce
            pending_update: false,
        }
    }

    /// Get current search query
    pub fn search_query(&self) -> &str {
        self.search_input.value()
    }

    /// Get mutable reference to search input for keyboard handling
    pub fn search_input_mut(&mut self) -> &mut Input {
        &mut self.search_input
    }

    /// Get current search mode
    pub fn search_mode(&self) -> &SearchMode {
        &self.search_mode
    }

    /// Cycle to next search mode
    pub fn cycle_search_mode(&mut self) {
        self.search_mode = match self.search_mode {
            SearchMode::Text => SearchMode::Status,
            SearchMode::Status => SearchMode::Priority,
            SearchMode::Priority => SearchMode::Tags,
            SearchMode::Tags => SearchMode::Text,
        };
    }

    /// Get filtered results
    pub fn filtered_results(&self) -> &[ProjectMatch] {
        &self.filtered_results
    }

    /// Get currently selected result
    pub fn selected_result(&self) -> Option<&ProjectMatch> {
        self.filtered_results.get(self.selected_index)
    }

    /// Get selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get active filters as display strings
    pub fn active_filters(&self) -> Vec<String> {
        let mut filters = Vec::new();

        if let Some(status) = &self.filter_status {
            filters.push(format!("Status: {:?}", status));
        }

        if let Some(priority) = &self.filter_priority {
            filters.push(format!("Priority: {:?}", priority));
        }

        if !self.filter_tags.is_empty() {
            filters.push(format!("Tags: {}", self.filter_tags.join(", ")));
        }

        filters
    }

    /// Check if any filters are active
    pub fn has_active_filters(&self) -> bool {
        self.filter_status.is_some()
            || self.filter_priority.is_some()
            || !self.filter_tags.is_empty()
    }

    /// Toggle status filter
    pub fn toggle_status_filter(&mut self, status: Option<ProjectStatus>) {
        self.filter_status = status;
        self.invalidate_cache();
    }

    /// Toggle priority filter
    pub fn toggle_priority_filter(&mut self, priority: Option<Priority>) {
        self.filter_priority = priority;
        self.invalidate_cache();
    }

    /// Add tag filter
    pub fn add_tag_filter(&mut self, tag: String) {
        if !self.filter_tags.contains(&tag) {
            self.filter_tags.push(tag);
            self.invalidate_cache();
        }
    }

    /// Remove tag filter
    pub fn remove_tag_filter(&mut self, tag: &str) {
        self.filter_tags.retain(|t| t != tag);
        self.invalidate_cache();
    }

    /// Clear all filters
    pub fn clear_filters(&mut self) {
        self.filter_status = None;
        self.filter_priority = None;
        self.filter_tags.clear();
        self.invalidate_cache();
    }

    /// Get current status filter
    pub fn get_status_filter(&self) -> &Option<ProjectStatus> {
        &self.filter_status
    }

    /// Get current priority filter  
    pub fn get_priority_filter(&self) -> &Option<Priority> {
        &self.filter_priority
    }

    /// Get current tag filters
    pub fn get_tag_filters(&self) -> &Vec<String> {
        &self.filter_tags
    }

    /// Request a search update (may be debounced)
    pub fn request_search_update(&mut self) {
        self.pending_update = true;
        self.last_update_time = Some(Instant::now());
    }

    /// Check if enough time has passed to perform the search update
    pub fn should_update_search(&self) -> bool {
        if !self.pending_update {
            return false;
        }

        if let Some(last_time) = self.last_update_time {
            Instant::now().duration_since(last_time) >= self.debounce_duration
        } else {
            true
        }
    }

    /// Update search results based on current query and filters
    pub fn update_search(&mut self, projects: &[Project]) {
        let current_query = self.search_input.value().to_string();

        // Use cache if query hasn't changed and no pending update
        if self.use_cache && current_query == self.last_search_query && !self.pending_update {
            return;
        }

        self.perform_search_update(projects, current_query);
    }

    /// Force immediate search update (bypass debouncing)
    pub fn force_search_update(&mut self, projects: &[Project]) {
        let current_query = self.search_input.value().to_string();
        self.perform_search_update(projects, current_query);
    }

    /// Perform the actual search update
    fn perform_search_update(&mut self, projects: &[Project], current_query: String) {
        self.filtered_results.clear();
        self.selected_index = 0;

        // Apply text search and filters
        for (index, project) in projects.iter().enumerate() {
            if let Some(project_match) = self.match_project(index, project, &current_query) {
                self.filtered_results.push(project_match);
            }
        }

        // Sort by score (best matches first)
        self.filtered_results.sort_by(|a, b| b.score.cmp(&a.score));

        // Limit to max display items for performance
        self.filtered_results.truncate(self.max_display_items * 2);

        self.last_search_query = current_query;
        self.use_cache = true;
        self.pending_update = false;
    }

    /// Check if a project matches current search criteria
    fn match_project(&self, index: usize, project: &Project, query: &str) -> Option<ProjectMatch> {
        // Apply filters first (early exit if doesn't match filters)
        if !self.passes_filters(project) {
            return None;
        }

        // If no text query, include all projects that pass filters
        if query.is_empty() {
            return Some(ProjectMatch {
                project_index: index,
                project: project.clone(),
                score: 100, // Default score for filter-only matches
                match_indices: Vec::new(),
                matched_field: MatchedField::Name, // Default field
            });
        }

        // Try fuzzy matching on different fields with weights
        let mut best_match: Option<ProjectMatch> = None;

        // Search project name (highest weight)
        if let Some((score, indices)) = self
            .matcher
            .fuzzy_indices(&project.name.to_lowercase(), &query.to_lowercase())
        {
            let weighted_score = (score as f64 * 1.0) as i64; // Weight 1.0 for name
            if best_match
                .as_ref()
                .map_or(true, |m| weighted_score > m.score)
            {
                best_match = Some(ProjectMatch {
                    project_index: index,
                    project: project.clone(),
                    score: weighted_score,
                    match_indices: indices,
                    matched_field: MatchedField::Name,
                });
            }
        }

        // Search project path (medium weight)
        if let Some((score, indices)) = self
            .matcher
            .fuzzy_indices(&project.project_root.to_lowercase(), &query.to_lowercase())
        {
            let weighted_score = (score as f64 * 0.6) as i64; // Weight 0.6 for path
            if best_match
                .as_ref()
                .map_or(true, |m| weighted_score > m.score)
            {
                best_match = Some(ProjectMatch {
                    project_index: index,
                    project: project.clone(),
                    score: weighted_score,
                    match_indices: indices,
                    matched_field: MatchedField::Path,
                });
            }
        }

        // Search project description (lower weight)
        if let Some(description) = &project.description {
            if let Some((score, indices)) = self
                .matcher
                .fuzzy_indices(&description.to_lowercase(), &query.to_lowercase())
            {
                let weighted_score = (score as f64 * 0.4) as i64; // Weight 0.4 for description
                if best_match
                    .as_ref()
                    .map_or(true, |m| weighted_score > m.score)
                {
                    best_match = Some(ProjectMatch {
                        project_index: index,
                        project: project.clone(),
                        score: weighted_score,
                        match_indices: indices,
                        matched_field: MatchedField::Description,
                    });
                }
            }
        }

        // Search tags (high weight)
        if let Some(tags) = &project.tags {
            for tag in tags {
                if let Some((score, indices)) = self
                    .matcher
                    .fuzzy_indices(&tag.to_lowercase(), &query.to_lowercase())
                {
                    let weighted_score = (score as f64 * 0.8) as i64; // Weight 0.8 for tags
                    if best_match
                        .as_ref()
                        .map_or(true, |m| weighted_score > m.score)
                    {
                        best_match = Some(ProjectMatch {
                            project_index: index,
                            project: project.clone(),
                            score: weighted_score,
                            match_indices: indices,
                            matched_field: MatchedField::Tag(tag.clone()),
                        });
                    }
                }
            }
        }

        best_match
    }

    /// Check if project passes current filters
    fn passes_filters(&self, project: &Project) -> bool {
        // Status filter
        if let Some(filter_status) = &self.filter_status {
            if &project.status != filter_status {
                return false;
            }
        }

        // Priority filter
        if let Some(filter_priority) = &self.filter_priority {
            if &project.priority != filter_priority {
                return false;
            }
        }

        // Tag filters (project must have at least one of the filter tags)
        if !self.filter_tags.is_empty() {
            if let Some(project_tags) = &project.tags {
                let has_matching_tag = self
                    .filter_tags
                    .iter()
                    .any(|filter_tag| project_tags.contains(filter_tag));
                if !has_matching_tag {
                    return false;
                }
            } else {
                // No tags on project but we're filtering by tags
                return false;
            }
        }

        true
    }

    /// Move selection to previous result
    pub fn select_previous(&mut self) {
        if !self.filtered_results.is_empty() {
            self.selected_index = if self.selected_index > 0 {
                self.selected_index - 1
            } else {
                self.filtered_results.len() - 1
            };
        }
    }

    /// Move selection to next result
    pub fn select_next(&mut self) {
        if !self.filtered_results.is_empty() {
            self.selected_index = if self.selected_index < self.filtered_results.len() - 1 {
                self.selected_index + 1
            } else {
                0
            };
        }
    }

    /// Get visible results (limited for UI display)
    pub fn visible_results(&self) -> &[ProjectMatch] {
        let end_index = std::cmp::min(self.max_display_items, self.filtered_results.len());
        &self.filtered_results[..end_index]
    }

    /// Reset search state
    pub fn reset(&mut self) {
        self.search_input = Input::default();
        self.search_mode = SearchMode::Text;
        self.clear_filters();
        self.filtered_results.clear();
        self.selected_index = 0;
        self.last_search_query.clear();
        self.use_cache = false;
    }

    /// Invalidate cache to force refresh on next update
    fn invalidate_cache(&mut self) {
        self.use_cache = false;
        self.request_search_update();
    }

    /// Handle character input
    pub fn handle_char(&mut self, c: char) {
        self.search_input
            .handle(tui_input::InputRequest::InsertChar(c));
        self.invalidate_cache();
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        self.search_input
            .handle(tui_input::InputRequest::DeletePrevChar);
        self.invalidate_cache();
    }

    /// Handle delete
    pub fn handle_delete(&mut self) {
        self.search_input
            .handle(tui_input::InputRequest::DeleteNextChar);
        self.invalidate_cache();
    }

    /// Clear search input
    pub fn clear_input(&mut self) {
        self.search_input = Input::default();
        self.invalidate_cache();
    }
}
