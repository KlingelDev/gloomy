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

// Widget::label() constructor must set background=None and
// corner_radii=[0;4] by default.
#[test]
fn test_label_constructor_default_background() {
  let label = Widget::label("hello");
  if let Widget::Label {
    background, corner_radii, ..
  } = &label
  {
    assert!(
      background.is_none(),
      "Label constructor must default background to None"
    );
    assert_eq!(
      *corner_radii,
      [0.0; 4],
      "Label constructor must default corner_radii to [0;4]"
    );
  } else {
    panic!("Widget::label() did not produce a Label");
  }
}

// Label with background set must retain the color value.
#[test]
fn test_label_with_background_set() {
  let mut label = Widget::label("test");
  if let Widget::Label {
    background,
    corner_radii,
    ..
  } = &mut label
  {
    *background = Some((0.2, 0.4, 0.6, 0.8));
    *corner_radii = [4.0, 4.0, 4.0, 4.0];
  }
  if let Widget::Label {
    background,
    corner_radii,
    ..
  } = &label
  {
    assert_eq!(
      *background,
      Some((0.2, 0.4, 0.6, 0.8)),
      "Label background must retain set value"
    );
    assert_eq!(
      *corner_radii,
      [4.0; 4],
      "Label corner_radii must retain set value"
    );
  }
}

// Label deserialization from RON with background field.
#[test]
fn test_label_ron_deserialize_with_background() {
  let ron_str = r#"Label(
    text: "today",
    width: 28.0,
    height: 20.0,
    size: 14.0,
    background: Some((0.1, 0.2, 0.3, 1.0)),
    corner_radii: (6.0, 6.0, 6.0, 6.0),
  )"#;
  let label: Widget = ron::from_str(ron_str).expect(
    "Label with background must deserialize from RON",
  );
  if let Widget::Label {
    text,
    width,
    height,
    background,
    corner_radii,
    ..
  } = &label
  {
    assert_eq!(text, "today");
    assert_eq!(*width, 28.0);
    assert_eq!(*height, 20.0);
    assert_eq!(
      *background,
      Some((0.1, 0.2, 0.3, 1.0)),
    );
    assert_eq!(*corner_radii, [6.0; 4]);
  } else {
    panic!("Deserialized widget is not a Label");
  }
}

// Label deserialization without background defaults to None.
#[test]
fn test_label_ron_deserialize_without_background() {
  let ron_str = r#"Label(text: "plain")"#;
  let label: Widget = ron::from_str(ron_str).expect(
    "Label without background must deserialize from RON",
  );
  if let Widget::Label {
    background,
    corner_radii,
    ..
  } = &label
  {
    assert!(
      background.is_none(),
      "Omitted background must default to None"
    );
    assert_eq!(
      *corner_radii,
      [0.0; 4],
      "Omitted corner_radii must default to [0;4]"
    );
  } else {
    panic!("Deserialized widget is not a Label");
  }
}
