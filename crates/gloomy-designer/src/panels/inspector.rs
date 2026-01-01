//! Property inspector panel.
//!
//! Displays and allows editing of the selected widget's properties.

use gloomy_core::layout::{Direction, Layout};
use gloomy_core::widget::{Widget, WidgetBounds};

/// Creates the property inspector UI for a selected widget.
/// Returns a list of Label/input widgets to display the properties.
pub fn create_property_widgets(
    widget: Option<&Widget>, 
    selection_index: Option<usize>
) -> Vec<Widget> {
    let mut children = vec![
        Widget::label("PROPERTIES"),
        Widget::label("─────────────"),
    ];
    
    match widget {
        None => {
            children.push(Widget::label("No selection"));
        }
        Some(w) => {
            // Show selection info
            if let Some(idx) = selection_index {
                children.push(Widget::label(format!("Selected: #{}", idx)));
            }
            children.push(Widget::label("─────────────"));
            
            // Show properties based on widget type
            match w {
                Widget::Icon { .. } => {
                    children.push(Widget::label("Type: Icon"));
                }
                Widget::Container { 
                    id, width, height, background, padding, ..
                } => {
                    children.push(Widget::label("Type: Container"));
                    if let Some(id) = id {
                        children.push(Widget::label(format!("ID: {}", id)));
                    }
                    if let Some(w) = width {
                        children.push(Widget::label(format!("Width: {}", w)));
                    }
                    if let Some(h) = height {
                        children.push(Widget::label(format!("Height: {}", h)));
                    }
                    if let Some(bg) = background {
                        children.push(Widget::label(format!(
                            "BG: ({:.1},{:.1},{:.1})", 
                            bg.0, bg.1, bg.2
                        )));
                    }
                    children.push(Widget::label(format!("Padding: {}", padding)));
                }
                Widget::Label { text, size, color, .. } => {
                    children.push(Widget::label("Type: Label"));
                    children.push(Widget::label(format!("Text: {}", text)));
                    children.push(Widget::label(format!("Size: {}", size)));
                    children.push(Widget::label(format!(
                        "Color: ({:.1},{:.1},{:.1})",
                        color.0, color.1, color.2
                    )));
                }
                Widget::Button { text, action, .. } => {
                    children.push(Widget::label("Type: Button"));
                    children.push(Widget::label(format!("Text: {}", text)));
                    children.push(Widget::label(format!("Action: {}", action)));
                }
                Widget::TextInput { id, placeholder, .. } => {
                    children.push(Widget::label("Type: TextInput"));
                    children.push(Widget::label(format!("ID: {}", id)));
                    children.push(Widget::label(format!("Placeholder: {}", placeholder)));
                }
                Widget::Checkbox { id, checked, .. } => {
                    children.push(Widget::label("Type: Checkbox"));
                    children.push(Widget::label(format!("ID: {}", id)));
                    children.push(Widget::label(format!("Checked: {}", checked)));
                }
                Widget::Slider { id, value, min, max, .. } => {
                    children.push(Widget::label("Type: Slider"));
                    children.push(Widget::label(format!("ID: {}", id)));
                    children.push(Widget::label(format!("Value: {:.2}", value)));
                    children.push(Widget::label(format!("Range: {} - {}", min, max)));
                }
                Widget::Spacer { size, .. } => {
                    children.push(Widget::label("Type: Spacer"));
                    children.push(Widget::label(format!("Size: {}", size)));
                }
                Widget::Image { path, width, height, .. } => {
                    children.push(Widget::label("Type: Image"));
                    children.push(Widget::label(format!("Path: {}", path)));
                    children.push(Widget::label(format!("Size: {}x{}", width, height)));
                }
            }
        }
    }
    
    children
}
