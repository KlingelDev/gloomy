//! Application state and logic for the designer.

use crate::model::{DesignDocument, WidgetPath};
use crate::panels::palette::{WidgetType, create_palette_item};
use crate::panels::inspector::create_property_widgets;
use gloomy_core::layout::{Direction, Layout};
use gloomy_core::widget::{Widget, WidgetBounds};

/// Main application state for the designer.
pub struct DesignerApp {
    /// The design being edited
    pub design: DesignDocument,
    
    /// Currently selected widget index (in design.root.children)
    pub selected_index: Option<usize>,
    
    /// Clipboard for copy/paste
    pub clipboard: Option<Widget>,
    
    /// Designer UI root
    pub ui_root: Widget,
}

impl DesignerApp {
    /// Creates a new designer application.
    pub fn new() -> Self {
        let ui_root = Self::create_designer_ui();
        
        Self {
            design: DesignDocument::new(),
            selected_index: None,
            clipboard: None,
            ui_root,
        }
    }
    
    /// Handles triggered actions from button clicks.
    pub fn handle_action(&mut self, action: &str) {
        // Handle palette widget creation actions
        for widget_type in WidgetType::all() {
            let expected_action = format!("add_{}", widget_type.name().to_lowercase());
            if action == expected_action {
                let new_widget = widget_type.create();
                self.add_widget_to_design(new_widget);
                return;
            }
        }
        
        // Handle selection in canvas (select_N)
        if action.starts_with("select_") {
            if let Ok(idx) = action[7..].parse::<usize>() {
                self.selected_index = Some(idx);
                log::info!("Selected widget #{}", idx);
                self.refresh_ui();
                return;
            }
        }
        
        // Handle delete
        if action == "delete_selected" {
            if let Some(idx) = self.selected_index {
                self.delete_widget_at(idx);
                return;
            }
        }
        
        log::debug!("Unhandled action: {}", action);
    }
    
    /// Adds a widget to the current design.
    fn add_widget_to_design(&mut self, widget: Widget) {
        if let Widget::Container { children, .. } = &mut self.design.root {
            children.push(widget);
            // Select the newly added widget
            self.selected_index = Some(children.len() - 1);
            log::info!("Added widget. Total: {}", children.len());
        }
    }
    
    /// Deletes widget at index.
    fn delete_widget_at(&mut self, index: usize) {
        if let Widget::Container { children, .. } = &mut self.design.root {
            if index < children.len() {
                children.remove(index);
                self.selected_index = None;
                log::info!("Deleted widget #{}", index);
            }
        }
    }
    
    /// Returns the currently selected widget.
    fn get_selected_widget(&self) -> Option<&Widget> {
        if let Widget::Container { children, .. } = &self.design.root {
            self.selected_index.and_then(|idx| children.get(idx))
        } else {
            None
        }
    }
    
    /// Refreshes the UI to reflect current design state.
    pub fn refresh_ui(&mut self) {
        self.update_canvas_children();
        self.update_tree_children();
        self.update_inspector();
    }
    
    /// Updates the canvas to show current design widgets with selection.
    fn update_canvas_children(&mut self) {
        // Clone data we need to avoid borrow conflicts
        let selected_index = self.selected_index;
        let design_children: Vec<Widget> = if let Widget::Container { 
            children, .. 
        } = &self.design.root {
            children.clone()
        } else {
            vec![]
        };
        
        // Build new canvas contents
        let new_canvas = if design_children.is_empty() {
            vec![Widget::label("Click widgets in palette to add")]
        } else {
            design_children.iter().enumerate().map(|(i, w)| {
                Self::wrap_with_selection_static(w.clone(), i, selected_index)
            }).collect()
        };
        
        // Now mutate ui_root
        if let Widget::Container { children, .. } = &mut self.ui_root {
            if let Some(Widget::Container { children: center, .. }) = children.get_mut(1) {
                if let Some(Widget::Container { children: canvas, .. }) = center.get_mut(0) {
                    *canvas = new_canvas;
                }
            }
        }
    }
    
    /// Static version to avoid borrow conflicts.
    fn wrap_with_selection_static(
        widget: Widget, 
        index: usize, 
        selected_index: Option<usize>
    ) -> Widget {
        let is_selected = selected_index == Some(index);
        let bg_color = if is_selected {
            Some((0.4, 0.4, 0.4, 0.3)) // Gray highlight
        } else {
            None
        };
        
        Widget::Container {
            id: Some(format!("wrapper_{}", index)),
            scrollable: false,
            bounds: WidgetBounds::default(),
            width: None,
            height: None,
            background: bg_color,
            border_color: if is_selected { 
                Some((0.8, 0.8, 0.8, 1.0)) // Light gray border
            } else { 
                None 
            },
            border_width: if is_selected { 2.0 } else { 0.0 },
            corner_radius: 4.0,
            corner_radii: None,
            padding: 4.0,
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
            layout: Layout::default(),
            children: vec![
                widget,
                Widget::Button {
                    text: "".to_string(),
                    action: format!("select_{}", index),
                    bounds: WidgetBounds { 
                        x: 0.0, y: 0.0, 
                        width: 1000.0, height: 50.0 
                    },
                    background: (0.0, 0.0, 0.0, 0.0),
                    hover_color: (1.0, 1.0, 1.0, 0.05),
                    active_color: (1.0, 1.0, 1.0, 0.1),
                    border_width: 0.0,
                    border_color: None,
                    corner_radius: 0.0,
                    corner_radii: None,
                    layout: Layout::default(),
                    flex: 0.0,
                    grid_col: None,
                    grid_row: None,
                    col_span: 1,
                    row_span: 1,
                },
            ],
        }
    }

    /// Updates the tree panel to show widget hierarchy.
    fn update_tree_children(&mut self) {
        if let Widget::Container { children, .. } = &mut self.ui_root {
            if let Some(Widget::Container { children: center, .. }) = children.get_mut(1) {
                if let Some(Widget::Container { children: tree, .. }) = center.get_mut(1) {
                    tree.clear();
                    tree.push(Widget::label("WIDGET TREE"));
                    
                    if let Widget::Container { children: design_c, .. } = &self.design.root {
                        tree.push(Widget::label(format!(
                            "▼ Container (root) [{}]", 
                            design_c.len()
                        )));
                        
                        for (i, child) in design_c.iter().enumerate() {
                            let name = Self::widget_type_name(child);
                            let sel = if self.selected_index == Some(i) { "► " } else { "  " };
                            tree.push(Widget::label(format!("{}├ {} #{}", sel, name, i)));
                        }
                    }
                }
            }
        }
    }
    
    /// Updates the inspector panel with selected widget properties.
    pub fn update_inspector(&mut self) {
        let selected = self.get_selected_widget().cloned();
        let props = create_property_widgets(selected.as_ref(), self.selected_index);
        
        if let Widget::Container { children, .. } = &mut self.ui_root {
            if let Some(Widget::Container { children: inspector, .. }) = children.get_mut(2) {
                *inspector = props;
            }
        }
    }
    
    /// Returns a type name for display.
    fn widget_type_name(widget: &Widget) -> &'static str {
        match widget {
            Widget::Container { .. } => "Container",
            Widget::Label { .. } => "Label",
            Widget::Button { .. } => "Button",
            Widget::TextInput { .. } => "TextInput",
            Widget::Checkbox { .. } => "Checkbox",
            Widget::Slider { .. } => "Slider",
            Widget::Spacer { .. } => "Spacer",
            Widget::Image { .. } => "Image",
        }
    }

    /// Creates the designer's own UI layout.
    fn create_designer_ui() -> Widget {
        Widget::Container {
            id: Some("designer_root".to_string()),
            scrollable: false,
            bounds: WidgetBounds {
                x: 0.0, y: 0.0,
                width: 1400.0, height: 900.0,
            },
            width: None,
            height: None,
            background: Some((0.15, 0.15, 0.15, 1.0)), // Flat dark gray
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            corner_radii: None,
            padding: 0.0,
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
            layout: Layout {
                direction: Direction::Row,
                ..Default::default()
            },
            children: vec![
                Self::create_palette_panel(),
                Self::create_center_panel(),
                Self::create_inspector_panel(),
            ],
        }
    }
    
    fn create_palette_panel() -> Widget {
        let mut palette_children = vec![
            Widget::label("WIDGETS"),
            Widget::label("─────────"),
        ];
        
        for widget_type in WidgetType::all() {
            palette_children.push(create_palette_item(*widget_type));
        }
        
        Widget::Container {
            id: Some("palette".to_string()),
            scrollable: true,
            bounds: WidgetBounds::default(),
            width: Some(200.0),
            height: None,
            background: Some((0.12, 0.12, 0.12, 1.0)), // Flat dark gray
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            corner_radii: None,
            padding: 8.0,
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
            layout: Layout {
                direction: Direction::Column,
                spacing: 4.0,
                ..Default::default()
            },
            children: palette_children,
        }
    }
    
    fn create_center_panel() -> Widget {
        Widget::Container {
            id: Some("center".to_string()),
            scrollable: false,
            bounds: WidgetBounds::default(),
            width: None,
            height: None,
            background: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            corner_radii: None,
            padding: 0.0,
            flex: 1.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
            layout: Layout {
                direction: Direction::Column,
                ..Default::default()
            },
            children: vec![
                Widget::Container {
                    id: Some("canvas".to_string()),
                    scrollable: true,
                    bounds: WidgetBounds::default(),
                    width: None,
                    height: None,
                    background: Some((0.1, 0.1, 0.1, 1.0)), // Flat dark gray
                    border_color: None,
                    border_width: 0.0,
                    corner_radius: 0.0,
                    corner_radii: None,
                    padding: 16.0,
                    flex: 1.0,
                    grid_col: None,
                    grid_row: None,
                    col_span: 1,
                    row_span: 1,
                    layout: Layout {
                        direction: Direction::Column,
                        spacing: 8.0,
                        ..Default::default()
                    },
                    children: vec![
                        Widget::label("Click widgets in palette to add"),
                    ],
                },
                Widget::Container {
                    id: Some("tree".to_string()),
                    scrollable: true,
                    bounds: WidgetBounds::default(),
                    width: None,
                    height: Some(150.0),
                    background: Some((0.1, 0.1, 0.1, 1.0)),
                    border_color: None,
                    border_width: 0.0,
                    corner_radius: 0.0,
                    corner_radii: None,
                    padding: 8.0,
                    flex: 0.0,
                    grid_col: None,
                    grid_row: None,
                    col_span: 1,
                    row_span: 1,
                    layout: Layout {
                        direction: Direction::Column,
                        spacing: 2.0,
                        ..Default::default()
                    },
                    children: vec![
                        Widget::label("WIDGET TREE"),
                        Widget::label("▼ Container (root) [0]"),
                    ],
                },
            ],
        }
    }
    
    fn create_inspector_panel() -> Widget {
        Widget::Container {
            id: Some("inspector".to_string()),
            scrollable: true,
            bounds: WidgetBounds::default(),
            width: Some(250.0),
            height: None,
            background: Some((0.12, 0.12, 0.12, 1.0)),
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            corner_radii: None,
            padding: 8.0,
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
            layout: Layout {
                direction: Direction::Column,
                spacing: 8.0,
                ..Default::default()
            },
            children: vec![
                Widget::label("PROPERTIES"),
                Widget::label("─────────────"),
                Widget::label("No selection"),
            ],
        }
    }
}

impl Default for DesignerApp {
    fn default() -> Self {
        Self::new()
    }
}
