use crate::state::{AppState, FocusArea};
use crate::ui::widgets::{ChatWidget, InputWidget};
use crate::ui::widgets::command_popup::{CommandPopupWidget, CommandHintWidget};
use crate::ui::widgets::{MentionPopupWidget, calculate_mention_popup_area};
use ratatui::prelude::*;

/// Render the chat interface
pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    
    // Create layout with chat area and input area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),         // Chat area (takes most space)
            Constraint::Length(3),      // Input area (fixed height)
        ])
        .split(area);
    
    // Render chat messages with focus information
    let chat_widget = ChatWidget::new(state.messages())
        .scroll_offset(state.scroll_offset())
        .show_timestamps(false)
        .focused(state.focus_area() == &FocusArea::Chat);
    
    frame.render_widget(chat_widget, chunks[0]);
    
    // Render input area with focus information
    let input_widget = InputWidget::new(state.input_buffer(), state.input_mode())
        .history_position(state.input_history_position())
        .placeholder("Type a message and press Enter...")
        .focused(state.focus_area() == &FocusArea::Input);
    
    frame.render_widget(input_widget, chunks[1]);
    
    // Render command popup overlay if in command mode
    if let Some(command_popup) = state.command_popup() {
        render_command_popup_overlay(frame, command_popup, chunks[1]);
    }
    
    // Render mention popup overlay if in mention mode
    if let Some(mention_popup) = state.mention_popup() {
        render_mention_popup_overlay(frame, mention_popup, chunks[1]);
    }
}

/// Render the command popup as an overlay above the input area
fn render_command_popup_overlay(frame: &mut Frame, popup: &crate::command_popup::CommandPopup, input_area: Rect) {
    let area = frame.area();
    
    // Calculate popup position (above the input area)
    let popup_height = (popup.result_count() as u16).min(8).max(1) + 2; // +2 for borders
    let popup_width = area.width.saturating_sub(4).min(80); // Leave some margin
    
    // Position popup above input area
    let popup_y = input_area.y.saturating_sub(popup_height + 1); // +1 for spacing
    let popup_x = area.x + 2; // Small left margin
    
    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };
    
    // Ensure popup doesn't go off screen
    if popup_y < area.y {
        // If no room above, show below input
        let below_popup_area = Rect {
            x: popup_x,
            y: input_area.y + input_area.height + 1,
            width: popup_width,
            height: popup_height.min(area.height.saturating_sub(input_area.y + input_area.height + 1)),
        };
        
        let popup_widget = CommandPopupWidget::new(popup).max_rows(popup_height.saturating_sub(2));
        frame.render_widget(popup_widget, below_popup_area);
        
        // Render hint below the popup if there's room
        let hint_y = below_popup_area.y + below_popup_area.height;
        if hint_y < area.y + area.height {
            let hint_area = Rect {
                x: popup_x,
                y: hint_y,
                width: popup_width,
                height: 1,
            };
            let hint_widget = CommandHintWidget::new(popup);
            frame.render_widget(hint_widget, hint_area);
        }
    } else {
        // Standard position above input
        let popup_widget = CommandPopupWidget::new(popup).max_rows(popup_height.saturating_sub(2));
        frame.render_widget(popup_widget, popup_area);
        
        // Render hint below the popup
        let hint_area = Rect {
            x: popup_x,
            y: popup_area.y + popup_area.height,
            width: popup_width,
            height: 1,
        };
        let hint_widget = CommandHintWidget::new(popup);
        frame.render_widget(hint_widget, hint_area);
    }
}

/// Render the mention popup as an overlay above the input area
fn render_mention_popup_overlay(frame: &mut Frame, popup: &crate::mention_popup::MentionPopup, input_area: Rect) {
    let area = frame.area();
    
    // Calculate popup position using the helper function
    let popup_area = calculate_mention_popup_area(area, popup);
    
    // Adjust if popup would overlap with input area or go off screen
    let final_popup_area = if popup_area.y + popup_area.height > input_area.y {
        // Popup would overlap with input, position it above with safe margin
        let safe_height = input_area.y.saturating_sub(area.y + 1); // +1 for margin
        let adjusted_height = popup_area.height.min(safe_height);
        
        Rect {
            x: popup_area.x,
            y: input_area.y.saturating_sub(adjusted_height + 1),
            width: popup_area.width,
            height: adjusted_height,
        }
    } else {
        popup_area
    };
    
    // Only render if we have space
    if final_popup_area.height >= 3 { // Need at least space for borders + 1 item
        let max_items = final_popup_area.height.saturating_sub(2); // -2 for borders
        let popup_widget = MentionPopupWidget::new(popup).max_rows(max_items);
        frame.render_widget(popup_widget, final_popup_area);
    }
}