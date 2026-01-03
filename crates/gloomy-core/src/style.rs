use serde::{Deserialize, Serialize};
use crate::widget::Color;

/// Global styling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStyle {
    pub bg_color: Color,
    pub text_color: Color,
    pub primary_color: Color,
    pub secondary_color: Color,
    pub font_size: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ListViewStyle {
    pub item_height: f32,
    #[serde(default)]
    pub idle: BoxStyle,
    #[serde(default)]
    pub hover: BoxStyle,
    #[serde(default)]
    pub selected: BoxStyle,
    pub text_color_idle: Color,
    pub text_color_selected: Color,
}

impl Default for ListViewStyle {
    fn default() -> Self {
        Self {
            item_height: 40.0,
            idle: BoxStyle { background: None, ..Default::default() },
            hover: BoxStyle::fill((1.0, 1.0, 1.0, 0.1)),
            selected: BoxStyle::fill((0.2, 0.6, 1.0, 1.0)),
            text_color_idle: (0.8, 0.8, 0.8, 1.0),
            text_color_selected: (1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl Default for GlobalStyle {
    fn default() -> Self {
        Self {
            bg_color: (0.1, 0.1, 0.1, 1.0),
            text_color: (0.9, 0.9, 0.9, 1.0),
            primary_color: (0.2, 0.6, 1.0, 1.0),
            secondary_color: (0.4, 0.4, 0.4, 1.0),
            font_size: 16.0,
        }
    }
}

// --- Primitive Styles ---

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub offset: (f32, f32),
    pub blur: f32,
    pub color: Color,
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            offset: (2.0, 2.0),
            blur: 4.0,
            color: (0.0, 0.0, 0.0, 0.5),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Gradient {
    pub start: Color,
    pub end: Color,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub enum BorderStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub struct Border {
    pub width: f32,
    pub color: Color,
    #[serde(default)]
    pub radius: [f32; 4], // Independent radii? Or on Box? Usually on Box. 
                          // Radius here might be redundant if BoxStyle has it.
                          // Let's keep radius on BoxStyle for geometry.
}

/// A unified style for box-like widgets (Container, Buttons, Cards).
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct BoxStyle {
    /// Solid background color. Overridden by gradient if present.
    #[serde(default)]
    pub background: Option<Color>,
    
    /// Linear gradient background (Top to Bottom).
    #[serde(default)]
    pub gradient: Option<Gradient>,
    
    /// Border stroke.
    #[serde(default)]
    pub border: Option<Border>,
    
    /// Drop shadow.
    #[serde(default)]
    pub shadow: Option<Shadow>,
    
    /// Corner radii [TopLeft, TopRight, BottomRight, BottomLeft].
    #[serde(default)]
    pub corner_radii: [f32; 4],
}

impl BoxStyle {
    pub fn reset() -> Self {
        Self::default()
    }
    
    pub fn fill(color: Color) -> Self {
        Self {
            background: Some(color),
            ..Default::default()
        }
    }
    
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.corner_radii = [radius; 4];
        self
    }
    
    pub fn with_border(mut self, color: Color, width: f32) -> Self {
        self.border = Some(Border { width, color, radius: [0.0;4] }); // radius ignored
        self
    }
    
    pub fn with_shadow(mut self, offset: (f32, f32), blur: f32, color: Color) -> Self {
        self.shadow = Some(Shadow { offset, blur, color });
        self
    }
}

// --- Widget Specific Styles ---

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ButtonStyle {
    #[serde(default)]
    pub idle: BoxStyle,
    #[serde(default)]
    pub hover: BoxStyle,
    #[serde(default)]
    pub active: BoxStyle,
    #[serde(default)]
    pub disabled: BoxStyle,
    #[serde(default)]
    pub text_color: Color,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            idle: BoxStyle::fill((0.2, 0.2, 0.25, 1.0)).with_radius(4.0),
            hover: BoxStyle::fill((0.25, 0.25, 0.3, 1.0)).with_radius(4.0),
            active: BoxStyle::fill((0.15, 0.15, 0.2, 1.0)).with_radius(4.0),
            disabled: BoxStyle::fill((0.1, 0.1, 0.1, 0.5)).with_radius(4.0),
            text_color: (0.9, 0.9, 0.9, 1.0),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TextInputStyle {
    #[serde(default)]
    pub idle: BoxStyle,
    #[serde(default)]
    pub focused: BoxStyle,
    #[serde(default)]
    pub placeholder_color: Color,
    #[serde(default)]
    pub text_color: Color,
    #[serde(default)]
    pub cursor_color: Color,
    #[serde(default)]
    pub font: Option<String>,
}

impl Default for TextInputStyle {
    fn default() -> Self {
        Self {
            idle: BoxStyle {
                background: Some((0.1, 0.1, 0.12, 1.0)),
                border: Some(Border { width: 1.0, color: (0.3, 0.3, 0.35, 1.0), radius: [4.0; 4] }),
                corner_radii: [4.0; 4],
                ..Default::default()
            },
            focused: BoxStyle {
                 background: Some((0.15, 0.15, 0.18, 1.0)),
                 border: Some(Border { width: 1.0, color: (0.2, 0.5, 0.9, 1.0), radius: [4.0; 4] }),
                 corner_radii: [4.0; 4],
                 ..Default::default()
            },
            placeholder_color: (0.5, 0.5, 0.5, 1.0),
            text_color: (0.9, 0.9, 0.9, 1.0),
            cursor_color: (0.2, 0.5, 0.9, 1.0),
            font: None,
        }
    }
}
