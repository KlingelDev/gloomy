//! Widget palette panel.
//! 
//! Displays a list of available widgets that can be dragged onto the canvas.

use gloomy_core::layout::{Direction, Layout};
use gloomy_core::widget::{Widget, WidgetBounds, TextAlign};

/// Widget type that can be created.
#[derive(Debug, Clone, Copy)]
pub enum WidgetType {
    Container,
    Label,
    Button,
    TextInput,
    Checkbox,
    Slider,
    Spacer,
    Icon,
}

impl WidgetType {
    /// Returns all available widget types.
    pub fn all() -> &'static [WidgetType] {
        &[
            WidgetType::Container,
            WidgetType::Label,
            WidgetType::Button,
            WidgetType::TextInput,
            WidgetType::Checkbox,
            WidgetType::Slider,
            WidgetType::Spacer,
            WidgetType::Icon,
        ]
    }
    
    /// Returns the display name.
    pub fn name(&self) -> &'static str {
        match self {
            WidgetType::Container => "Container",
            WidgetType::Label => "Label",
            WidgetType::Button => "Button",
            WidgetType::TextInput => "TextInput",
            WidgetType::Checkbox => "Checkbox",
            WidgetType::Slider => "Slider",
            WidgetType::Spacer => "Spacer",
            WidgetType::Icon => "Icon",
        }
    }
    
    /// Creates a new instance of this widget type.
    pub fn create(&self) -> Widget {
        match self {
            WidgetType::Container => Widget::Container {
                id: Some("new_container".to_string()),
                scrollable: false,
                bounds: WidgetBounds { 
                    x: 0.0, y: 0.0, 
                    width: 200.0, height: 100.0 
                },
                width: Some(200.0),
                height: Some(100.0),
                background: Some((0.2, 0.2, 0.2, 1.0)),
                border: None,
                gradient: None,
                shadow: None,
                corner_radius: 4.0,
                corner_radii: None,
                padding: 8.0,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                layout: Layout {
                    direction: Direction::Column,
                    ..Default::default()
                },
                children: Vec::new(),
            },
            WidgetType::Label => Widget::Label {
                text: "New Label".to_string(),
                x: 0.0,
                y: 0.0,
                size: 16.0,
                color: (1.0, 1.0, 1.0, 1.0),
                text_align: TextAlign::Left,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            WidgetType::Button => Widget::Button {
                text: "Button".to_string(),
                action: "new_button".to_string(),
                bounds: WidgetBounds { 
                    x: 0.0, y: 0.0, 
                    width: 100.0, height: 32.0 
                },
                background: (0.3, 0.3, 0.3, 1.0),
                hover_color: (0.4, 0.4, 0.4, 1.0),
                active_color: (0.2, 0.2, 0.2, 1.0),
                border: None,
                gradient: None,
                shadow: None,
                corner_radius: 4.0,
                corner_radii: None,
                layout: Layout::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            WidgetType::TextInput => Widget::TextInput {
                id: "new_input".to_string(),
                value: String::new(),
                placeholder: "Enter text...".to_string(),
                font_size: 14.0,
                text_align: TextAlign::Left,
                bounds: WidgetBounds { 
                    x: 0.0, y: 0.0, 
                    width: 200.0, height: 32.0 
                },
                style: gloomy_core::widget::TextInputStyle {
                    background: Some((0.15, 0.15, 0.15, 1.0).into()),
                    border: Some(gloomy_core::widget::Border {
                        width: 1.0, 
                        color: (0.3, 0.3, 0.3, 1.0).into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                width: 200.0,
                height: 32.0,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            WidgetType::Checkbox => Widget::Checkbox {
                id: "new_checkbox".to_string(),
                checked: false,
                size: 20.0,
                style: gloomy_core::widget::CheckboxStyle {
                    background: (0.2, 0.2, 0.2, 1.0).into(),
                    checkmark_color: (1.0, 1.0, 1.0, 1.0).into(),
                    ..Default::default()
                },
                bounds: WidgetBounds::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            WidgetType::Slider => Widget::Slider {
                id: "new_slider".to_string(),
                value: 0.5,
                min: 0.0,
                max: 1.0,
                style: gloomy_core::widget::SliderStyle {
                    track_height: 4.0,
                    thumb_radius: 8.0,
                    active_track_color: (0.6, 0.6, 0.6, 1.0).into(),
                    track_color: (0.2, 0.2, 0.2, 1.0).into(),
                    ..Default::default()
                },
                bounds: WidgetBounds::default(),
                width: 200.0,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            WidgetType::Spacer => Widget::Spacer {
                size: 16.0,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            WidgetType::Icon => Widget::Icon {
                id: "new_icon".to_string(),
                icon_name: "default_icon".to_string(),
                size: 24.0,
                color: Some((1.0, 1.0, 1.0, 1.0)),
                bounds: WidgetBounds::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
        }
    }
    
}

/// Creates a clickable palette item Button for a widget type.
pub fn create_palette_item(widget_type: WidgetType) -> Widget {
    Widget::Button {
        text: format!("  {}", widget_type.name()),
        action: format!("add_{}", widget_type.name().to_lowercase()),
        bounds: WidgetBounds::default(),
        background: (0.15, 0.15, 0.18, 1.0),
        hover_color: (0.2, 0.25, 0.3, 1.0),
        active_color: (0.25, 0.3, 0.4, 1.0),
        border: None,
        gradient: None,
        shadow: None,
        corner_radius: 2.0,
        corner_radii: None,
        layout: Layout::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
    }
}
