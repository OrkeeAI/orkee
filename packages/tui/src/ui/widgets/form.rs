use crossterm::event::Event;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use std::collections::HashMap;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;
use tui_textarea::TextArea;

/// A form widget for collecting structured input data
#[derive(Debug)]
pub struct FormWidget {
    pub steps: Vec<FormStep>,
    pub current_step: usize,
    pub current_field: usize,
    pub validation_errors: HashMap<String, String>,
    pub title: String,
    pub is_edit_mode: bool,
}

/// A single step in a multi-step form
#[derive(Debug)]
pub struct FormStep {
    pub title: String,
    pub fields: Vec<FormField>,
}

impl FormStep {
    /// Create a new form step
    pub fn new(title: String) -> Self {
        Self {
            title,
            fields: Vec::new(),
        }
    }

    /// Add a field to this step
    pub fn add_field(&mut self, field: FormField) {
        self.fields.push(field);
    }
}

/// Enum to hold different types of input widgets
pub enum FieldValue {
    SingleLine(Input),
    MultiLine(TextArea<'static>),
    Selection {
        options: Vec<String>,
        selected: usize,
    },
}

impl std::fmt::Debug for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::SingleLine(input) => f
                .debug_struct("SingleLine")
                .field("value", &input.value())
                .finish(),
            FieldValue::MultiLine(textarea) => f
                .debug_struct("MultiLine")
                .field("lines", &textarea.lines().len())
                .finish(),
            FieldValue::Selection { options, selected } => f
                .debug_struct("Selection")
                .field("options", options)
                .field("selected", selected)
                .finish(),
        }
    }
}

/// A single field in a form
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_value: FieldValue,
    pub field_type: FieldType,
    pub required: bool,
    pub validator: Option<fn(&str) -> Result<(), String>>,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
}

impl std::fmt::Debug for FormField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormField")
            .field("name", &self.name)
            .field("label", &self.label)
            .field("field_value", &self.field_value)
            .field("field_type", &self.field_type)
            .field("required", &self.required)
            .field("validator", &self.validator.map(|_| "<function>"))
            .field("placeholder", &self.placeholder)
            .field("help_text", &self.help_text)
            .finish()
    }
}

impl FieldValue {
    /// Get the string value of this field
    pub fn value(&self) -> String {
        match self {
            FieldValue::SingleLine(input) => input.value().to_string(),
            FieldValue::MultiLine(textarea) => textarea.lines().join("\n"),
            FieldValue::Selection { options, selected } => {
                options.get(*selected).cloned().unwrap_or_default()
            }
        }
    }

    /// Check if this field is empty
    pub fn is_empty(&self) -> bool {
        match self {
            FieldValue::SingleLine(input) => input.value().is_empty(),
            FieldValue::MultiLine(textarea) => {
                textarea.lines().iter().all(|line| line.trim().is_empty())
            }
            FieldValue::Selection { options, selected } => {
                options.get(*selected).map_or(true, |s| s.is_empty())
            }
        }
    }
}

/// Types of form fields supported
#[derive(Debug, Clone)]
pub enum FieldType {
    Text,
    Path,
    MultilineText,
    Selection(Vec<String>),
    Tags,
}

impl FormWidget {
    /// Create a new multi-step form widget
    pub fn new(title: String) -> Self {
        Self {
            steps: Vec::new(),
            current_step: 0,
            current_field: 0,
            validation_errors: HashMap::new(),
            title,
            is_edit_mode: false,
        }
    }

    /// Create a new multi-step form widget for editing
    pub fn new_for_edit(title: String) -> Self {
        Self {
            steps: Vec::new(),
            current_step: 0,
            current_field: 0,
            validation_errors: HashMap::new(),
            title,
            is_edit_mode: true,
        }
    }

    /// Add a step to the form
    pub fn add_step(&mut self, step: FormStep) {
        self.steps.push(step);
    }

    /// Get the total number of steps
    pub fn total_steps(&self) -> usize {
        self.steps.len()
    }

    /// Get the current step (1-indexed)
    pub fn current_step_number(&self) -> usize {
        self.current_step + 1
    }

    /// Get current step's fields
    pub fn current_step_fields(&self) -> Option<&[FormField]> {
        self.steps
            .get(self.current_step)
            .map(|step| step.fields.as_slice())
    }

    /// Get current step's fields mutably
    pub fn current_step_fields_mut(&mut self) -> Option<&mut [FormField]> {
        self.steps
            .get_mut(self.current_step)
            .map(|step| step.fields.as_mut_slice())
    }

    /// Move to the next field within current step
    pub fn next_field(&mut self) -> bool {
        if let Some(fields) = self.current_step_fields() {
            if self.current_field < fields.len().saturating_sub(1) {
                self.current_field += 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Move to the previous field within current step
    pub fn previous_field(&mut self) -> bool {
        if self.current_field > 0 {
            self.current_field -= 1;
            true
        } else {
            false
        }
    }

    /// Move to the next step
    pub fn next_step(&mut self) -> bool {
        if self.current_step < self.steps.len().saturating_sub(1) {
            self.current_step += 1;
            self.current_field = 0; // Reset field index for new step
            true
        } else {
            false
        }
    }

    /// Move to the previous step
    pub fn previous_step(&mut self) -> bool {
        if self.current_step > 0 {
            self.current_step -= 1;
            self.current_field = 0; // Reset field index for new step
            true
        } else {
            false
        }
    }

    /// Get the currently selected field
    pub fn current_field(&self) -> Option<&FormField> {
        self.current_step_fields()?.get(self.current_field)
    }

    /// Get the currently selected field mutably
    pub fn current_field_mut(&mut self) -> Option<&mut FormField> {
        let current_field_idx = self.current_field;
        self.current_step_fields_mut()?.get_mut(current_field_idx)
    }

    /// Handle input for the current field (route to appropriate widget)
    pub fn handle_input(&mut self, event: &Event) -> bool {
        // Get field info first to avoid borrowing conflicts
        let current_field_idx = self.current_field;
        let current_step_idx = self.current_step;

        if let Some(step) = self.steps.get_mut(current_step_idx) {
            if let Some(field) = step.fields.get_mut(current_field_idx) {
                let field_name = field.name.clone(); // Clone name to avoid borrow issues

                let handled = match &mut field.field_value {
                    FieldValue::SingleLine(input) => {
                        // Use tui-input's handle_event method directly
                        input.handle_event(event).is_some()
                    }
                    FieldValue::MultiLine(textarea) => {
                        // For textarea, we need to convert crossterm KeyEvent to ratatui KeyEvent
                        if let Event::Key(key_event) = event {
                            // Block Enter key handling - let form navigation handle it
                            if key_event.code == crossterm::event::KeyCode::Enter {
                                return false;
                            }

                            // Map crossterm types to ratatui types manually (version mismatch)
                            let ratatui_code = match key_event.code {
                                crossterm::event::KeyCode::Backspace => {
                                    ratatui::crossterm::event::KeyCode::Backspace
                                }
                                crossterm::event::KeyCode::Left => {
                                    ratatui::crossterm::event::KeyCode::Left
                                }
                                crossterm::event::KeyCode::Right => {
                                    ratatui::crossterm::event::KeyCode::Right
                                }
                                crossterm::event::KeyCode::Up => {
                                    ratatui::crossterm::event::KeyCode::Up
                                }
                                crossterm::event::KeyCode::Down => {
                                    ratatui::crossterm::event::KeyCode::Down
                                }
                                crossterm::event::KeyCode::Home => {
                                    ratatui::crossterm::event::KeyCode::Home
                                }
                                crossterm::event::KeyCode::End => {
                                    ratatui::crossterm::event::KeyCode::End
                                }
                                crossterm::event::KeyCode::PageUp => {
                                    ratatui::crossterm::event::KeyCode::PageUp
                                }
                                crossterm::event::KeyCode::PageDown => {
                                    ratatui::crossterm::event::KeyCode::PageDown
                                }
                                crossterm::event::KeyCode::Tab => {
                                    ratatui::crossterm::event::KeyCode::Tab
                                }
                                crossterm::event::KeyCode::Delete => {
                                    ratatui::crossterm::event::KeyCode::Delete
                                }
                                crossterm::event::KeyCode::Char(c) => {
                                    ratatui::crossterm::event::KeyCode::Char(c)
                                }
                                _ => return false, // Not handled
                            };

                            let ratatui_modifiers =
                                ratatui::crossterm::event::KeyModifiers::empty();

                            let ratatui_key_event = ratatui::crossterm::event::KeyEvent::new(
                                ratatui_code,
                                ratatui_modifiers,
                            );
                            textarea.input(ratatui_key_event)
                        } else {
                            false
                        }
                    }
                    FieldValue::Selection { options, selected } => {
                        // Handle selection navigation with arrow keys
                        if let Event::Key(key_event) = event {
                            match key_event.code {
                                crossterm::event::KeyCode::Up => {
                                    if *selected > 0 {
                                        *selected -= 1;
                                        true
                                    } else {
                                        false
                                    }
                                }
                                crossterm::event::KeyCode::Down => {
                                    if *selected < options.len().saturating_sub(1) {
                                        *selected += 1;
                                        true
                                    } else {
                                        false
                                    }
                                }
                                crossterm::event::KeyCode::Home => {
                                    *selected = 0;
                                    true
                                }
                                crossterm::event::KeyCode::End => {
                                    *selected = options.len().saturating_sub(1);
                                    true
                                }
                                _ => false,
                            }
                        } else {
                            false
                        }
                    }
                };

                if handled {
                    // Clear validation error when field is updated
                    self.validation_errors.remove(&field_name);
                }

                handled
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Insert a newline character into the current multiline field
    pub fn insert_newline(&mut self) -> bool {
        // Get field info first to avoid borrowing conflicts
        let current_field_idx = self.current_field;
        let current_step_idx = self.current_step;

        if let Some(step) = self.steps.get_mut(current_step_idx) {
            if let Some(field) = step.fields.get_mut(current_field_idx) {
                match &mut field.field_value {
                    FieldValue::MultiLine(textarea) => {
                        // Insert newline directly into textarea
                        let ratatui_key_event = ratatui::crossterm::event::KeyEvent::new(
                            ratatui::crossterm::event::KeyCode::Enter,
                            ratatui::crossterm::event::KeyModifiers::empty(),
                        );
                        textarea.input(ratatui_key_event);

                        // Clear validation error when field is updated
                        self.validation_errors.remove(&field.name);
                        true
                    }
                    _ => false, // Only works for multiline fields
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Validate the current field
    pub fn validate_current_field(&mut self) -> bool {
        // Get field info without borrowing self
        let current_field_idx = self.current_field;
        let current_step_idx = self.current_step;

        if let Some(step) = self.steps.get(current_step_idx) {
            if let Some(field) = step.fields.get(current_field_idx) {
                let field_name = field.name.clone();
                let field_value_str = field.field_value.value();
                let is_required = field.required;

                // Check if required field is empty
                if is_required && field_value_str.trim().is_empty() {
                    self.validation_errors
                        .insert(field_name.clone(), "This field is required".to_string());
                    return false;
                }

                // Run custom validator if present
                if let Some(validator) = field.validator {
                    if let Err(error) = validator(&field_value_str) {
                        self.validation_errors.insert(field_name.clone(), error);
                        return false;
                    }
                }

                // Clear any previous error for this field
                self.validation_errors.remove(&field_name);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Validate all fields in the current step
    pub fn validate_current_step(&mut self) -> bool {
        let step_fields_len = self
            .current_step_fields()
            .map(|fields| fields.len())
            .unwrap_or(0);
        let mut all_valid = true;
        for i in 0..step_fields_len {
            let old_current = self.current_field;
            self.current_field = i;
            if !self.validate_current_field() {
                all_valid = false;
            }
            self.current_field = old_current;
        }
        all_valid
    }

    /// Validate all fields across all steps
    pub fn validate_all_steps(&mut self) -> bool {
        let mut all_valid = true;
        let old_step = self.current_step;
        let old_field = self.current_field;

        for step_idx in 0..self.steps.len() {
            self.current_step = step_idx;
            if !self.validate_current_step() {
                all_valid = false;
            }
        }

        self.current_step = old_step;
        self.current_field = old_field;
        all_valid
    }

    /// Check if the current step is complete and valid
    pub fn is_current_step_complete(&self) -> bool {
        if let Some(fields) = self.current_step_fields() {
            // All required fields in current step must have values
            for field in fields {
                if field.required && field.field_value.value().trim().is_empty() {
                    return false;
                }
            }

            // No validation errors for current step fields
            for field in fields {
                if self.validation_errors.contains_key(&field.name) {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    /// Check if all steps are complete and valid
    pub fn is_all_steps_complete(&self) -> bool {
        for step in &self.steps {
            // All required fields must have values
            for field in &step.fields {
                if field.required && field.field_value.value().trim().is_empty() {
                    return false;
                }
            }

            // No validation errors
            for field in &step.fields {
                if self.validation_errors.contains_key(&field.name) {
                    return false;
                }
            }
        }
        true
    }

    /// Check if the current field is multiline
    pub fn current_field_is_multiline(&self) -> bool {
        if let Some(field) = self.current_field() {
            matches!(field.field_value, FieldValue::MultiLine(_))
        } else {
            false
        }
    }

    /// Check if we're currently on the review step (last step)
    pub fn is_review_step(&self) -> bool {
        self.current_step == self.steps.len().saturating_sub(1) && !self.steps.is_empty()
    }

    /// Render a summary of all collected field values for review
    fn render_review_summary(&self, frame: &mut Frame, area: Rect) {
        let mut summary_lines = Vec::new();
        summary_lines.push("📋 Project Summary:".to_string());
        summary_lines.push("".to_string());

        // Collect all field values from all steps
        for (step_idx, step) in self.steps.iter().enumerate() {
            if step_idx == self.current_step {
                continue; // Skip the review step itself
            }

            summary_lines.push(format!("▸ {}", step.title));
            for field in &step.fields {
                let value = field.field_value.value();
                if !value.trim().is_empty() {
                    // Format multiline values nicely
                    if value.contains('\n') {
                        summary_lines.push(format!("  {}: ", field.label));
                        for line in value.lines() {
                            summary_lines.push(format!("    {}", line));
                        }
                    } else {
                        summary_lines.push(format!("  {}: {}", field.label, value));
                    }
                } else if field.required {
                    summary_lines.push(format!("  {}: (required - not set)", field.label));
                }
            }
            summary_lines.push("".to_string());
        }

        summary_lines.push("".to_string());
        let action_text = if self.is_edit_mode {
            "✅ Ready to update project? Press Enter to confirm, Esc to go back, or Shift+C to cancel."
        } else {
            "✅ Ready to create project? Press Enter to confirm, Esc to go back, or Shift+C to cancel."
        };
        summary_lines.push(action_text.to_string());

        // Create a paragraph with all the summary text
        let summary_text = summary_lines.join("\n");
        let summary_paragraph = Paragraph::new(summary_text)
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((0, 0));

        frame.render_widget(summary_paragraph, area);
    }

    /// Render the form widget
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Clear the area
        frame.render_widget(Clear, area);

        // Get current step information
        let current_step = self.steps.get(self.current_step);
        let step_title = current_step.map(|s| s.title.as_str()).unwrap_or("Unknown");
        let title = format!(
            "{} - {} (Step {}/{})",
            self.title,
            step_title,
            self.current_step_number(),
            self.total_steps()
        );

        let block = Block::default()
            .title(title)
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Handle special case for review step
        if self.is_review_step() {
            self.render_review_summary(frame, inner);
            self.render_help_text(frame, area);
            return;
        }

        // Get current step's fields
        let fields = match self.current_step_fields() {
            Some(fields) => fields,
            None => return,
        };

        if fields.is_empty() {
            return;
        }

        // Calculate actual height for each field
        let field_heights: Vec<u16> = fields
            .iter()
            .map(|field| self.calculate_field_height(field))
            .collect();

        // Determine which fields can fit in available space
        let available_height = inner.height as u16;
        let mut total_height = 0u16;
        let mut end_field = 0;

        // Start from current field and work outward to ensure current field is always visible
        for i in self.current_field..fields.len() {
            let field_height = field_heights[i];
            if total_height + field_height <= available_height {
                total_height += field_height;
                end_field = i + 1;
            } else {
                break;
            }
        }

        // If we have room, try to include fields before the current field
        let mut start_field = self.current_field;
        while start_field > 0 {
            let prev_field_idx = start_field - 1;
            let field_height = field_heights[prev_field_idx];
            if total_height + field_height <= available_height {
                total_height += field_height;
                start_field = prev_field_idx;
            } else {
                break;
            }
        }

        // Create constraints for visible fields
        let field_constraints: Vec<Constraint> = (start_field..end_field)
            .map(|i| Constraint::Length(field_heights[i]))
            .collect();

        if field_constraints.is_empty() {
            return;
        }

        let field_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(field_constraints)
            .split(inner);

        // Render each visible field
        for (i, field_index) in (start_field..end_field).enumerate() {
            if let Some(field) = fields.get(field_index) {
                self.render_field(
                    frame,
                    field_areas[i],
                    field,
                    field_index == self.current_field,
                );
            }
        }

        // Render help text at the bottom
        self.render_help_text(frame, area);
    }

    /// Render a single field
    fn render_field(&self, frame: &mut Frame, area: Rect, field: &FormField, is_current: bool) {
        // Render label first
        let required_indicator = if field.required { " *" } else { "" };
        let label = format!("{}{}: ", field.label, required_indicator);
        let label_style = if is_current {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        // Choose layout based on field type
        let (label_area, input_area, error_area) = match &field.field_value {
            FieldValue::MultiLine(textarea) => {
                // Calculate dynamic height for textarea based on content (like chat input)
                let textarea_lines = textarea.lines();
                let line_count = if textarea_lines.is_empty()
                    || (textarea_lines.len() == 1 && textarea_lines[0].is_empty())
                {
                    1 // At least 1 line for empty textarea
                } else {
                    textarea_lines.len()
                };
                let textarea_height = (line_count as u16).min(10) + 2; // +2 for borders, max 10 lines of content

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),               // Label
                        Constraint::Length(textarea_height), // Textarea (dynamic height)
                        Constraint::Length(1),               // Error/spacing
                    ])
                    .split(area);
                (chunks[0], chunks[1], chunks[2])
            }
            FieldValue::SingleLine(_) => {
                // Single line layout - need 3 lines for bordered input
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1), // Label
                        Constraint::Length(3), // Input (with borders)
                        Constraint::Length(1), // Error/spacing
                    ])
                    .split(area);
                (chunks[0], chunks[1], chunks[2])
            }
            FieldValue::Selection { options, .. } => {
                // Selection layout - need height for all radio button options + borders
                let selection_height = options.len() as u16 + 2; // +2 for borders

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),                // Label
                        Constraint::Length(selection_height), // Selection options
                        Constraint::Length(1),                // Error/spacing
                    ])
                    .split(area);
                (chunks[0], chunks[1], chunks[2])
            }
        };

        // Render label
        let label_paragraph = Paragraph::new(label).style(label_style);
        frame.render_widget(label_paragraph, label_area);

        // Render the input using the widget's built-in render method
        match &field.field_value {
            FieldValue::SingleLine(input) => {
                // For tui-input, we need to create a Paragraph with the input's value
                let input_style = if is_current {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                };

                // Calculate scroll for long input values
                let width = input_area.width.max(3).saturating_sub(2) as usize;
                let scroll = input.visual_scroll(width);

                // Create a paragraph with the input value
                let input_value = input.value();
                let display_value = if input_value.is_empty() {
                    field
                        .placeholder
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or(input_value)
                } else {
                    input_value
                };

                let input_paragraph =
                    Paragraph::new(display_value)
                        .style(input_style)
                        .scroll((0, scroll as u16))
                        .block(Block::default().borders(Borders::ALL).border_style(
                            if is_current {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default().fg(Color::Gray)
                            },
                        ));

                frame.render_widget(input_paragraph, input_area);

                // Set cursor position if this field is current
                if is_current {
                    let cursor_x = input.visual_cursor().max(scroll);
                    // Position cursor inside the bordered block (add 1 for border, then cursor position)
                    frame.set_cursor_position((
                        input_area.x + 1 + cursor_x as u16,
                        input_area.y + 1,
                    ));
                }
            }
            FieldValue::MultiLine(textarea) => {
                // Create a block for the textarea
                let textarea_block =
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(if is_current {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::Gray)
                        });

                // Render the block first
                frame.render_widget(&textarea_block, input_area);

                // Get the inner area for the textarea content
                let inner_area = textarea_block.inner(input_area);

                // Render the textarea inside the block
                frame.render_widget(textarea, inner_area);
            }
            FieldValue::Selection { options, selected } => {
                // Create a block for the selection
                let selection_block = Block::default()
                    .borders(Borders::ALL)
                    .title("Select an option")
                    .border_style(if is_current {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Gray)
                    });

                // Render the block first
                frame.render_widget(&selection_block, input_area);

                // Get the inner area for the options
                let inner_area = selection_block.inner(input_area);

                // Create radio button options
                let mut option_lines = Vec::new();
                for (i, option) in options.iter().enumerate() {
                    let radio_symbol = if i == *selected { "●" } else { "○" };
                    let option_style = if i == *selected {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let line = Line::from(vec![
                        Span::styled(format!("{} ", radio_symbol), option_style),
                        Span::styled(option.clone(), option_style),
                    ]);
                    option_lines.push(line);
                }

                // Render the options
                let options_paragraph = Paragraph::new(option_lines);
                frame.render_widget(options_paragraph, inner_area);
            }
        }

        // Render error or help text
        if let Some(error) = self.validation_errors.get(&field.name) {
            let error_text = format!("❌ {}", error);
            let error_paragraph = Paragraph::new(error_text).style(Style::default().fg(Color::Red));
            frame.render_widget(error_paragraph, error_area);
        } else if is_current {
            if let Some(ref help) = field.help_text {
                let help_text = format!("💡 {}", help);
                let help_paragraph =
                    Paragraph::new(help_text).style(Style::default().fg(Color::Gray));
                frame.render_widget(help_paragraph, error_area);
            }
        }
    }

    /// Calculate the actual height needed for a field
    fn calculate_field_height(&self, field: &FormField) -> u16 {
        match &field.field_value {
            FieldValue::MultiLine(textarea) => {
                // Calculate dynamic height for textarea based on content
                let textarea_lines = textarea.lines();
                let line_count = if textarea_lines.is_empty()
                    || (textarea_lines.len() == 1 && textarea_lines[0].is_empty())
                {
                    1 // At least 1 line for empty textarea
                } else {
                    textarea_lines.len()
                };
                let textarea_height = (line_count as u16).min(10) + 2; // +2 for borders, max 10 lines
                1 + textarea_height + 1 // Label + textarea + error/spacing
            }
            FieldValue::SingleLine(_) => {
                5 // Label + bordered input (3 lines) + error/spacing
            }
            FieldValue::Selection { options, .. } => {
                let selection_height = options.len() as u16 + 2; // +2 for borders
                1 + selection_height + 1 // Label + selection options + error/spacing
            }
        }
    }

    /// Render help text at the bottom of the form
    fn render_help_text(&self, frame: &mut Frame, area: Rect) {
        let help_text = if self.is_review_step() {
            if self.is_edit_mode {
                "Enter: Confirm & Update Project • Esc: Back to Edit • Shift+C: Cancel"
            } else {
                "Enter: Confirm & Add Project • Esc: Back to Edit • Shift+C: Cancel"
            }
        } else if self.current_field_is_multiline() {
            if self.is_current_step_complete() {
                "Enter/↓/Tab: Continue • ↑/Shift+Tab: Previous • Shift+Enter: New Line • Esc: Cancel"
            } else {
                "Enter/↓/Tab: Next • ↑/Shift+Tab: Previous • Shift+Enter: New Line • Esc: Cancel"
            }
        } else if self.is_current_step_complete() {
            "Enter/↓/Tab: Continue • ↑/Shift+Tab: Previous • Esc: Cancel"
        } else {
            "Enter/↓/Tab: Next • ↑/Shift+Tab: Previous • Esc: Cancel"
        };

        let help_area = Rect {
            x: area.x + 1,
            y: area.y + area.height - 2,
            width: area.width - 2,
            height: 1,
        };

        let help_paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help_paragraph, help_area);
    }
}

impl FormField {
    /// Create a new text field
    pub fn text(name: String, label: String, required: bool) -> Self {
        let input = Input::default();
        // Set placeholder if we have one later

        Self {
            name,
            label,
            field_value: FieldValue::SingleLine(input),
            field_type: FieldType::Text,
            required,
            validator: None,
            placeholder: None,
            help_text: Some("Enter/↓/Tab to continue, ↑ to go back".to_string()),
        }
    }

    /// Create a new path field
    pub fn path(name: String, label: String, required: bool) -> Self {
        let input = Input::default();

        Self {
            name,
            label,
            field_value: FieldValue::SingleLine(input),
            field_type: FieldType::Path,
            required,
            validator: None,
            placeholder: Some("/path/to/project".to_string()),
            help_text: Some("Enter the full path to the project directory • Enter/↓/Tab to continue, ↑ to go back".to_string()),
        }
    }

    /// Create a new multiline text field
    pub fn multiline_text(name: String, label: String, required: bool) -> Self {
        let mut textarea = TextArea::default();
        // Remove the default underline cursor line style
        textarea.set_cursor_line_style(Style::default());

        Self {
            name,
            label,
            field_value: FieldValue::MultiLine(textarea),
            field_type: FieldType::MultilineText,
            required,
            validator: None,
            placeholder: Some("Enter description...".to_string()),
            help_text: Some(
                "Use Shift+Enter for new lines, Enter/↓/Tab to continue, ↑ to go back".to_string(),
            ),
        }
    }

    /// Create a new selection field
    pub fn selection(name: String, label: String, options: Vec<String>, required: bool) -> Self {
        let selected = 0; // Default to first option

        Self {
            name,
            label,
            field_value: FieldValue::Selection {
                options: options.clone(),
                selected,
            },
            field_type: FieldType::Selection(options),
            required,
            validator: None,
            placeholder: None,
            help_text: Some(
                "Use ↑↓ arrow keys to select, Enter/↓/Tab to continue, ↑ to go back".to_string(),
            ),
        }
    }

    /// Create a new tags field
    pub fn tags(name: String, label: String, required: bool) -> Self {
        let input = Input::default();

        Self {
            name,
            label,
            field_value: FieldValue::SingleLine(input),
            field_type: FieldType::Tags,
            required,
            validator: None,
            placeholder: Some("tag1, tag2, tag3".to_string()),
            help_text: Some(
                "Separate tags with commas • Enter/↓/Tab to continue, ↑ to go back".to_string(),
            ),
        }
    }

    /// Set a custom validator for this field
    pub fn with_validator(mut self, validator: fn(&str) -> Result<(), String>) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Set a placeholder for this field
    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    /// Set help text for this field
    pub fn with_help_text(mut self, help_text: String) -> Self {
        self.help_text = Some(help_text);
        self
    }
}
