use serde::{Deserialize, Serialize};

/// Node in the tree hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    /// Unique identifier for the node
    pub id: String,
    
    /// Display label
    pub label: String,
    
    /// Optional icon (unicode character or asset helper)
    pub icon: Option<String>,
    
    /// Children nodes
    #[serde(default)]
    pub children: Vec<TreeNode>,
    
    /// Whether this node cannot have children (visual hint)
    #[serde(default)]
    pub leaf: bool,
}

impl TreeNode {
    /// Creates a new tree node.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            children: Vec::new(),
            leaf: false,
        }
    }
    
    /// Marks the node as a leaf (no children/expander).
    pub fn leaf(mut self) -> Self {
        self.leaf = true;
        self
    }
    
    /// Sets the icon for the node.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    
    /// Adds a child node.
    pub fn child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }
    
    /// Adds multiple children.
    pub fn with_children(mut self, children: Vec<TreeNode>) -> Self {
        self.children = children;
        self
    }
}

/// Visual styling for the Tree widget.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TreeStyle {
    pub font_size: f32,
    pub text_color: (f32, f32, f32, f32),
    pub icon_color: (f32, f32, f32, f32),
    pub indent_size: f32,
    pub row_height: f32,
    pub selected_background: (f32, f32, f32, f32),
    pub hover_background: (f32, f32, f32, f32),
}

impl Default for TreeStyle {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            text_color: (0.9, 0.9, 0.9, 1.0),
            icon_color: (0.7, 0.7, 0.7, 1.0),
            indent_size: 20.0,
            row_height: 24.0,
            selected_background: (0.2, 0.4, 0.8, 0.8), // Blue selection
            hover_background: (1.0, 1.0, 1.0, 0.1),
        }
    }
}
