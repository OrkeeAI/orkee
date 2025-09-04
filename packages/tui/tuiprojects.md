# TUI Project Management Implementation Plan

## Overview

This document outlines the comprehensive implementation plan for adding interactive project management capabilities to the Orkee TUI. The goal is to provide a rich, terminal-based interface for managing projects that matches and exceeds the CLI functionality with better usability and visual feedback.

## Current Architecture

### Core Components
- **TUI Package**: `packages/tui/` - Ratatui-based terminal interface
- **Projects Library**: `packages/projects/` - Shared Rust library for project CRUD operations
- **Direct Integration**: TUI uses projects library directly (no HTTP client needed)

### Key Files
- `src/app.rs` - Main application loop and event handling
- `src/state.rs` - Application state management
- `src/ui/projects.rs` - Projects screen rendering
- `src/ui/mod.rs` - UI routing and main render function
- `src/input/mod.rs` - Input handling and modes

## Phase 1: Interactive Project List ✅ COMPLETED

### Features Implemented
- ✅ Arrow key navigation (↑↓) in project list
- ✅ Enter key to view project details
- ✅ Escape key to return from detail view
- ✅ Rich project display with colors and icons
- ✅ Project selection highlighting
- ✅ Context-aware keyboard shortcuts
- ✅ Comprehensive detail view with all project fields

### Technical Changes Made
```rust
// AppState extensions
pub selected_project: Option<usize>  // Current selection index
pub fn select_previous_project(&mut self) -> bool
pub fn select_next_project(&mut self) -> bool
pub fn get_selected_project(&self) -> Option<&Project>
pub fn view_selected_project_details(&mut self) -> bool
pub fn return_to_projects_list(&mut self)

// New screen type
Screen::ProjectDetail

// Enhanced keyboard handling
KeyCode::Up/Down => project navigation when on Projects screen
KeyCode::Enter => view details when project selected
KeyCode::Esc => return to list from detail view
```

## Phase 2: Interactive Project Creation

### Goal
Implement a multi-step form system for creating new projects with real-time validation and user-friendly input handling.

### Technical Components Needed

#### 2.1 Form Widget System
**File**: `src/ui/widgets/form.rs`
```rust
pub struct FormWidget {
    pub fields: Vec<FormField>,
    pub current_field: usize,
    pub validation_errors: HashMap<String, String>,
    pub title: String,
}

pub struct FormField {
    pub name: String,
    pub label: String,
    pub value: String,
    pub field_type: FieldType,
    pub required: bool,
    pub validator: Option<Box<dyn Fn(&str) -> Result<(), String>>>,
    pub placeholder: Option<String>,
}

pub enum FieldType {
    Text,
    Path,
    MultilineText,
    Selection(Vec<String>),
    Tags,
}
```

#### 2.2 Form State Management
**File**: `src/state.rs` - Extensions
```rust
pub enum InputMode {
    Normal,
    Command,
    Search,
    History,
    Edit,
    Form,  // New mode for form navigation
}

pub struct AppState {
    // ... existing fields
    pub form_state: Option<FormState>,
}

pub struct FormState {
    pub form: FormWidget,
    pub step: usize,
    pub total_steps: usize,
    pub can_submit: bool,
}
```

#### 2.3 New Project Creation Flow
**Steps:**
1. **Step 1: Basic Info**
   - Name (required, validated for uniqueness)
   - Project Path (required, validated for existence)
   - Description (optional)

2. **Step 2: Configuration**  
   - Status (Active/Inactive/Archived)
   - Priority (High/Medium/Low)
   - Tags (comma-separated)

3. **Step 3: Scripts**
   - Setup Script (optional)
   - Dev Script (optional)
   - Cleanup Script (optional)

4. **Step 4: Confirmation**
   - Review all fields
   - Confirm creation or go back to edit

#### 2.4 Validation System
```rust
// In validator functions
fn validate_project_name(name: &str, existing_projects: &[Project]) -> Result<(), String>
fn validate_project_path(path: &str) -> Result<(), String>  
fn validate_script_command(script: &str) -> Result<(), String>
```

#### 2.5 Form Navigation
```rust
// Keyboard handling in app.rs
KeyCode::Tab => next_form_field(),
KeyCode::BackTab => previous_form_field(),  
KeyCode::Enter => {
    if on_last_field && form_valid {
        submit_form()
    } else {
        next_form_field()
    }
}
KeyCode::Esc => cancel_form(),
```

### UI/UX Design

#### Form Layout
```
┌─ Create New Project (Step 1/4) ─────────────────┐
│                                                 │
│ Project Name: [my-new-project____________] *    │
│ Project Path: [/home/user/projects/______] *    │  
│ Description:  [Optional description_____]       │
│                                                 │
│ * Required fields                               │
│                                                 │
│ Tab: Next • Shift+Tab: Previous • Enter: Next  │
│ Esc: Cancel • F1: Help                         │
└─────────────────────────────────────────────────┘
```

#### Error Display
```
┌─ Create New Project (Step 1/4) ─────────────────┐
│                                                 │
│ Project Name: [my-new-project____________] *    │
│ ❌ Project with this name already exists        │
│                                                 │
│ Project Path: [/invalid/path/____________] *    │  
│ ❌ Path does not exist or is not accessible     │
│                                                 │
│ Description:  [Optional description_____]       │
└─────────────────────────────────────────────────┘
```

### Implementation Steps

1. **Create form widget system** (`ui/widgets/form.rs`)
2. **Extend AppState with form support** (`state.rs`)
3. **Add form rendering logic** (`ui/projects.rs`)
4. **Implement form navigation** (`app.rs`)
5. **Add real-time validation** 
6. **Integrate with projects library for creation**
7. **Add success/error feedback**

### Testing Strategy
- Unit tests for form validation logic
- Integration tests for form state management  
- Manual testing of complete creation workflow
- Edge case testing (invalid paths, duplicate names, etc.)

## Phase 3: Project Editing

### Goal
Enable editing of existing projects with the same form system, pre-populated with current values.

### Technical Components

#### 3.1 Edit Mode Detection
```rust
pub enum FormMode {
    Create,
    Edit(String), // Project ID being edited
}

pub struct FormState {
    pub mode: FormMode,
    pub form: FormWidget,
    // ... other fields
}
```

#### 3.2 Pre-population Logic
```rust
impl FormWidget {
    pub fn from_project(project: &Project) -> Self {
        // Create form with all fields pre-filled
    }
    
    pub fn to_project_update(&self) -> ProjectUpdateInput {
        // Convert form data to update input
    }
}
```

#### 3.3 Change Detection
```rust
pub struct FormField {
    // ... existing fields
    pub original_value: Option<String>,
    pub is_modified: bool,
}
```

### UI Enhancements
- Visual indication of modified fields
- "Unsaved changes" warning on cancel
- Side-by-side diff view (optional)

### Implementation Steps
1. **Extend form system for edit mode**
2. **Add change tracking**
3. **Implement pre-population from existing project**
4. **Add edit-specific validation (avoid duplicate names except current)**
5. **Integrate with projects library for updates**
6. **Add confirmation dialogs**

## Phase 4: Project Deletion

### Goal
Safe project deletion with confirmation and undo capability.

### Technical Components

#### 4.1 Confirmation Dialog
**File**: `src/ui/widgets/dialog.rs`
```rust
pub struct ConfirmationDialog {
    pub title: String,
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub dangerous: bool, // Red styling for destructive actions
}

pub enum DialogResult {
    Confirmed,
    Cancelled,
    Pending,
}
```

#### 4.2 Deletion State Management
```rust
pub struct AppState {
    // ... existing fields
    pub confirmation_dialog: Option<ConfirmationDialog>,
}

pub enum PendingAction {
    DeleteProject(String), // Project ID to delete
}
```

#### 4.3 Safety Features
- Two-step confirmation for destructive actions
- Display project details before deletion
- Optional backup/archive instead of permanent deletion
- Undo functionality (optional)

### UI Design
```
┌─ Delete Project ────────────────────────────────┐
│                                                 │
│ ⚠️  Are you sure you want to delete this        │
│     project?                                    │
│                                                 │
│ Name: "My Important Project"                    │
│ Path: /home/user/important-project              │
│                                                 │
│ ⚠️  This action cannot be undone!               │
│                                                 │
│          [Delete] [Cancel]                      │
│                                                 │
│ Enter: Confirm • Esc: Cancel • Tab: Navigate    │
└─────────────────────────────────────────────────┘
```

## Phase 5: Enhanced Navigation & Context

### Goal
Implement project context switching and enhanced navigation features.

### Features
- Current project indicator in status bar
- Project-specific command history
- Quick project switching (Ctrl+P popup)
- Recent projects list
- Project bookmarking

### Technical Components

#### 5.1 Project Context
```rust
pub struct AppState {
    // ... existing fields
    pub current_project_context: Option<String>, // Project ID
    pub recent_projects: Vec<String>, // Recently accessed project IDs
    pub project_bookmarks: Vec<String>, // Bookmarked project IDs
}
```

#### 5.2 Quick Switcher
```rust
pub struct ProjectSwitcher {
    pub projects: Vec<Project>,
    pub filtered_projects: Vec<Project>,
    pub selected_index: usize,
    pub filter: String,
}
```

#### 5.3 Status Bar Enhancement
- Current project name and status
- Unsaved changes indicator
- Background operation status

## Phase 6: Search and Filtering

### Goal
Advanced search and filtering capabilities for large project collections.

### Features
- Fuzzy search by name
- Filter by status, priority, tags
- Search in project paths and descriptions
- Saved search filters
- Search history

### Technical Components

#### 6.1 Search Engine
```rust
pub struct ProjectSearch {
    pub query: String,
    pub filters: SearchFilters,
    pub results: Vec<usize>, // Indices into projects array
}

pub struct SearchFilters {
    pub status: Option<ProjectStatus>,
    pub priority: Option<Priority>, 
    pub tags: Vec<String>,
    pub has_scripts: Option<bool>,
}
```

#### 6.2 Fuzzy Matching
- Integration with fuzzy matching library (e.g., `fuzzy-matcher`)
- Ranking based on relevance
- Highlighting of matched text

## Error Handling Strategy

### Categories
1. **Validation Errors**: User input validation, shown inline
2. **System Errors**: File system, permissions, shown in dialogs  
3. **Network Errors**: Not applicable (direct library integration)
4. **State Errors**: Inconsistent application state, logged and recovered

### Error Display
```rust
pub enum ErrorSeverity {
    Info,    // Blue, informational
    Warning, // Yellow, user should be aware
    Error,   // Red, action failed
    Fatal,   // Red, application may be unstable
}

pub struct ErrorMessage {
    pub severity: ErrorSeverity,
    pub title: String,
    pub message: String,
    pub details: Option<String>,
    pub actions: Vec<ErrorAction>,
}
```

## Testing Strategy

### Unit Tests
- Form validation logic
- State management functions  
- Search and filter algorithms
- Error handling edge cases

### Integration Tests
- Complete workflows (create, edit, delete)
- Form navigation and validation
- Project switching and context
- Error recovery scenarios

### Manual Testing Checklist
- [ ] Create project with all field combinations
- [ ] Edit project with various changes
- [ ] Delete project with confirmation
- [ ] Navigate between projects seamlessly
- [ ] Search and filter projects effectively
- [ ] Handle errors gracefully
- [ ] Keyboard shortcuts work consistently
- [ ] Visual feedback is clear and helpful

## Performance Considerations

### Optimizations
- Lazy loading of project details
- Efficient list rendering for large collections
- Debounced search input
- Minimal redraws on state changes

### Memory Management
- Limit search result cache size
- Clean up unused form states
- Efficient string handling for large descriptions

## Accessibility Features

### Keyboard Navigation
- Full keyboard support (no mouse required)
- Logical tab order
- Standard shortcuts (Ctrl+C, Ctrl+V where applicable)
- Help system (F1 key)

### Visual Design
- High contrast color scheme
- Clear focus indicators
- Consistent iconography
- Readable fonts and spacing

## Future Enhancements

### Advanced Features
- Project templates
- Bulk operations (multi-select)
- Import/export functionality
- Project statistics and analytics
- Integration with external tools (Git, IDEs)

### UI/UX Improvements
- Customizable themes
- Layout preferences
- Keyboard shortcut customization
- Plugin system for extensions

## File Structure

```
packages/tui/src/
├── app.rs                    # Main application loop
├── state.rs                  # Application state management
├── events.rs                 # Event handling
├── input/
│   ├── mod.rs               # Input modes and buffer
│   ├── buffer.rs            # Text input buffer
│   └── history.rs           # Input history
├── ui/
│   ├── mod.rs               # UI routing
│   ├── projects.rs          # Projects screen and details
│   ├── dashboard.rs         # Dashboard screen
│   ├── chat.rs              # Chat interface
│   └── widgets/
│       ├── mod.rs           # Widget exports
│       ├── form.rs          # Form widget system
│       ├── dialog.rs        # Confirmation dialogs
│       ├── search.rs        # Search/filter widgets
│       └── status.rs        # Status bar widget
├── chat/
│   ├── mod.rs               # Chat functionality
│   └── history.rs           # Message history
├── command_popup.rs         # Slash command popup
├── mention_popup.rs         # @ mention popup
├── slash_command.rs         # Command definitions
└── lib.rs                   # Library entry point
```

## Implementation Priority

### High Priority (Core Functionality)
1. ✅ Phase 1: Interactive Project List
2. 🔄 Phase 2: Project Creation Forms
3. 🔄 Phase 3: Project Editing
4. 🔄 Phase 4: Project Deletion

### Medium Priority (Enhanced UX)
5. Phase 5: Navigation & Context
6. Phase 6: Search & Filtering

### Low Priority (Polish & Features)
7. Advanced error handling
8. Performance optimizations
9. Accessibility improvements
10. Future enhancements

## Success Criteria

### Functional Requirements
- ✅ Users can view all projects in an organized list
- 🔄 Users can create new projects through guided forms
- 🔄 Users can edit existing projects with validation
- 🔄 Users can safely delete projects with confirmation
- 🔄 Users can quickly find projects through search
- 🔄 All operations provide clear feedback

### Non-Functional Requirements
- Interface is responsive and fast
- No data loss during operations
- Graceful error handling and recovery
- Intuitive keyboard navigation
- Consistent visual design
- Reliable state management

---

*This document serves as the comprehensive implementation guide for TUI project management. Update as development progresses and requirements evolve.*