//! Style context that combines theme and global styles.
//!
//! The StyleContext holds the active theme and global style,
//! providing a central place to manage UI appearance that can
//! be switched at runtime.

use crate::theme::Theme;
use crate::style::GlobalStyle;
use crate::widget::Color;

/// Context holding the active theme and global style settings.
///
/// This is the main entry point for theming in Gloomy UI.
/// Pass a reference to StyleContext through the rendering pipeline
/// to allow widgets to resolve theme colors and use global style defaults.
#[derive(Debug, Clone)]
pub struct StyleContext {
    /// Active color theme
    pub theme: Theme,
    /// Active global style settings
    pub global_style: GlobalStyle,
}

impl StyleContext {
    /// Creates a new style context with the given theme and style.
    pub fn new(theme: Theme, global_style: GlobalStyle) -> Self {
        Self {
            theme,
            global_style,
        }
    }
    
    /// Sets a new theme, replacing the current one.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
    
    /// Sets new global style settings, replacing the current ones.
    pub fn set_global_style(&mut self, global_style: GlobalStyle) {
        self.global_style = global_style;
    }
    
    /// Gets a theme color by name.
    ///
    /// Returns None if the color name is not recognized.
    ///
    /// # Example
    /// ```ignore
    /// let color = ctx.get_theme_color("primary");
    /// ```
    pub fn get_theme_color(&self, name: &str) -> Option<Color> {
        self.theme.get_color(name)
    }
    
    /// Gets a theme color by name, or returns a fallback color.
    ///
    /// # Example
    /// ```ignore
    /// let color = ctx.get_theme_color_or(
    ///     "primary",
    ///     (0.5, 0.5, 0.5, 1.0)
    /// );
    /// ```
    pub fn get_theme_color_or(&self, name: &str, fallback: Color) -> Color {
        self.get_theme_color(name).unwrap_or(fallback)
    }
}

impl Default for StyleContext {
    /// Returns a default style context with dark theme and modern style.
    fn default() -> Self {
        Self::new(Theme::default(), GlobalStyle::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_style_context_creation() {
        let ctx = StyleContext::default();
        assert_eq!(ctx.theme.name, "Dark");
    }
    
    #[test]
    fn test_theme_switching() {
        let mut ctx = StyleContext::default();
        assert_eq!(ctx.theme.name, "Dark");
        
        ctx.set_theme(Theme::light());
        assert_eq!(ctx.theme.name, "Light");
    }
    
    #[test]
    fn test_get_theme_color() {
        let ctx = StyleContext::default();
        
        let primary = ctx.get_theme_color("primary");
        assert!(primary.is_some());
        
        let invalid = ctx.get_theme_color("invalid");
        assert!(invalid.is_none());
    }
    
    #[test]
    fn test_get_theme_color_or() {
        let ctx = StyleContext::default();
        let fallback = (1.0, 0.0, 0.0, 1.0);
        
        let primary = ctx.get_theme_color_or("primary", fallback);
        assert_ne!(primary, fallback);
        
        let invalid = ctx.get_theme_color_or("invalid", fallback);
        assert_eq!(invalid, fallback);
    }
}
