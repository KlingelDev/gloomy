use crate::widget::{Widget, WidgetBounds};
use crate::layout::{Layout, Direction, Justify, Align};
use crate::layout_engine::compute_layout;
use crate::style::BoxStyle;
use std::cell::RefCell;

fn test_layout(mut root: Widget, w: f32, h: f32) -> Widget {
    if let Widget::Container { bounds, .. } = &mut root {
        bounds.width = w;
        bounds.height = h;
    }
    compute_layout(&mut root, 0.0, 0.0, w, h);
    root
}

#[test]
fn test_container_fixed_size() {
    let child = Widget::Container {
        id: Some("child".into()),
        style: BoxStyle::default(),
        width: Some(100.0),   // Direct field
        height: Some(200.0),  // Direct field
        layout: Layout::default(),
        children: vec![],
        bounds: Default::default(),
        padding: 0.0,
        scrollable: false,
        layout_cache: None,
        render_cache: RefCell::new(None),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
    };

    let root = Widget::Container {
        id: Some("root".into()),
        style: BoxStyle::default(),
        width: None,
        height: None,
        layout: Layout {
            direction: Direction::Column, // Stack it
            ..Default::default()
        },
        children: vec![child],
        bounds: Default::default(),
        padding: 0.0,
        scrollable: false,
        layout_cache: None,
        render_cache: RefCell::new(None),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
    };

    let result = test_layout(root, 800.0, 600.0);
    
    if let Widget::Container { children, .. } = result {
        let child_res = &children[0];
        if let Widget::Container { bounds, .. } = child_res {
             assert_eq!(bounds.width, 100.0);
             assert_eq!(bounds.height, 200.0);
        } else {
             panic!("Child is not a container");
        }
    } else {
        panic!("Root is not a container");
    }
}

#[test]
fn test_flex_row_distribution() {
    let child1 = Widget::Container {
        id: Some("c1".into()),
        style: BoxStyle::default(),
        width: None,
        height: Some(50.0),
        flex: 1.0,  // Direct field
        layout: Layout::default(),
        children: vec![],
        bounds: Default::default(),
        padding: 0.0,
        scrollable: false,
        layout_cache: None,
        render_cache: RefCell::new(None),
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
    };
    
    let child2 = Widget::Container {
        id: Some("c2".into()),
        style: BoxStyle::default(),
        width: None,
        height: Some(50.0),
        flex: 1.0,  // Direct field
        layout: Layout::default(),
        children: vec![],
        bounds: Default::default(),
        padding: 0.0,
        scrollable: false,
        layout_cache: None,
        render_cache: RefCell::new(None),
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
    };

    let root = Widget::Container {
        id: Some("root".into()),
        style: BoxStyle::default(),
        width: None,
        height: None,
        flex: 0.0,
        layout: Layout {
            direction: Direction::Row,
            ..Default::default()
        },
        children: vec![child1, child2],
        bounds: Default::default(),
        padding: 0.0,
        scrollable: false,
        layout_cache: None,
        render_cache: RefCell::new(None),
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
    };

    // Layout in 200x100 box
    let result = test_layout(root, 200.0, 100.0);

    if let Widget::Container { children, .. } = result {
        assert_eq!(children.len(), 2);
        
        // Child 1
        let c1 = &children[0];
        let b1 = get_bounds(c1);
        assert_eq!(b1.width, 100.0, "Child 1 should have half width (flex=1)");
        assert_eq!(b1.x, 0.0);

        // Child 2
        let c2 = &children[1];
        let b2 = get_bounds(c2);
        assert_eq!(b2.width, 100.0, "Child 2 should have half width (flex=1)");
        assert_eq!(b2.x, 100.0);
    } else {
        panic!("Root is not a container");
    }
}

fn get_bounds(w: &Widget) -> WidgetBounds {
    match w {
        Widget::Container { bounds, .. } => *bounds,
        _ => panic!("Not a container"),
    }
}
