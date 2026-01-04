//! Automation API for testing Gloomy UIs.

use gloomy_core::widget::{Widget, WidgetBounds};
use gloomy_core::{InteractionState, hit_test, compute_layout, load_ui, parse_ui};
use glam::Vec2;

/// A headless driver for interacting with a Gloomy UI tree.
pub struct GloomyDriver {
    pub root: Widget,
    pub interaction: InteractionState,
    pub width: f32,
    pub height: f32,
}

impl GloomyDriver {
    /// Creates a new driver with the given root widget and screen dimensions.
    ///
    /// This performs an initial layout calculation.
    pub fn new(mut root: Widget, width: f32, height: f32) -> Self {
        // Set root dimensions if it's a container
        if let Widget::Container { bounds, .. } = &mut root {
            bounds.width = width;
            bounds.height = height;
        }
        
        // Initial layout
        compute_layout(&mut root, 0.0, 0.0, width, height);

        Self {
            root,
            interaction: InteractionState::default(),
            width,
            height,
        }
    }

    /// Finds a widget by its ID.
    pub fn find<'a>(&'a self, id: &str) -> Option<&'a Widget> {
        Self::find_recursive(&self.root, id)
    }

    fn find_recursive<'a>(widget: &'a Widget, target_id: &str) -> Option<&'a Widget> {
        match widget {
            Widget::Container { id, children, .. } => {
                if id.as_deref() == Some(target_id) {
                    return Some(widget);
                }
                for child in children {
                    if let Some(found) = Self::find_recursive(child, target_id) {
                        return Some(found);
                    }
                }
            }
            Widget::Button { .. } | Widget::Label { .. } | Widget::TextInput { .. } | 
            Widget::NumberInput { .. } | Widget::DatePicker { .. } | Widget::Autocomplete { .. } |
            Widget::Checkbox { .. } | Widget::Slider { .. } | Widget::Image { .. } | Widget::Icon { .. } => {
                if let Some(wid) = Self::get_widget_id(widget) {
                    if wid == target_id {
                        return Some(widget);
                    }
                }
            }
            _ => {
                 if let Some(wid) = Self::get_widget_id(widget) {
                    if wid == target_id {
                        return Some(widget);
                    }
                }
            }
        }
        None
    }
    
    fn get_widget_id(widget: &Widget) -> Option<&str> {
        match widget {
             Widget::Container { id, .. } => id.as_deref(),
             Widget::ToggleSwitch { id, .. } => Some(id),
             Widget::TextInput { id, .. } => Some(id),
             Widget::NumberInput { id, .. } => Some(id),
             Widget::DatePicker { id, .. } => Some(id),
             Widget::Autocomplete { id, .. } => Some(id),
             Widget::Checkbox { id, .. } => Some(id),
             Widget::Slider { id, .. } => Some(id),
             Widget::Dropdown { id, .. } => Some(id),
             Widget::KpiCard { id, .. } => id.as_deref(),
             Widget::DataGrid { id, .. } => id.as_deref(),
             Widget::Icon { id, .. } => Some(id),
             _ => None,
        }
    }
    
    /// Finds a widget's bounds by ID.
    pub fn find_bounds(&self, id: &str) -> Option<WidgetBounds> {
        self.find(id).map(|w| Self::get_bounds(w))
    }
    
    fn get_bounds(widget: &Widget) -> WidgetBounds {
        match widget {
            Widget::Container { bounds, .. } => *bounds,
            Widget::Label { x, y, width, height, .. } => WidgetBounds { x: *x, y: *y, width: *width, height: *height },
            Widget::Button { bounds, .. } => *bounds,
            Widget::TextInput { bounds, .. } => *bounds,
            Widget::NumberInput { bounds, .. } => *bounds,
            Widget::DatePicker { bounds, .. } => *bounds,
            Widget::Autocomplete { bounds, .. } => *bounds,
            Widget::Checkbox { bounds, .. } => *bounds,
            Widget::Slider { bounds, .. } => *bounds,
            Widget::Dropdown { bounds, .. } => *bounds,
            Widget::ToggleSwitch { bounds, .. } => *bounds,
            Widget::ProgressBar { bounds, .. } => *bounds,
            Widget::RadioButton { bounds, .. } => *bounds,
            Widget::Divider { bounds, .. } => *bounds,
            Widget::Scrollbar { bounds, .. } => *bounds,
            Widget::DataGrid { bounds, .. } => *bounds,
            Widget::KpiCard { bounds, .. } => *bounds,
            Widget::ListView { bounds, .. } => *bounds,
            Widget::Tree { .. } => WidgetBounds::default(),
            Widget::Image { bounds, .. } => *bounds,
            Widget::Icon { bounds, .. } => *bounds,
            Widget::Spacer { .. } => WidgetBounds::default(), 
        }
    }

    /// Simulates a click on the widget with the given ID.
    pub fn click(&mut self, id: &str) -> Option<String> {
        let bounds = self.find_bounds(id)?;
        
        let center_x = bounds.x + bounds.width * 0.5;
        let center_y = bounds.y + bounds.height * 0.5;
        
        self.interaction.update_mouse(Vec2::new(center_x, center_y));
        self.interaction.set_pressed(true);
        let hit = hit_test(&self.root, Vec2::new(center_x, center_y), Some(&self.interaction));
        self.interaction.handle_hit(hit.as_ref().map(|h| h.action.clone()));
        
        self.interaction.set_pressed(false);
        
        if let Some(w) = self.find(id) {
             match w {
                 Widget::Button { action, .. } => Some(action.clone()),
                 Widget::ToggleSwitch { id, .. } => Some(id.clone()),
                 Widget::Checkbox { id, .. } => Some(id.clone()),
                 _ => None
             }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gloomy_core::style::ButtonStyle;
    
    #[test]
    fn test_driver_sanity() {
        let mut root = Widget::container();
         if let Widget::Container { id, children, .. } = &mut root {
             *id = Some("root".to_string());
             *children = vec![
                 Widget::Button {
                     text: "Click Me".to_string(),
                     action: "my_action".to_string(),
                     bounds: WidgetBounds { x: 0.0, y: 0.0, width: 100.0, height: 50.0 }, 
                     style: ButtonStyle::default(),
                     width: Some(100.0), height: Some(50.0), disabled: false, layout: Default::default(),
                     flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, font: None,
                 },
             ];
        } else {
            panic!("Widget::container() did not return a Container");
        }
        
        let driver = GloomyDriver::new(root, 800.0, 600.0);
        let found = driver.find("root");
        assert!(found.is_some());
    }
}
