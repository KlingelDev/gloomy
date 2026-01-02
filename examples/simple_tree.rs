/// Simple Tree Widget Example
///
/// Demonstrates:
/// - Tree widget with hierarchical file-system-like structure
/// - Expand/collapse functionality  
/// - Node selection
/// - Hit testing and state management
///
/// Run with: cargo run --example simple_tree

use gloomy_core::{
    layout::Layout,
    layout_engine::compute_layout,
    ui::{render_ui, hit_test},
    widget::{Widget, WidgetBounds, TextAlign},
    tree::{TreeNode, TreeStyle},
    interaction::InteractionState,
    Vec2,
};
use std::{cell::RefCell, rc::Rc, collections::HashSet};
use winit::event::{ElementState, MouseButton};

struct AppState {
    interaction: InteractionState,
    ui_root: Widget,
    expanded_ids: HashSet<String>,
    selected_id: Option<String>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Start with root nodes expanded
    let mut expanded = HashSet::new();
    expanded.insert("root".to_string());
    expanded.insert("src".to_string());
    
    let initial_ui = create_ui(&expanded, None);
    
    let state = Rc::new(RefCell::new(AppState {
        interaction: InteractionState::new(),
        ui_root: initial_ui,
        expanded_ids: expanded,
        selected_id: None,
    }));
    
    println!("Tree Widget Example");
    println!("Click ► to expand/collapse nodes");
    println!("Click on node label to select");
    
    // Clones for callbacks
    let state_draw = state.clone();
    let state_move = state.clone();
    let state_click = state.clone();
    
    gloomy_app::GloomyApp::new()
        .on_cursor_move(move |_win, x, y| {
            let mut s = state_move.borrow_mut();
            let pos = Vec2::new(x, y);
            s.interaction.update_mouse(pos);
             // Hit test to update hovered state
            let hit_action = hit_test(
                &s.ui_root, 
                pos, 
                Some(&s.interaction.scroll_offsets)
            )
            .map(|h| h.action.clone());
            
            s.interaction.handle_hit(hit_action);
        })
        .on_mouse_input(move |win, state_enum, button| {
            if button != MouseButton::Left {
                return;
            }
            
            let pressed = state_enum == ElementState::Pressed;
            let mut s = state_click.borrow_mut();
            s.interaction.set_pressed(pressed);
            
            if pressed {
                let hovered = s.interaction.hovered_action.clone();
                
                // Parse action format: "tree:toggle:node_id" 
                // or "tree:select:node_id"
                if let Some(action) = hovered {
                    let parts: Vec<&str> = 
                        action.split(':').collect();
                    
                    if parts.len() >= 3 && parts[0] == "tree" {
                        let action_type = parts[1];
                        let node_id = parts[2];
                        
                        match action_type {
                            "toggle" => {
                                // Toggle expansion state
                                if s.expanded_ids.contains(node_id) {
                                    s.expanded_ids.remove(node_id);
                                } else {
                                    s.expanded_ids
                                        .insert(node_id.to_string());
                                }
                                println!(
                                    "Toggled node: {} (now {})", 
                                    node_id,
                                    if s.expanded_ids
                                        .contains(node_id) 
                                    {
                                        "expanded"
                                    } else {
                                        "collapsed"
                                    }
                                );
                                
                                win.window.request_redraw();
                            }
                            "select" => {
                                // Update selection
                                s.selected_id = 
                                    Some(node_id.to_string());
                                println!(
                                    "Selected node: {}", 
                                    node_id
                                );
                                
                                win.window.request_redraw();
                            }
                            _ => {}
                        }
                    }
                }
            }
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            
            // Rebuild UI with current state
            let ui_root_new = create_ui(
                &s.expanded_ids, 
                s.selected_id.as_ref()
            );
            s.ui_root = ui_root_new;
            
            let window_size = win.window.inner_size();
            let width = window_size.width as f32;
            let height = window_size.height as f32;
            
            // Manually update root bounds
                bounds.width = width;
                bounds.height = height;
            }
            
            compute_layout(
                &mut s.ui_root, 
                0.0, 
                0.0, 
                width, 
                height
            );
            
            // Render
            render_ui(
                &s.ui_root,
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                Some(&s.interaction),
                None,
            );
        })
        .run()
}

/// Creates the UI with a sample file-system-like tree.
fn create_ui(
    expanded_ids: &HashSet<String>,
    selected_id: Option<&String>,
) -> Widget {
    // Build a sample tree structure
    let tree_data = vec![
        TreeNode::new("root", "📁 Project")
            .child(
                TreeNode::new("src", "📁 src")
                    .child(
                        TreeNode::new("main", "📄 main.rs")
                            .leaf()
                    )
                    .child(
                        TreeNode::new("lib", "📄 lib.rs")
                            .leaf()
                    )
                    .child(
                        TreeNode::new("widgets", "📁 widgets")
                            .child(
                                TreeNode::new(
                                    "button", 
                                    "📄 button.rs"
                                )
                                .leaf()
                            )
                            .child(
                                TreeNode::new(
                                    "tree", 
                                    "📄 tree.rs"
                                )
                                .leaf()
                            )
                            .child(
                                TreeNode::new(
                                    "label", 
                                    "📄 label.rs"
                                )
                                .leaf()
                            )
                    )
            )
            .child(
                TreeNode::new("examples", "📁 examples")
                    .child(
                        TreeNode::new(
                            "simple_tree", 
                            "📄 simple_tree.rs"
                        )
                        .leaf()
                    )
                    .child(
                        TreeNode::new(
                            "datagrid", 
                            "📄 simple_datagrid.rs"
                        )
                        .leaf()
                    )
            )
            .child(
                TreeNode::new("docs", "📁 docs")
                    .child(
                        TreeNode::new(
                            "readme", 
                            "📄 README.md"
                        )
                        .leaf()
                    )
                    .child(
                        TreeNode::new(
                            "roadmap", 
                            "📄 roadmap.md"
                        )
                        .leaf()
                    )
            )
            .child(
                TreeNode::new("cargo", "📄 Cargo.toml")
                    .leaf()
            ),
    ];
    
    // Create the tree widget
    let tree_widget = Widget::Tree {
        id: Some("tree".to_string()),
        bounds: WidgetBounds::default(),
        root_nodes: tree_data,
        selected_id: selected_id.cloned(),
        expanded_ids: expanded_ids.clone(),
        style: TreeStyle::default(),
        flex: 1.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
    };
    
        bounds: WidgetBounds::default(),
        background: Some((0.12, 0.12, 0.15, 1.0)),
        border: None,
        shadow: None,
        gradient: None,
        corner_radius: 0.0,
        corner_radii: None,
        padding: 20.0,
        children: vec![
            // Title
            Widget::Label {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 30.0,
                text: "Tree Widget - Click ► to Expand, \
                       Click Label to Select"
                    .to_string(),
                size: 18.0,
                color: (0.9, 0.9, 0.95, 1.0),
                font: None,
                text_align: TextAlign::Left,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            tree_widget,
        ],
        layout: Layout {
            direction: gloomy_core::layout::Direction::Column,
            spacing: 10.0,
            align_items: gloomy_core::layout::Align::Stretch,
            justify_content: 
                gloomy_core::layout::Justify::Start,
            template_columns: vec![],
        },
        width: None,
        height: None,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        id: None,
        scrollable: false,
    }
}
