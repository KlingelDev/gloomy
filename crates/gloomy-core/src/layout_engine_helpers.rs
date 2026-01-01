
// Helper to get grid column
fn get_grid_col(widget: &Widget) -> usize {
  match widget {
    Widget::Container { grid_col, .. } => *grid_col,
    Widget::Button { grid_col, .. } => *grid_col,
    Widget::Label { grid_col, .. } => *grid_col,
    Widget::Spacer { grid_col, .. } => *grid_col,
  }
}

// Helper to get grid row
fn get_grid_row(widget: &Widget) -> usize {
  match widget {
    Widget::Container { grid_row, .. } => *grid_row,
    Widget::Button { grid_row, .. } => *grid_row,
    Widget::Label { grid_row, .. } => *grid_row,
    Widget::Spacer { grid_row, .. } => *grid_row,
  }
}

// Helper to get col span
fn get_col_span(widget: &Widget) -> usize {
  match widget {
    Widget::Container { col_span, .. } => *col_span,
    Widget::Button { col_span, .. } => *col_span,
    Widget::Label { col_span, .. } => *col_span,
    Widget::Spacer { col_span, .. } => *col_span,
  }
}

// Helper to get row span
fn get_row_span(widget: &Widget) -> usize {
  match widget {
    Widget::Container { row_span, .. } => *row_span,
    Widget::Button { row_span, .. } => *row_span,
    Widget::Label { row_span, .. } => *row_span,
    Widget::Spacer { row_span, .. } => *row_span,
  }
}
