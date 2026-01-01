//! Theme system for Gloomy UI.
//!
//! Provides a flexible theming mechanism with:
//! - Predefined color palettes
//! - Semantic color naming
//! - Runtime theme switching
//! - RON configuration support

use serde::{Deserialize, Serialize};
use crate::widget::Color;

/// A complete UI theme with named color palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name for identification
    pub name: String,
    /// Color palette for this theme
    pub colors: ColorPalette,
}

/// Semantic color palette for UI theming.
///
/// All colors follow a consistent naming scheme:
/// - Base colors: background, surface, primary, secondary, accent
/// - Text colors: text, text_secondary, text_disabled
/// - Interactive: hover, active, focus
/// - Semantic: success, warning, error, info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    // Base colors
    /// Main background color
    pub background: Color,
    /// Surface color (containers, cards)
    pub surface: Color,
    /// Primary brand color
    pub primary: Color,
    /// Secondary brand color
    pub secondary: Color,
    /// Accent color for highlights
    pub accent: Color,
    
    // Text colors
    /// Primary text color
    pub text: Color,
    /// Secondary/muted text color
    pub text_secondary: Color,
    /// Disabled text color
    pub text_disabled: Color,
    
    // Interactive states
    /// Hover state color
    pub hover: Color,
    /// Active/pressed state color
    pub active: Color,
    /// Focus indicator color
    pub focus: Color,
    
    // Semantic colors
    /// Success state color (green)
    pub success: Color,
    /// Warning state color (yellow/orange)
    pub warning: Color,
    /// Error state color (red)
    pub error: Color,
    /// Info state color (blue)
    pub info: Color,
    
    // Borders and dividers
    /// Border color
    pub border: Color,
    /// Divider line color
    pub divider: Color,
}

impl Theme {
    /// Creates a new theme with the given name and colors.
    pub fn new(name: String, colors: ColorPalette) -> Self {
        Self { name, colors }
    }
    
    /// Gets a color by semantic name.
    ///
    /// Returns None if the color name is not recognized.
    pub fn get_color(&self, name: &str) -> Option<Color> {
        match name {
            "background" => Some(self.colors.background),
            "surface" => Some(self.colors.surface),
            "primary" => Some(self.colors.primary),
            "secondary" => Some(self.colors.secondary),
            "accent" => Some(self.colors.accent),
            "text" => Some(self.colors.text),
            "text_secondary" => Some(self.colors.text_secondary),
            "text_disabled" => Some(self.colors.text_disabled),
            "hover" => Some(self.colors.hover),
            "active" => Some(self.colors.active),
            "focus" => Some(self.colors.focus),
            "success" => Some(self.colors.success),
            "warning" => Some(self.colors.warning),
            "error" => Some(self.colors.error),
            "info" => Some(self.colors.info),
            "border" => Some(self.colors.border),
            "divider" => Some(self.colors.divider),
            _ => None,
        }
    }
}

impl Default for Theme {
    /// Returns the default "Dark" theme.
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Creates a dark theme (default).
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            colors: ColorPalette {
                background: (0.12, 0.12, 0.12, 1.0),
                surface: (0.18, 0.18, 0.18, 1.0),
                primary: (0.4, 0.6, 1.0, 1.0),
                secondary: (0.6, 0.4, 1.0, 1.0),
                accent: (1.0, 0.6, 0.2, 1.0),
                
                text: (0.95, 0.95, 0.95, 1.0),
                text_secondary: (0.7, 0.7, 0.7, 1.0),
                text_disabled: (0.5, 0.5, 0.5, 1.0),
                
                hover: (0.25, 0.25, 0.25, 1.0),
                active: (0.35, 0.35, 0.35, 1.0),
                focus: (0.4, 0.6, 1.0, 0.5),
                
                success: (0.2, 0.8, 0.3, 1.0),
                warning: (1.0, 0.7, 0.2, 1.0),
                error: (0.9, 0.2, 0.2, 1.0),
                info: (0.3, 0.7, 1.0, 1.0),
                
                border: (0.3, 0.3, 0.3, 1.0),
                divider: (0.25, 0.25, 0.25, 1.0),
            },
        }
    }
    
    /// Creates a light theme.
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            colors: ColorPalette {
                background: (0.98, 0.98, 0.98, 1.0),
                surface: (1.0, 1.0, 1.0, 1.0),
                primary: (0.2, 0.4, 0.9, 1.0),
                secondary: (0.4, 0.2, 0.9, 1.0),
                accent: (0.9, 0.4, 0.1, 1.0),
                
                text: (0.1, 0.1, 0.1, 1.0),
                text_secondary: (0.4, 0.4, 0.4, 1.0),
                text_disabled: (0.6, 0.6, 0.6, 1.0),
                
                hover: (0.95, 0.95, 0.95, 1.0),
                active: (0.9, 0.9, 0.9, 1.0),
                focus: (0.2, 0.4, 0.9, 0.3),
                
                success: (0.1, 0.6, 0.2, 1.0),
                warning: (0.9, 0.5, 0.1, 1.0),
                error: (0.8, 0.1, 0.1, 1.0),
                info: (0.1, 0.5, 0.9, 1.0),
                
                border: (0.8, 0.8, 0.8, 1.0),
                divider: (0.85, 0.85, 0.85, 1.0),
            },
        }
    }
    
    /// Creates a high contrast theme for accessibility.
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),
            colors: ColorPalette {
                background: (0.0, 0.0, 0.0, 1.0),
                surface: (0.1, 0.1, 0.1, 1.0),
                primary: (0.0, 0.8, 1.0, 1.0),
                secondary: (1.0, 0.8, 0.0, 1.0),
                accent: (1.0, 0.4, 0.0, 1.0),
                
                text: (1.0, 1.0, 1.0, 1.0),
                text_secondary: (0.9, 0.9, 0.9, 1.0),
                text_disabled: (0.6, 0.6, 0.6, 1.0),
                
                hover: (0.3, 0.3, 0.3, 1.0),
                active: (0.5, 0.5, 0.5, 1.0),
                focus: (0.0, 0.8, 1.0, 0.8),
                
                success: (0.0, 1.0, 0.3, 1.0),
                warning: (1.0, 0.8, 0.0, 1.0),
                error: (1.0, 0.0, 0.0, 1.0),
                info: (0.0, 0.8, 1.0, 1.0),
                
                border: (0.5, 0.5, 0.5, 1.0),
                divider: (0.4, 0.4, 0.4, 1.0),
            },
        }
    }
    
    /// Loads a theme from a RON file.
    ///
    /// # Example
    /// ```ignore
    /// let theme = Theme::load("themes/dark.ron")?;
    /// ```
    pub fn load(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let theme: Theme = ron::from_str(&content)?;
        Ok(theme)
    }
    
    /// Saves the theme to a RON file.
    ///
    /// # Example
    /// ```ignore
    /// theme.save("themes/my_theme.ron")?;
    /// ```
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let ron_string = ron::ser::to_string_pretty(
            self,
            ron::ser::PrettyConfig::default()
        )?;
        std::fs::write(path, ron_string)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_color() {
        let theme = Theme::dark();
        
        assert_eq!(theme.get_color("primary"), Some(theme.colors.primary));
        assert_eq!(theme.get_color("text"), Some(theme.colors.text));
        assert_eq!(theme.get_color("invalid"), None);
    }
    
    #[test]
    fn test_default_themes() {
        let dark = Theme::dark();
        let light = Theme::light();
        let hc = Theme::high_contrast();
        
        assert_eq!(dark.name, "Dark");
        assert_eq!(light.name, "Light");
        assert_eq!(hc.name, "High Contrast");
    }
}
