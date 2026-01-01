//! Design document model.

use gloomy_core::widget::Widget;

/// A design document being edited.
#[derive(Debug, Clone)]
pub struct DesignDocument {
    /// Root widget of the design
    pub root: Widget,
    
    /// Metadata about the design
    pub metadata: DesignMetadata,
}

/// Metadata about a design document.
#[derive(Debug, Clone, Default)]
pub struct DesignMetadata {
    pub name: String,
    pub version: String,
}

impl DesignDocument {
    /// Creates a new empty design document.
    pub fn new() -> Self {
        Self {
            root: Widget::container(),
            metadata: DesignMetadata {
                name: "Untitled".to_string(),
                version: "1.0".to_string(),
            },
        }
    }
}

impl Default for DesignDocument {
    fn default() -> Self {
        Self::new()
    }
}
