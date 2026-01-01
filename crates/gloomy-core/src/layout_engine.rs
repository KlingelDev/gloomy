//! Layout engine for recalculating widget bounds.

use crate::layout::{Align, Direction, Justify, TrackSize};
use crate::widget::Widget;

/// Computes the layout for a widget tree.
pub fn compute_layout(
  widget: &mut Widget,
  _parent_x: f32,
  _parent_y: f32,
  _parent_width: f32,
  _parent_height: f32,
) {
  match widget {
    Widget::Container {
      bounds,
      layout,
      padding,
      children,
      ..
    } => {
      // Effective content area
      let content_width = (bounds.width - *padding * 2.0).max(0.0);
      let content_height = (bounds.height - *padding * 2.0).max(0.0);

      // Check if we have an active layout
      match layout.direction {
        Direction::Row | Direction::Column => {
          // FLEX LAYOUT ALGORITHM
          let mut total_flex = 0.0;
          let mut total_fixed_main = 0.0;
          let mut count = 0;

          // 1. Calculate totals
          for child in children.iter() {
            let flex_val = get_flex(child);
            if flex_val > 0.0 {
              total_flex += flex_val;
            } else {
              let (w, h) = get_fixed_size(child);
              match layout.direction {
                Direction::Row => total_fixed_main += w,
                Direction::Column => total_fixed_main += h,
                _ => {}
              }
            }
            count += 1;
          }

          let spacing_total = if count > 1 {
            (count - 1) as f32 * layout.spacing
          } else {
            0.0
          };

          // Available space for flex items
          let main_axis_size = match layout.direction {
            Direction::Row => content_width,
            Direction::Column => content_height,
            _ => 0.0,
          };

          let available_flex_space =
            (main_axis_size - total_fixed_main - spacing_total).max(0.0);

          // 2. Position items
          let mut current_main = 0.0;

          // Handle Justify (only if no flex items consume all space)
          if total_flex == 0.0 && available_flex_space > 0.0 {
            match layout.justify_content {
              Justify::Start => {} // current_main = 0
              Justify::Center => current_main = available_flex_space / 2.0,
              Justify::End => current_main = available_flex_space,
              Justify::SpaceBetween => { /* Handled in loop */ }
              Justify::SpaceAround => { /* Handled in loop */ }
            }
          }

          // Space between step
          let step_extra = if total_flex == 0.0 && count > 1 {
            match layout.justify_content {
              Justify::SpaceBetween => available_flex_space / (count - 1) as f32,
              Justify::SpaceAround => available_flex_space / count as f32,
              _ => 0.0,
            }
          } else {
            0.0
          };

          if total_flex == 0.0
            && matches!(layout.justify_content, Justify::SpaceAround)
          {
            current_main += step_extra / 2.0;
          }

          for child in children.iter_mut() {
            let child_flex = get_flex(child);
            let (mut child_w, mut child_h) = get_fixed_size(child); // Start with desired/fixed size

            // Calculate main axis size
            if child_flex > 0.0 {
              let share = child_flex / total_flex;
              let flex_size = share * available_flex_space;
              match layout.direction {
                Direction::Row => child_w = flex_size,
                Direction::Column => child_h = flex_size,
                _ => {}
              }
            }

            // Calculate cross axis size/alignment
            let cross_axis_size = match layout.direction {
              Direction::Row => content_height,
              Direction::Column => content_width,
              _ => 0.0,
            };

            match layout.align_items {
              Align::Stretch => {
                match layout.direction {
                  Direction::Row => child_h = cross_axis_size,
                  Direction::Column => child_w = cross_axis_size,
                  _ => {}
                }
              }
              _ => {} // Keep fixed/natural size on cross axis
            }

            // Set size
            set_size(child, child_w, child_h);

            // Set position
            match layout.direction {
              Direction::Row => {
                let cross_pos = match layout.align_items {
                  Align::Start | Align::Stretch => 0.0,
                  Align::Center => (content_height - child_h) / 2.0,
                  Align::End => content_height - child_h,
                };
                set_pos(child, *padding + current_main, *padding + cross_pos);
                current_main += child_w;
              }
              Direction::Column => {
                let cross_pos = match layout.align_items {
                  Align::Start | Align::Stretch => 0.0,
                  Align::Center => (content_width - child_w) / 2.0,
                  Align::End => content_width - child_w,
                };
                set_pos(child, *padding + cross_pos, *padding + current_main);
                current_main += child_h;
              }
              _ => {}
            }

            // Add spacing (and extra justification space)
            current_main += layout.spacing + step_extra;

            // Recurse
            compute_layout(
              child,
              bounds.x,
              bounds.y,
              bounds.width,
              bounds.height,
            );
          }
        }
        Direction::Grid { columns: cols } => {
          let cols = cols;
          if cols == 0 {
             return;
          }

          // --- AUTO-FLOW PASS ---
          // Assign grid positions to children that don't have explicit ones.
          // Track which cells are occupied.
          let child_count = children.len();
          // Estimate rows needed (may grow)
          let mut estimated_rows = (child_count + cols - 1) / cols;
          estimated_rows = estimated_rows.max(1);
          
          // Storage for assigned positions
          let mut assigned_positions: Vec<(usize, usize)> = Vec::with_capacity(child_count);
          
          // Occupancy grid (row-major)
          let mut occupied = vec![vec![false; cols]; estimated_rows];
          
          // Helper to find next free cell
          let mut current_row = 0usize;
          let mut current_col = 0usize;
          
          for child in children.iter() {
              let explicit_col = get_explicit_grid_col(child);
              let explicit_row = get_explicit_grid_row(child);
              let c_span = get_col_span(child);
              let r_span = get_row_span(child);
              
              let (c, r) = if explicit_col.is_some() && explicit_row.is_some() {
                  // Explicit position
                  (explicit_col.unwrap(), explicit_row.unwrap())
              } else {
                  // Auto-flow: find next available cell
                  loop {
                      // Grow occupancy grid if needed
                      while current_row >= occupied.len() {
                          occupied.push(vec![false; cols]);
                      }
                      
                      if current_col < cols && !occupied[current_row][current_col] {
                          // Check if span fits
                          let mut fits = true;
                          for dc in 0..c_span {
                              for dr in 0..r_span {
                                  let tc = current_col + dc;
                                  let tr = current_row + dr;
                                  while tr >= occupied.len() {
                                      occupied.push(vec![false; cols]);
                                  }
                                  if tc >= cols || occupied[tr][tc] {
                                      fits = false;
                                      break;
                                  }
                              }
                              if !fits { break; }
                          }
                          if fits {
                              break;
                          }
                      }
                      // Move to next cell
                      current_col += 1;
                      if current_col >= cols {
                          current_col = 0;
                          current_row += 1;
                      }
                  }
                  (current_col, current_row)
              };
              
              // Mark cells as occupied
              for dc in 0..c_span {
                  for dr in 0..r_span {
                      let tc = c + dc;
                      let tr = r + dr;
                      while tr >= occupied.len() {
                          occupied.push(vec![false; cols]);
                      }
                      if tc < cols {
                          occupied[tr][tc] = true;
                      }
                  }
              }
              
              assigned_positions.push((c, r));
              
              // Advance cursor for auto-flow
              if explicit_col.is_none() || explicit_row.is_none() {
                  current_col += c_span;
                  if current_col >= cols {
                      current_col = 0;
                      current_row += 1;
                  }
              }
          }
          
          let rows = occupied.len();
          if rows == 0 {
              return;
          }

          let mut col_widths = vec![0.0f32; cols];
          let mut row_heights = vec![0.0f32; rows];

          // --- COLUMN SIZING ---
          if !layout.template_columns.is_empty() {
             let total_fixed: f32 = layout.template_columns
                 .iter().take(cols)
                 .map(|t| match t { TrackSize::Px(v) => *v, _ => 0.0 })
                 .sum();
             
             let total_fr: f32 = layout.template_columns
                 .iter().take(cols)
                 .map(|t| match t {
                     TrackSize::Fr(v) => *v,
                     TrackSize::Auto => 1.0,
                     _ => 0.0
                 })
                 .sum();
             
             let spacing_total = if cols > 1 {
                 (cols - 1) as f32 * layout.spacing
             } else {
                 0.0
             };
             let available = (content_width - total_fixed - spacing_total).max(0.0);
             
             for (i, track) in layout.template_columns.iter().enumerate().take(cols) {
                 match track {
                     TrackSize::Px(v) => col_widths[i] = *v,
                     TrackSize::Fr(v) => col_widths[i] = (v / total_fr) * available,
                     TrackSize::Auto => col_widths[i] = (1.0 / total_fr) * available,
                 }
             }
          } else {
              // Auto-sizing based on content
              for (idx, child) in children.iter().enumerate() {
                  let (c, _r) = assigned_positions[idx];
                  let c_span = get_col_span(child);
                  let (w, _h) = get_fixed_size(child);
                  if c_span == 1 && c < cols {
                      col_widths[c] = col_widths[c].max(w);
                  }
              }
          }
          
          // --- ROW SIZING ---
          for (idx, child) in children.iter().enumerate() {
              let (_c, r) = assigned_positions[idx];
              let r_span = get_row_span(child);
              let (_w, h) = get_fixed_size(child);
              if r_span == 1 && r < rows {
                  row_heights[r] = row_heights[r].max(h);
              }
          }

          // --- COMPUTE OFFSETS ---
          let mut col_offsets = vec![0.0; cols + 1];
          let mut row_offsets = vec![0.0; rows + 1];
          
          let mut current_offset = *padding;
          for i in 0..cols {
              col_offsets[i] = current_offset;
              current_offset += col_widths[i] + layout.spacing;
          }

          current_offset = *padding;
          for i in 0..rows {
              row_offsets[i] = current_offset;
              current_offset += row_heights[i] + layout.spacing;
          }

          // --- POSITION CHILDREN ---
          for (idx, child) in children.iter_mut().enumerate() {
              let (c, r) = assigned_positions[idx];
              let c_span = get_col_span(child);
              let r_span = get_row_span(child);

              if c < cols && r < rows {
                  let x = col_offsets[c];
                  let y = row_offsets[r];
                  
                  // Calculate spanned size
                  let mut width = 0.0;
                  for i in c..std::cmp::min(c + c_span, cols) {
                      width += col_widths[i];
                      if i > c { width += layout.spacing; }
                  }

                  let mut height = 0.0;
                  for i in r..std::cmp::min(r + r_span, rows) {
                      height += row_heights[i];
                      if i > r { height += layout.spacing; }
                  }
                  
                  let (fw, fh) = get_fixed_size(child);
                  
                  let (final_w, x_off) = if matches!(layout.align_items, Align::Stretch) {
                      (width, 0.0)
                  } else {
                       let w = fw.min(width);
                       match layout.justify_content {
                           Justify::Center => (w, (width - w) / 2.0),
                           Justify::End => (w, width - w),
                           _ => (w, 0.0)
                       }
                  };

                  let (final_h, y_off) = match layout.align_items {
                      Align::Stretch => (height, 0.0),
                      Align::Center => {
                          let h = fh.min(height);
                          (h, (height - h) / 2.0)
                      }
                      Align::End => {
                          let h = fh.min(height);
                          (h, height - h)
                      }
                      Align::Start => (fh.min(height), 0.0),
                  };

                  set_pos(child, x + x_off, y + y_off);
                  set_size(child, final_w, final_h);
                  
                  compute_layout(child, x, y, width, height);
              }
          }
        }
        Direction::None => {
          // Manual layout, just recurse
          for child in children.iter_mut() {
            compute_layout(
              child,
              bounds.x,
              bounds.y,
              bounds.width,
              bounds.height,
            );
          }
        }
      }
    }
    _ => {
      // Leaf widgets
    }
  }
}

// Helper to get flex factor
fn get_flex(widget: &Widget) -> f32 {
  match widget {
    Widget::Container { flex, .. } => *flex,
    Widget::Button { flex, .. } => *flex,
    Widget::Label { flex, .. } => *flex,
    Widget::TextInput { flex, .. } => *flex,
    Widget::Spacer { flex, .. } => *flex,
    Widget::Checkbox { flex, .. } => *flex,
    Widget::Slider { flex, .. } => *flex,
    Widget::Image { flex, .. } => *flex,
    Widget::Icon { flex, .. } => *flex,
  }
}

// Helper to get fixed/intrinsic size
fn get_fixed_size(widget: &Widget) -> (f32, f32) {
  match widget {
    Widget::Container { bounds, width, height, padding, children, .. } => {
        // If explicit width/height specified, use those
        let mut w = width.unwrap_or(0.0);
        let mut h = height.unwrap_or(0.0);
        
        // If not specified, try to compute from children
        if w <= 0.0 && !children.is_empty() {
            // Sum widths for Row, max widths for Column
            let mut max_w = 0.0f32;
            for child in children {
                let (cw, _) = get_fixed_size(child);
                max_w = max_w.max(cw);
            }
            w = max_w + padding * 2.0;
        }
        
        if h <= 0.0 && !children.is_empty() {
            // Sum heights for Column, max heights for Row
            let mut max_h = 0.0f32;
            for child in children {
                let (_, ch) = get_fixed_size(child);
                max_h = max_h.max(ch);
            }
            h = max_h + padding * 2.0;
        }
        
        // Fallback to bounds if still zero
        if w <= 0.0 { w = bounds.width; }
        if h <= 0.0 { h = bounds.height; }
        
        (w, h)
    },
    Widget::Image { width, height, .. } => {
        let w = if *width > 0.0 { *width } else { 100.0 };
        let h = if *height > 0.0 { *height } else { 100.0 };
        (w, h)
    },
    Widget::Icon { size, .. } => (*size, *size),
    Widget::Button { bounds, text, .. } => {
        // If bounds are set, use them. Otherwise estimate.
        if bounds.width > 0.0 && bounds.height > 0.0 {
            (bounds.width, bounds.height)
        } else {
             // 16px font, padding 10px H, 5px V
             let w = text.len() as f32 * 16.0 * 0.6 + 20.0;
             let h = 16.0 + 10.0;
             (w, h)
        }
    },
    Widget::Label { size, text, .. } => {
      // Rough estimation for text size if not set manually
      // roughly 0.6 * size per char width, size height
      let w = text.len() as f32 * size * 0.6;
      (w, *size)
    }
    Widget::TextInput { width, height, .. } => {
        let w = if *width > 0.0 { *width } else { 200.0 }; // Default width
        let h = if *height > 0.0 { *height } else { 32.0 }; // Default height
        (w, h)
    }
    Widget::Spacer { size, .. } => (*size, *size),
    Widget::Checkbox { size, .. } => (*size, *size),
    Widget::Slider { width, track_height, thumb_radius, .. } => {
          let h = (*thumb_radius * 2.0).max(*track_height);
          let w = if *width > 0.0 { *width } else { 200.0 };
          (w, h)
    },

  }
}

// Helper to set widget size
fn set_size(widget: &mut Widget, w: f32, h: f32) {
  match widget {
    Widget::Container { bounds, .. } => {
      bounds.width = w;
      bounds.height = h;
    }
    Widget::Image { bounds, .. } => {
        bounds.width = w;
        bounds.height = h;
    }
    Widget::Icon { bounds, .. } => {
        bounds.width = w;
        bounds.height = h;
    }
    Widget::Button { bounds, .. } => {
      bounds.width = w;
      bounds.height = h;
    }
    Widget::Label { .. } => { /* Text usually auto-sizes */ }
    Widget::TextInput { bounds, .. } => {
      bounds.width = w;
      bounds.height = h;
    }
    Widget::Spacer { size, .. } => {
      *size = w.max(h);
    }
    Widget::Checkbox { bounds, .. } => {
        bounds.width = w;
        bounds.height = h;
    }
    Widget::Slider { bounds, .. } => {
        bounds.width = w;
        bounds.height = h;
    }
  }
}

// Helper to set widget position relative to parent
fn set_pos(widget: &mut Widget, x: f32, y: f32) {
  match widget {
    Widget::Container { bounds, .. } => {
      bounds.x = x;
      bounds.y = y;
    }
    Widget::Image { bounds, .. } => {
        bounds.x = x;
        bounds.y = y;
    }
    Widget::Icon { bounds, .. } => {
        bounds.x = x;
        bounds.y = y;
    }
    Widget::Button { bounds, .. } => {
      bounds.x = x;
      bounds.y = y;
    }
    Widget::Label { x: lx, y: ly, .. } => {
      *lx = x;
      *ly = y;
    }
    Widget::TextInput { bounds, .. } => {
      bounds.x = x;
      bounds.y = y;
    }
    Widget::Spacer { .. } => {}
    Widget::Checkbox { bounds, .. } => {
        bounds.x = x;
        bounds.y = y;
    }
    Widget::Slider { bounds, .. } => {
        bounds.x = x;
        bounds.y = y;
    }
  }
}
// Helper to get grid column (returns 0 if unset for auto-flow)
fn get_grid_col(widget: &Widget) -> usize {
  match widget {
    Widget::Container { grid_col, .. } => grid_col.unwrap_or(0),
    Widget::Button { grid_col, .. } => grid_col.unwrap_or(0),
    Widget::Label { grid_col, .. } => grid_col.unwrap_or(0),
    Widget::TextInput { grid_col, .. } => grid_col.unwrap_or(0),
    Widget::Spacer { grid_col, .. } => grid_col.unwrap_or(0),
    Widget::Checkbox { grid_col, .. } => grid_col.unwrap_or(0),
    Widget::Slider { grid_col, .. } => grid_col.unwrap_or(0),
    Widget::Image { grid_col, .. } => grid_col.unwrap_or(0),
    Widget::Icon { grid_col, .. } => grid_col.unwrap_or(0),
  }
}

// Helper to get grid row (returns 0 if unset for auto-flow)
fn get_grid_row(widget: &Widget) -> usize {
  match widget {
    Widget::Container { grid_row, .. } => grid_row.unwrap_or(0),
    Widget::Image { grid_row, .. } => grid_row.unwrap_or(0),
    Widget::Icon { grid_row, .. } => grid_row.unwrap_or(0),
    Widget::Button { grid_row, .. } => grid_row.unwrap_or(0),
    Widget::Label { grid_row, .. } => grid_row.unwrap_or(0),
    Widget::TextInput { grid_row, .. } => grid_row.unwrap_or(0),
    Widget::Spacer { grid_row, .. } => grid_row.unwrap_or(0),
    Widget::Checkbox { grid_row, .. } => grid_row.unwrap_or(0),
    Widget::Slider { grid_row, .. } => grid_row.unwrap_or(0),
  }
}


// Helper to get explicit grid column (returns None if auto-flow should apply)
fn get_explicit_grid_col(widget: &Widget) -> Option<usize> {
  match widget {
    Widget::Container { grid_col, .. } => *grid_col,
    Widget::Button { grid_col, .. } => *grid_col,
    Widget::Label { grid_col, .. } => *grid_col,
    Widget::TextInput { grid_col, .. } => *grid_col,
    Widget::Spacer { grid_col, .. } => *grid_col,
    Widget::Checkbox { grid_col, .. } => *grid_col,
    Widget::Slider { grid_col, .. } => *grid_col,
    Widget::Image { grid_col, .. } => *grid_col,
    Widget::Icon { grid_col, .. } => *grid_col,
  }
}

// Helper to get explicit grid row (returns None if auto-flow should apply)
fn get_explicit_grid_row(widget: &Widget) -> Option<usize> {
  match widget {
    Widget::Container { grid_row, .. } => *grid_row,
    Widget::Image { grid_row, .. } => *grid_row,
    Widget::Icon { grid_row, .. } => *grid_row,
    Widget::Button { grid_row, .. } => *grid_row,
    Widget::Label { grid_row, .. } => *grid_row,
    Widget::TextInput { grid_row, .. } => *grid_row,
    Widget::Spacer { grid_row, .. } => *grid_row,
    Widget::Checkbox { grid_row, .. } => *grid_row,
    Widget::Slider { grid_row, .. } => *grid_row,
  }
}

// Helper to get col span
fn get_col_span(widget: &Widget) -> usize {
  match widget {
    Widget::Container { col_span, .. } => *col_span,
    Widget::Image { col_span, .. } => *col_span,
    Widget::Icon { col_span, .. } => *col_span,
    Widget::Button { col_span, .. } => *col_span,
    Widget::Label { col_span, .. } => *col_span,
    Widget::TextInput { col_span, .. } => *col_span,
    Widget::Spacer { col_span, .. } => *col_span,
    Widget::Checkbox { col_span, .. } => *col_span,
    Widget::Slider { col_span, .. } => *col_span,

  }
}

// Helper to get row span
fn get_row_span(widget: &Widget) -> usize {
  match widget {
    Widget::Container { row_span, .. } => *row_span,
    Widget::Image { row_span, .. } => *row_span,
    Widget::Icon { row_span, .. } => *row_span,
    Widget::Button { row_span, .. } => *row_span,
    Widget::Label { row_span, .. } => *row_span,
    Widget::TextInput { row_span, .. } => *row_span,
    Widget::Spacer { row_span, .. } => *row_span,
    Widget::Checkbox { row_span, .. } => *row_span,
    Widget::Slider { row_span, .. } => *row_span,

  }
}
