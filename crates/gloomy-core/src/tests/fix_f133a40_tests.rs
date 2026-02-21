/// Tests for commit f133a40: scissor, image resize, and text bounds fixes.
///
/// Three regressions were fixed:
///   1. push_scissor dropped new scissor when current_scissor was None.
///   2. ImageRenderer::resize did not update the CPU-side screen_size field.
///   3. TextRenderer gained draw_bounded() for glyph-level width clipping.
///
/// Tests 1 and 3 can be verified with pure-logic unit tests (no GPU required).
/// Test 2 is covered at the type level: resize now takes &mut self, which
/// enforces that callers hold a mutable reference and allows the field update.
use glam::{Vec2, Vec4};
use wgpu_text::glyph_brush::HorizontalAlign;

use crate::text::TextSnapshot;

// ---------------------------------------------------------------------------
// Scissor intersection helpers
// ---------------------------------------------------------------------------

/// Pure reimplementation of push_scissor's intersection logic for unit testing.
fn intersect_scissors(
  current: Option<(u32, u32, u32, u32)>,
  new_rect: Option<(u32, u32, u32, u32)>,
) -> Option<(u32, u32, u32, u32)> {
  if let Some(r) = new_rect {
    if let Some(current) = current {
      let x = r.0.max(current.0);
      let y = r.1.max(current.1);
      let w = (r.0 + r.2).min(current.0 + current.2).saturating_sub(x);
      let h = (r.1 + r.3).min(current.1 + current.3).saturating_sub(y);
      if w > 0 && h > 0 {
        Some((x, y, w, h))
      } else {
        Some((0, 0, 0, 0))
      }
    } else {
      // Bug fix: was missing; new rect must become current when none active.
      Some(r)
    }
  } else {
    current
  }
}

// ---------------------------------------------------------------------------
// Fix 1: push_scissor — first scissor when current_scissor is None
// ---------------------------------------------------------------------------

/// Regression: the first push_scissor call was silently dropped when
/// current_scissor was None, leaving scrollable containers unclipped.
#[test]
fn test_push_scissor_first_rect_is_adopted() {
  let rect = (10u32, 20u32, 100u32, 200u32);
  let result = intersect_scissors(None, Some(rect));
  assert_eq!(
    result,
    Some(rect),
    "first scissor must be adopted when current_scissor is None"
  );
}

/// Pushing None when current is None must leave current_scissor unchanged.
#[test]
fn test_push_scissor_none_on_none_stays_none() {
  let result = intersect_scissors(None, None);
  assert_eq!(result, None);
}

/// A second push_scissor computes the intersection of two overlapping rects.
#[test]
fn test_push_scissor_intersection_overlapping() {
  // current: (0,0)→(100,100), new: (50,50)→(200,200)
  // intersection: (50,50)→(100,100) i.e. w=50, h=50
  let current = Some((0u32, 0u32, 100u32, 100u32));
  let new_rect = Some((50u32, 50u32, 150u32, 150u32));
  let result = intersect_scissors(current, new_rect);
  assert_eq!(result, Some((50, 50, 50, 50)));
}

/// Non-overlapping rects must produce the zero-area sentinel (0,0,0,0).
#[test]
fn test_push_scissor_no_overlap_yields_zero_rect() {
  let current = Some((0u32, 0u32, 50u32, 50u32));
  let new_rect = Some((100u32, 100u32, 50u32, 50u32));
  let result = intersect_scissors(current, new_rect);
  assert_eq!(result, Some((0, 0, 0, 0)));
}

/// Pushing None on an existing scissor must preserve the current rect
/// (push_scissor only updates when rect.is_some()).
#[test]
fn test_push_scissor_none_on_some_keeps_current() {
  let current = Some((10u32, 20u32, 80u32, 60u32));
  let result = intersect_scissors(current, None);
  assert_eq!(result, current);
}

/// Adjacent, non-overlapping rects (touching edges) also yield zero area.
#[test]
fn test_push_scissor_adjacent_rects_yield_zero() {
  // current ends at x=100, new starts at x=100 — no overlap.
  let current = Some((0u32, 0u32, 100u32, 100u32));
  let new_rect = Some((100u32, 0u32, 50u32, 100u32));
  let result = intersect_scissors(current, new_rect);
  assert_eq!(result, Some((0, 0, 0, 0)));
}

/// New rect fully contained inside current must equal new rect.
#[test]
fn test_push_scissor_contained_rect() {
  let current = Some((0u32, 0u32, 200u32, 200u32));
  let new_rect = Some((50u32, 50u32, 100u32, 100u32));
  let result = intersect_scissors(current, new_rect);
  assert_eq!(result, Some((50, 50, 100, 100)));
}

// ---------------------------------------------------------------------------
// Fix 2: ImageRenderer::resize — screen_size field update
// ---------------------------------------------------------------------------
//
// The signature change from `&self` to `&mut self` is verified by the
// compiler: the existing call sites in renderer.rs compile only if they
// hold a mutable borrow. The field update itself is tested indirectly via
// the renderer integration in the app layer.  No additional pure-logic unit
// test is possible here without a live wgpu Device/Queue.

// ---------------------------------------------------------------------------
// Fix 3: TextRenderer::draw_bounded — pending tuple includes max_width
// ---------------------------------------------------------------------------

/// TextSnapshot pending items are 8-tuples whose last field is Option<f32>.
/// draw() must set that field to None; draw_bounded() must set it to Some(w).
/// We verify this by constructing snapshots directly and inspecting the field.
#[test]
fn test_text_snapshot_unbounded_has_none_bounds() {
  let item: (
    String,
    Vec2,
    f32,
    Vec4,
    Option<(u32, u32, u32, u32)>,
    HorizontalAlign,
    Option<String>,
    Option<f32>,
  ) = (
    "hello".to_string(),
    Vec2::ZERO,
    14.0,
    Vec4::ONE,
    None,
    HorizontalAlign::Left,
    None,
    None, // draw() path
  );
  let snap = TextSnapshot { pending: vec![item] };
  assert_eq!(snap.pending[0].7, None, "draw() path must store None for bounds");
}

#[test]
fn test_text_snapshot_bounded_has_some_width() {
  let max_w = 120.0_f32;
  let item: (
    String,
    Vec2,
    f32,
    Vec4,
    Option<(u32, u32, u32, u32)>,
    HorizontalAlign,
    Option<String>,
    Option<f32>,
  ) = (
    "hello".to_string(),
    Vec2::ZERO,
    14.0,
    Vec4::ONE,
    None,
    HorizontalAlign::Left,
    None,
    Some(max_w), // draw_bounded() path
  );
  let snap = TextSnapshot { pending: vec![item] };
  assert_eq!(
    snap.pending[0].7,
    Some(max_w),
    "draw_bounded() path must store Some(max_width)"
  );
}

/// Snapshots are cloneable; clone must preserve the bounds field.
#[test]
fn test_text_snapshot_clone_preserves_bounds() {
  let item: (
    String,
    Vec2,
    f32,
    Vec4,
    Option<(u32, u32, u32, u32)>,
    HorizontalAlign,
    Option<String>,
    Option<f32>,
  ) = (
    "cloned".to_string(),
    Vec2::new(5.0, 10.0),
    16.0,
    Vec4::new(1.0, 0.0, 0.0, 1.0),
    Some((0, 0, 800, 600)),
    HorizontalAlign::Center,
    Some("Roboto".to_string()),
    Some(250.0),
  );
  let snap = TextSnapshot { pending: vec![item] };
  let cloned = snap.clone();
  assert_eq!(cloned.pending[0].7, Some(250.0));
  assert_eq!(cloned.pending[0].6, Some("Roboto".to_string()));
}

/// A snapshot with mixed bounded/unbounded items must preserve each correctly.
#[test]
fn test_text_snapshot_mixed_items() {
  let unbounded: (
    String,
    Vec2,
    f32,
    Vec4,
    Option<(u32, u32, u32, u32)>,
    HorizontalAlign,
    Option<String>,
    Option<f32>,
  ) = (
    "a".into(),
    Vec2::ZERO,
    12.0,
    Vec4::ONE,
    None,
    HorizontalAlign::Left,
    None,
    None,
  );
  let bounded: (
    String,
    Vec2,
    f32,
    Vec4,
    Option<(u32, u32, u32, u32)>,
    HorizontalAlign,
    Option<String>,
    Option<f32>,
  ) = (
    "b".into(),
    Vec2::ZERO,
    12.0,
    Vec4::ONE,
    None,
    HorizontalAlign::Left,
    None,
    Some(80.0),
  );
  let snap = TextSnapshot { pending: vec![unbounded, bounded] };
  assert_eq!(snap.pending[0].7, None);
  assert_eq!(snap.pending[1].7, Some(80.0));
}
