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

fn make_grid_cell(w: f32, h: f32) -> Widget {
  Widget::Container {
    id: None,
    style: BoxStyle::default(),
    width: Some(w),
    height: Some(h),
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
  }
}

// Regression: get_fixed_size for Grid must sum row heights, not take max.
// A 2-column grid with 4 cells of height=50 has 2 rows; intrinsic
// height must be 100, not 50.
#[test]
fn test_grid_intrinsic_height_sums_rows() {
  let grid = Widget::Container {
    id: Some("grid".into()),
    style: BoxStyle::default(),
    width: None,
    height: None,
    layout: Layout {
      direction: Direction::Grid { columns: 2 },
      spacing: 0.0,
      ..Default::default()
    },
    children: vec![
      make_grid_cell(100.0, 50.0),
      make_grid_cell(100.0, 50.0),
      make_grid_cell(100.0, 50.0),
      make_grid_cell(100.0, 50.0),
    ],
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
      direction: Direction::Column,
      align_items: Align::Start,
      ..Default::default()
    },
    children: vec![grid],
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
    if let Widget::Container { bounds, .. } = &children[0] {
      assert_eq!(
        bounds.height,
        100.0,
        "Grid height must sum row heights (2 rows × 50 = 100)"
      );
    } else {
      panic!("grid child is not a container");
    }
  } else {
    panic!("root is not a container");
  }
}

// Regression: get_fixed_size for Row must sum child widths, not max.
#[test]
fn test_row_intrinsic_width_sums_children() {
  let row = Widget::Container {
    id: Some("row".into()),
    style: BoxStyle::default(),
    width: None,
    height: None,
    layout: Layout {
      direction: Direction::Row,
      spacing: 0.0,
      ..Default::default()
    },
    children: vec![
      make_grid_cell(80.0, 30.0),
      make_grid_cell(60.0, 30.0),
      make_grid_cell(40.0, 30.0),
    ],
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
      direction: Direction::Column,
      align_items: Align::Start,
      ..Default::default()
    },
    children: vec![row],
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
    let b = get_bounds(&children[0]);
    assert_eq!(
      b.width, 180.0,
      "Row intrinsic width must be sum of children (80+60+40)"
    );
  } else {
    panic!("root is not a container");
  }
}

// Regression: get_fixed_size for Column must sum child heights.
#[test]
fn test_column_intrinsic_height_sums_children() {
  let col = Widget::Container {
    id: Some("col".into()),
    style: BoxStyle::default(),
    width: None,
    height: None,
    layout: Layout {
      direction: Direction::Column,
      spacing: 0.0,
      ..Default::default()
    },
    children: vec![
      make_grid_cell(50.0, 40.0),
      make_grid_cell(50.0, 60.0),
      make_grid_cell(50.0, 20.0),
    ],
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
      direction: Direction::Row,
      align_items: Align::Start,
      ..Default::default()
    },
    children: vec![col],
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
    let b = get_bounds(&children[0]);
    assert_eq!(
      b.height, 120.0,
      "Column intrinsic height must be sum of children (40+60+20)"
    );
  } else {
    panic!("root is not a container");
  }
}

// Row cross-axis (height) must be max of children, not sum.
#[test]
fn test_row_intrinsic_height_is_max() {
  let row = Widget::Container {
    id: Some("row".into()),
    style: BoxStyle::default(),
    width: None,
    height: None,
    layout: Layout {
      direction: Direction::Row,
      spacing: 0.0,
      ..Default::default()
    },
    children: vec![
      make_grid_cell(50.0, 30.0),
      make_grid_cell(50.0, 70.0),
      make_grid_cell(50.0, 50.0),
    ],
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
      direction: Direction::Column,
      align_items: Align::Start,
      ..Default::default()
    },
    children: vec![row],
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
    let b = get_bounds(&children[0]);
    assert_eq!(
      b.height, 70.0,
      "Row intrinsic height must be max of children (70)"
    );
  } else {
    panic!("root is not a container");
  }
}

// Column cross-axis (width) must be max of children, not sum.
#[test]
fn test_column_intrinsic_width_is_max() {
  let col = Widget::Container {
    id: Some("col".into()),
    style: BoxStyle::default(),
    width: None,
    height: None,
    layout: Layout {
      direction: Direction::Column,
      spacing: 0.0,
      ..Default::default()
    },
    children: vec![
      make_grid_cell(40.0, 50.0),
      make_grid_cell(90.0, 50.0),
      make_grid_cell(60.0, 50.0),
    ],
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
      direction: Direction::Row,
      align_items: Align::Start,
      ..Default::default()
    },
    children: vec![col],
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
    let b = get_bounds(&children[0]);
    assert_eq!(
      b.width, 90.0,
      "Column intrinsic width must be max of children (90)"
    );
  } else {
    panic!("root is not a container");
  }
}

// Row with spacing: intrinsic width includes inter-child spacing.
#[test]
fn test_row_intrinsic_width_includes_spacing() {
  let row = Widget::Container {
    id: Some("row".into()),
    style: BoxStyle::default(),
    width: None,
    height: None,
    layout: Layout {
      direction: Direction::Row,
      spacing: 10.0,
      ..Default::default()
    },
    children: vec![
      make_grid_cell(50.0, 30.0),
      make_grid_cell(50.0, 30.0),
      make_grid_cell(50.0, 30.0),
    ],
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
      direction: Direction::Column,
      align_items: Align::Start,
      ..Default::default()
    },
    children: vec![row],
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

  // 3 children × 50 + 2 gaps × 10 = 170
  let result = test_layout(root, 800.0, 600.0);
  if let Widget::Container { children, .. } = result {
    let b = get_bounds(&children[0]);
    assert_eq!(
      b.width, 170.0,
      "Row width must include spacing (3×50 + 2×10 = 170)"
    );
  } else {
    panic!("root is not a container");
  }
}

// Column with spacing: intrinsic height includes inter-child spacing.
#[test]
fn test_column_intrinsic_height_includes_spacing() {
  let col = Widget::Container {
    id: Some("col".into()),
    style: BoxStyle::default(),
    width: None,
    height: None,
    layout: Layout {
      direction: Direction::Column,
      spacing: 5.0,
      ..Default::default()
    },
    children: vec![
      make_grid_cell(30.0, 40.0),
      make_grid_cell(30.0, 40.0),
    ],
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
      direction: Direction::Row,
      align_items: Align::Start,
      ..Default::default()
    },
    children: vec![col],
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

  // 2 children × 40 + 1 gap × 5 = 85
  let result = test_layout(root, 800.0, 600.0);
  if let Widget::Container { children, .. } = result {
    let b = get_bounds(&children[0]);
    assert_eq!(
      b.height, 85.0,
      "Column height must include spacing (2×40 + 1×5 = 85)"
    );
  } else {
    panic!("root is not a container");
  }
}

// Row with padding: intrinsic width includes 2×padding.
#[test]
fn test_row_intrinsic_width_includes_padding() {
  let row = Widget::Container {
    id: Some("row".into()),
    style: BoxStyle::default(),
    width: None,
    height: None,
    layout: Layout {
      direction: Direction::Row,
      spacing: 0.0,
      ..Default::default()
    },
    children: vec![
      make_grid_cell(100.0, 30.0),
    ],
    bounds: Default::default(),
    padding: 8.0,
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
      direction: Direction::Column,
      align_items: Align::Start,
      ..Default::default()
    },
    children: vec![row],
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

  // 100 + 2×8 padding = 116
  let result = test_layout(root, 800.0, 600.0);
  if let Widget::Container { children, .. } = result {
    let b = get_bounds(&children[0]);
    assert_eq!(
      b.width, 116.0,
      "Row width must include padding (100 + 2×8 = 116)"
    );
  } else {
    panic!("root is not a container");
  }
}

// Grid intrinsic width = max_col_w × cols + spacing + padding.
#[test]
fn test_grid_intrinsic_width_uses_max_col_width() {
  let grid = Widget::Container {
    id: Some("grid".into()),
    style: BoxStyle::default(),
    width: None,
    height: None,
    layout: Layout {
      direction: Direction::Grid { columns: 3 },
      spacing: 4.0,
      ..Default::default()
    },
    children: vec![
      make_grid_cell(30.0, 20.0),
      make_grid_cell(50.0, 20.0),
      make_grid_cell(40.0, 20.0),
      make_grid_cell(50.0, 20.0),
      make_grid_cell(30.0, 20.0),
      make_grid_cell(50.0, 20.0),
    ],
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
      direction: Direction::Column,
      align_items: Align::Start,
      ..Default::default()
    },
    children: vec![grid],
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

  // max_col_w=50, 3 cols → 50×3 + 2×4 spacing = 158
  let result = test_layout(root, 800.0, 600.0);
  if let Widget::Container { children, .. } = result {
    let b = get_bounds(&children[0]);
    assert_eq!(
      b.width, 158.0,
      "Grid width = max_col_w × cols + spacing (50×3 + 2×4 = 158)"
    );
  } else {
    panic!("root is not a container");
  }
}

// Regression: Spacer set_size must not overwrite declared size with
// cross-axis value when parent uses Align::Stretch.
#[test]
fn test_spacer_size_preserved_with_stretch_align() {
  let spacer = Widget::Spacer {
    size: 20.0,
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
      direction: Direction::Column,
      align_items: Align::Stretch,
      ..Default::default()
    },
    children: vec![spacer],
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

  let result = test_layout(root, 500.0, 300.0);
  if let Widget::Container { children, .. } = result {
    if let Widget::Spacer { size, .. } = &children[0] {
      assert_eq!(
        *size,
        20.0,
        "Spacer declared size must not be corrupted by Stretch cross-axis"
      );
    } else {
      panic!("expected a Spacer widget");
    }
  } else {
    panic!("root is not a container");
  }
}
