///! Global style defaults for Gloomy UI.
///!
///! Provides default values for:
///! - Border radii
///! - Border widths
///! - Spacing values
///! - Shadow presets
///! - Typography sizes

use serde::{Deserialize, Serialize};
use crate::widget::{Shadow, Gradient};

/// Global style defaults that can be applied across all widgets.
///
/// These values provide consistency across the UI and can be
/// customized per application or switched at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStyle {
    // Border radii
    /// Small corner radius (e.g., chips, tags)
    pub corner_radius_small: f32,
    /// Medium corner radius (e.g., buttons, inputs)
    pub corner_radius_medium: f32,
    /// Large corner radius (e.g., cards, modals)
    pub corner_radius_large: f32,
    
    // Border widths
    /// Thin border (e.g., dividers)
    pub border_width_thin: f32,
    /// Normal border (e.g., inputs, buttons)
    pub border_width_normal: f32,
    /// Thick border (e.g., focus indicators)
    pub border_width_thick: f32,
    
    // Spacing
    /// Small spacing (e.g., tight padding)
    pub spacing_small: f32,
    /// Medium spacing (e.g., default padding/gaps)
    pub spacing_medium: f32,
    /// Large spacing (e.g., section gaps)
    pub spacing_large: f32,
    
    // Shadows
    /// Small shadow (e.g., buttons)
    pub shadow_small: Option<Shadow>,
    /// Medium shadow (e.g., cards)
    pub shadow_medium: Option<Shadow>,
    /// Large shadow (e.g., modals, dropdowns)
    pub shadow_large: Option<Shadow>,
    
    // Typography
    /// Small font size (e.g., captions, hints)
    pub font_size_small: f32,
    /// Normal font size (e.g., body text)
    pub font_size_normal: f32,
    /// Large font size (e.g., subheadings)
    pub font_size_large: f32,
    /// Heading font size
    pub font_size_heading: f32,
    
    // Optional gradient presets
    /// Primary gradient preset
    pub gradient_primary: Option<Gradient>,
    /// Secondary gradient preset
    pub gradient_secondary: Option<Gradient>,
}

impl Default for GlobalStyle {
    /// Returns the default "Modern" style.
    fn default() -> Self {
        Self::modern()
    }
}

impl GlobalStyle {
    /// Creates a modern style with smooth corners and subtle shadows.
    pub fn modern() -> Self {
        Self {
            corner_radius_small: 4.0,
            corner_radius_medium: 8.0,
            corner_radius_large: 12.0,
            
            border_width_thin: 1.0,
            border_width_normal: 2.0,
            border_width_thick: 3.0,
            
            spacing_small: 8.0,
            spacing_medium: 16.0,
            spacing_large: 24.0,
            
            shadow_small: Some(Shadow {
                offset: (0.0, 2.0),
                blur: 4.0,
                color: (0.0, 0.0, 0.0, 0.1),
            }),
            shadow_medium: Some(Shadow {
                offset: (0.0, 4.0),
                blur: 8.0,
                color: (0.0, 0.0, 0.0, 0.15),
            }),
            shadow_large: Some(Shadow {
                offset: (0.0, 8.0),
                blur: 16.0,
                color: (0.0, 0.0, 0.0, 0.2),
            }),
            
            font_size_small: 12.0,
            font_size_normal: 16.0,
            font_size_large: 20.0,
            font_size_heading: 28.0,
            
            gradient_primary: None,
            gradient_secondary: None,
        }
    }
    
    /// Creates a classic style with sharper corners and no shadows.
    pub fn classic() -> Self {
        Self {
            corner_radius_small: 0.0,
            corner_radius_medium: 0.0,
            corner_radius_large: 0.0,
            
            border_width_thin: 1.0,
            border_width_normal: 2.0,
            border_width_thick: 3.0,
            
            spacing_small: 8.0,
            spacing_medium: 16.0,
            spacing_large: 24.0,
            
            shadow_small: None,
            shadow_medium: None,
            shadow_large: None,
            
            font_size_small: 12.0,
            font_size_normal: 16.0,
            font_size_large: 20.0,
            font_size_heading: 28.0,
            
            gradient_primary: None,
            gradient_secondary: None,
        }
    }
    
    /// Creates a minimal style with subtle rounded corners.
    pub fn minimal() -> Self {
        Self {
            corner_radius_small: 2.0,
            corner_radius_medium: 4.0,
            corner_radius_large: 6.0,
            
            border_width_thin: 1.0,
            border_width_normal: 1.0,
            border_width_thick: 2.0,
            
            spacing_small: 6.0,
            spacing_medium: 12.0,
            spacing_large: 18.0,
            
            shadow_small: Some(Shadow {
                offset: (0.0, 1.0),
                blur: 2.0,
                color: (0.0, 0.0, 0.0, 0.05),
            }),
            shadow_medium: None,
            shadow_large: None,
            
            font_size_small: 12.0,
            font_size_normal: 14.0,
            font_size_large: 18.0,
            font_size_heading: 24.0,
            
            gradient_primary: None,
            gradient_secondary: None,
        }
    }
    
    /// Loads a global style from a RON file.
    ///
    /// # Example
    /// ```ignore
    /// let style = GlobalStyle::load("styles/modern.ron")?;
    /// ```
    pub fn load(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let style: GlobalStyle = ron::from_str(&content)?;
        Ok(style)
    }
    
    /// Saves the global style to a RON file.
    ///
    /// # Example
    /// ```ignore
    /// style.save("styles/my_style.ron")?;
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
    fn test_default_styles() {
        let modern = GlobalStyle::modern();
        let classic = GlobalStyle::classic();
        let minimal = GlobalStyle::minimal();
        
        // Modern has rounded corners
        assert!(modern.corner_radius_medium > 0.0);
        
        // Classic has no corners
        assert_eq!(classic.corner_radius_medium, 0.0);
        
        // Minimal has smaller corners
        assert!(minimal.corner_radius_medium < modern.corner_radius_medium);
    }
}
