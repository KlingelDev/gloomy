use crate::layout::Layout;
use crate::style::BoxStyle;
use crate::ui::find_widget_mut;
use crate::widget::Widget;
use std::cell::RefCell;

fn make_container(id: Option<&str>, children: Vec<Widget>) -> Widget {
  Widget::Container {
    id: id.map(Into::into),
    style: BoxStyle::default(),
    width: None,
    height: None,
    layout: Layout::default(),
    children,
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

// Regression: find_widget_mut must match a Container by its id field.
// Previously only leaf widgets (TextInput, Button, etc.) were matched.
#[test]
fn test_find_widget_mut_container_id() {
  let mut root = make_container(Some("target"), vec![]);
  let found = find_widget_mut(&mut root, "target");
  assert!(
    found.is_some(),
    "find_widget_mut must find a container by id"
  );
}

// Regression: Container id lookup must recurse into nested containers.
#[test]
fn test_find_widget_mut_nested_container_id() {
  let inner = make_container(Some("inner"), vec![]);
  let mut root = make_container(Some("root"), vec![inner]);
  let found = find_widget_mut(&mut root, "inner");
  assert!(
    found.is_some(),
    "find_widget_mut must find nested container by id"
  );
}

#[test]
fn test_find_widget_mut_nonexistent_returns_none() {
  let mut root = make_container(Some("root"), vec![]);
  let found = find_widget_mut(&mut root, "nonexistent");
  assert!(
    found.is_none(),
    "find_widget_mut must return None for unknown id"
  );
}

// Container with no id must not match any search id.
#[test]
fn test_find_widget_mut_unnamed_container_not_matched() {
  let mut root = make_container(None, vec![]);
  let found = find_widget_mut(&mut root, "anything");
  assert!(
    found.is_none(),
    "unnamed container must not match any id"
  );
}
