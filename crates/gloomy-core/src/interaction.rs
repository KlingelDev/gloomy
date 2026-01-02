//! Interaction state management for UI.

use crate::widget::Widget;
use glam::Vec2;

/// Tracks the state of user interaction (mouse, hover, active).
#[derive(Debug, Default, Clone)]
pub struct InteractionState {
  /// Current mouse position.
  pub mouse_pos: Vec2,
  /// Is the mouse button currently pressed?
  pub is_pressed: bool,
  /// ID Action of the widget currently being hovered.
  pub hovered_action: Option<String>,
  /// ID Action of the widget actively being pressed.
  pub active_action: Option<String>,
  /// Action triggered in the current frame (clicked).
  pub triggered_action: Option<String>,
  /// ID Action of the widget currently focused (e.g. for text input).
  pub focused_id: Option<String>,
  /// ID of the widget clicked this frame (down-press).
  pub clicked_id: Option<String>,
  /// Scroll offsets for scrollable containers (ID -> Offset).
  pub scroll_offsets: std::collections::HashMap<String, Vec2>,
}

impl InteractionState {
  /// Create a new interaction state.
  pub fn new() -> Self {
    Self::default()
  }

  /// Update state with new mouse position.
  pub fn update_mouse(&mut self, pos: Vec2) {
    self.mouse_pos = pos;
  }

  /// Update state with mouse press/release.
  pub fn set_pressed(&mut self, pressed: bool) {
    self.is_pressed = pressed;
    if !pressed {
      // release
      self.active_action = None;
    }
  }

  /// Update state with clicked ID.
  pub fn set_clicked(&mut self, id: Option<String>) {
    self.clicked_id = id;
  }

  /// Update state with active ID (dragging).
  pub fn set_active(&mut self, id: Option<String>) {
    self.active_action = id;
  }

  /// Check if a specific action is currently hovered.
  pub fn is_hovered(&self, action: &str) -> bool {
    self.hovered_action.as_deref() == Some(action)
  }

  /// Check if a specific action is currently active (pressed).
  pub fn is_active(&self, action: &str) -> bool {
    self.active_action.as_deref() == Some(action)
  }

  /// Cycles focus to the next element in the list.
  pub fn focus_next(&mut self, focusable_ids: &[String]) {
    if focusable_ids.is_empty() {
        self.focused_id = None;
        return;
    }

    if let Some(current) = &self.focused_id {
        if let Some(idx) = focusable_ids.iter().position(|id| id == current) {
            let next_idx = (idx + 1) % focusable_ids.len();
            self.focused_id = Some(focusable_ids[next_idx].clone());
        } else {
            self.focused_id = Some(focusable_ids[0].clone());
        }
    } else {
        self.focused_id = Some(focusable_ids[0].clone());
    }
  }

  /// Cycles focus to the previous element in the list.
  pub fn focus_prev(&mut self, focusable_ids: &[String]) {
    if focusable_ids.is_empty() {
        self.focused_id = None;
        return;
    }

    if let Some(current) = &self.focused_id {
        if let Some(idx) = focusable_ids.iter().position(|id| id == current) {
            let prev_idx = if idx == 0 { focusable_ids.len() - 1 } else { idx - 1 };
            self.focused_id = Some(focusable_ids[prev_idx].clone());
        } else {
            self.focused_id = Some(focusable_ids.last().unwrap().clone());
        }
    } else {
        self.focused_id = Some(focusable_ids.last().unwrap().clone());
    }
  }

  /// Update state based on hit test result.
  pub fn handle_hit(&mut self, action_id: Option<String>) {
      if let Some(act) = action_id {
          self.hovered_action = Some(act);
          if self.is_pressed {
              self.active_action = self.hovered_action.clone();
              self.focused_id = self.hovered_action.clone();
          }
      } else {
          self.hovered_action = None;
          if self.is_pressed {
              self.focused_id = None;
          }
      }
  }

  /// Handle scroll event for a specific widget ID.
  pub fn handle_scroll(&mut self, id: &str, delta: Vec2) {
      let current = self.scroll_offsets.entry(id.to_string()).or_insert(Vec2::ZERO);
      current.x -= delta.x;
      current.y -= delta.y;
  }
}

/// Hit test result.
pub struct HitTestResult<'a> {
  pub widget: &'a Widget,
  pub action: String,
}
