//! UI loading and rendering from RON definitions.

use crate::interaction::HitTestResult;
use crate::interaction::InteractionState;
use crate::primitives::PrimitiveRenderer;
use crate::text::TextRenderer;
use crate::widget::Widget;
use glam::{Vec2, Vec4};
use std::fs;
use std::path::Path;
use winit::keyboard::{Key, NamedKey};
use winit::event::ElementState;

/// Loads a UI definition from a RON file.
///
/// # Arguments
/// * `path` - Path to the RON file
///
/// # Returns
/// The root widget, or an error if loading/parsing fails.
pub fn load_ui(path: impl AsRef<Path>) -> anyhow::Result<Widget> {
  let content = fs::read_to_string(path)?;
  let widget: Widget = ron::from_str(&content)?;
  Ok(widget)
}

/// Loads a UI definition from a RON string.
pub fn parse_ui(ron_str: &str) -> anyhow::Result<Widget> {
  let widget: Widget = ron::from_str(ron_str)?;
  Ok(widget)
}

use crate::image_renderer::ImageRenderer;
use crate::texture::Texture;
use std::collections::HashMap;

/// Render context for widget rendering.
pub struct RenderContext<'a> {
  pub primitives: &'a mut PrimitiveRenderer,
  pub text: &'a mut TextRenderer,
  pub images: &'a mut ImageRenderer,
  pub textures: &'a mut HashMap<String, Texture>,
  pub device: &'a wgpu::Device,
  pub queue: &'a wgpu::Queue,
  pub interaction: Option<&'a InteractionState>,
  /// Current offset for nested containers
  pub offset: Vec2,
  pub scissor_stack: Vec<Option<(u32, u32, u32, u32)>>,
  pub current_scissor: Option<(u32, u32, u32, u32)>,
}

impl<'a> RenderContext<'a> {
  /// Creates a new render context.
  pub fn new(
    primitives: &'a mut PrimitiveRenderer,
    text: &'a mut TextRenderer,
    images: &'a mut ImageRenderer,
    textures: &'a mut HashMap<String, Texture>,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    interaction: Option<&'a InteractionState>,
  ) -> Self {
    Self {
      primitives,
      text,
      images,
      textures,
      device,
      queue,
      interaction,
      offset: Vec2::ZERO,
      scissor_stack: Vec::new(),
      current_scissor: None,
    }
  }

  /// Pushes a new scissor rect, intersecting with the current one.
  pub fn push_scissor(&mut self, rect: Option<(u32, u32, u32, u32)>) {
      self.scissor_stack.push(self.current_scissor);
      
      if let Some(r) = rect {
          if let Some(current) = self.current_scissor {
              let x = r.0.max(current.0);
              let y = r.1.max(current.1);
              let right = (r.0 + r.2).min(current.0 + current.2);
              let bottom = (r.1 + r.3).min(current.1 + current.3);
              
              let w = if right > x { right - x } else { 0 };
              let h = if bottom > y { bottom - y } else { 0 };
              self.current_scissor = Some((x, y, w, h));
          } else {
              self.current_scissor = Some(r);
          }
      }
      
      self.update_renderers();
  }
  
  pub fn pop_scissor(&mut self) {
      if let Some(prev) = self.scissor_stack.pop() {
          self.current_scissor = prev;
          self.update_renderers();
      }
  }

  fn update_renderers(&mut self) {
    // This helper depends on `self.current_scissor`
    self.primitives.set_scissor(self.current_scissor);
    self.text.set_scissor(self.current_scissor);
    self.images.set_scissor(self.current_scissor);
  }
}

/// Renders a widget tree recursively.
pub fn render_widget(widget: &Widget, ctx: &mut RenderContext) {
  match widget {
    Widget::Container { id, children, bounds, padding: _, background, corner_radius, scrollable, .. } => {
      let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
      
      // Draw background
      if let Some(bc) = background {
          let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
          let half_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
          ctx.primitives.draw_rect(
            center,
            half_size,
            Vec4::new(bc.0, bc.1, bc.2, bc.3),
            [*corner_radius; 4],
            0.0,
          );
      }

      let mut child_offset = pos;
      let mut pushed_scissor = false;

      if *scrollable {
          let scroll = if let Some(wid) = id {
              ctx.interaction.and_then(|i| i.scroll_offsets.get(wid)).copied().unwrap_or(Vec2::ZERO)
          } else {
              Vec2::ZERO
          };
          
          child_offset = pos - scroll;

          let x = pos.x.max(0.0) as u32;
          let y = pos.y.max(0.0) as u32;
          let w = bounds.width.max(0.0) as u32;
          let h = bounds.height.max(0.0) as u32;
          
          ctx.push_scissor(Some((x, y, w, h)));
          pushed_scissor = true;
      }

      let old_offset = ctx.offset;
      ctx.offset = child_offset;
      for child in children {
          render_widget(child, ctx);
      }
      ctx.offset = old_offset;

      if pushed_scissor {
          ctx.pop_scissor();
      }
    }

    Widget::Label { text, x, y, size, color, .. } => {
      let pos = ctx.offset + Vec2::new(*x, *y);
      
      ctx.text.draw(
        ctx.device,
        ctx.queue,
        text,
        pos,
        *size,
        Vec4::new(color.0, color.1, color.2, color.3),
      );
    }

    Widget::Button {
      text,
      action,
      bounds,
      background,
      hover_color,
      active_color,
      border_width,
      border_color,
      corner_radius,
      ..
    } => {
      let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
      
      let mut color_to_use = *background;
      let mut active_border = None;

      if let Some(state) = ctx.interaction {
        if state.is_active(action) {
          color_to_use = *active_color;
          active_border = Some((0.5, 0.5, 0.5, 1.0)); // Light gray border active
        } else if state.is_hovered(action) {
          color_to_use = *hover_color;
          active_border = Some((0.6, 0.6, 0.6, 0.5)); // Subtle gray glow
        }
      }

      let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
      let half_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
      
      ctx.primitives.draw_rect(
        center,
        half_size,
        Vec4::new(color_to_use.0, color_to_use.1, color_to_use.2, color_to_use.3),
        [*corner_radius; 4],
        0.0,
      );

      // Draw border (configured or active/hover effect)
      if let Some(bc) = active_border {
           ctx.primitives.draw_rect(center, half_size, Vec4::new(bc.0, bc.1, bc.2, bc.3), [*corner_radius; 4], 2.0);
      } else if *border_width > 0.0 {
          if let Some(bc) = border_color {
              ctx.primitives.draw_rect(center, half_size, Vec4::new(bc.0, bc.1, bc.2, bc.3), [*corner_radius; 4], *border_width);
          }
      }

      let text_size = 16.0;
      let text_dims = ctx.text.measure(text, text_size);
      let text_pos = pos + Vec2::new((bounds.width - text_dims.x) * 0.5, (bounds.height - text_dims.y) * 0.5);
      // Bright text
      let text_col = Vec4::new(1.0, 1.0, 1.0, 1.0);

      ctx.text.draw(ctx.device, ctx.queue, text, text_pos, text_size, text_col);
    }

    Widget::TextInput {
      value,
      placeholder,
      id,
      font_size,
      text_align,
      bounds,
      background,
      border_width,
      border_color,
      ..
    } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
        let half_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);

        let is_focused = ctx.interaction.map(|s| s.focused_id.as_deref() == Some(id)).unwrap_or(false);
        // Darker bg when not focused, slightly lighter when focused
        let bg = background.unwrap_or_else(|| if is_focused { (0.15, 0.15, 0.18, 1.0) } else { (0.1, 0.1, 0.12, 1.0) });
        
        ctx.primitives.draw_rect(center, half_size, Vec4::new(bg.0, bg.1, bg.2, bg.3), [4.0; 4], 0.0);
        
        // Focus border - White/Gray
        if is_focused {
             ctx.primitives.draw_rect(center, half_size, Vec4::new(0.75, 0.75, 0.75, 1.0), [4.0; 4], 1.5);
        } else if *border_width > 0.0 {
            if let Some(bc) = border_color {
                ctx.primitives.draw_rect(center, half_size, Vec4::new(bc.0, bc.1, bc.2, bc.3), [4.0; 4], *border_width);
            }
        }

        let text = if value.is_empty() { placeholder } else { value };
        let col = if value.is_empty() { Vec4::new(0.5, 0.5, 0.6, 1.0) } else { Vec4::new(0.9, 0.9, 0.95, 1.0) };
        let size = if *font_size > 0.0 { *font_size } else { 14.0 };
        
        let text_dims = ctx.text.measure(text, size);
        let align_x = match text_align {
            crate::widget::TextAlign::Left => 10.0,
            crate::widget::TextAlign::Center => (bounds.width - text_dims.x) * 0.5,
            crate::widget::TextAlign::Right => bounds.width - text_dims.x - 10.0,
        };
        let text_pos = pos + Vec2::new(align_x, (bounds.height - size) * 0.5);
        ctx.text.draw(ctx.device, ctx.queue, text, text_pos, size, col);
        
        // Draw cursor if focused - White
        if is_focused {
            let cursor_x = if value.is_empty() {
                align_x
            } else {
                let val_dims = ctx.text.measure(value, size);
                align_x + val_dims.x + 2.0
            };
            
            let cursor_pos = pos + Vec2::new(cursor_x + 1.0, bounds.height * 0.5);
            ctx.primitives.draw_rect(
                cursor_pos, 
                Vec2::new(1.0, size * 0.8 * 0.5), 
                Vec4::new(0.8, 0.8, 0.8, 1.0), 
                [0.0; 4], 
                0.0
            );
        }
    }

    Widget::Spacer { .. } => {}

    Widget::Checkbox { checked, size, color, check_color, bounds, .. } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
        let hs = *size * 0.5;
        ctx.primitives.draw_rect(center, Vec2::new(hs, hs), Vec4::new(color.0, color.1, color.2, color.3), [4.0; 4], 0.0);
        if *checked {
            let cs = *size * 0.3;
            ctx.primitives.draw_rect(center, Vec2::new(cs, cs), Vec4::new(check_color.0, check_color.1, check_color.2, check_color.3), [1.0; 4], 0.0);
        }
    }

    Widget::Image { path, bounds, .. } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
        let size = Vec2::new(bounds.width, bounds.height);
        
        if !ctx.textures.contains_key(path) {
            if let Ok(tex) = Texture::from_path(ctx.device, ctx.queue, path) {
                ctx.textures.insert(path.clone(), tex);
            }
        }
        
        if let Some(tex) = ctx.textures.get(path) {
            ctx.images.draw(ctx.device, tex, center, size, Vec4::ONE);
        }
    }

    Widget::Icon { icon_name, color, bounds, .. } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
        let size = Vec2::new(bounds.width, bounds.height);
        
        if let Some(tex) = ctx.textures.get(icon_name) {
             let tint = if let Some(c) = color {
                 Vec4::new(c.0, c.1, c.2, c.3)
             } else {
                 Vec4::ONE
             };
            ctx.images.draw(ctx.device, tex, center, size, tint);
        }
    }

    Widget::Slider { value, min, max, track_height, thumb_radius, active_color, inactive_color, bounds, .. } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let cy = pos.y + bounds.height * 0.5;
        let pct = ((*value - *min) / (*max - *min)).clamp(0.0, 1.0);
        
        ctx.primitives.draw_rect(Vec2::new(pos.x + bounds.width * 0.5, cy), Vec2::new(bounds.width * 0.5, *track_height * 0.5), Vec4::new(inactive_color.0, inactive_color.1, inactive_color.2, inactive_color.3), [*track_height * 0.5; 4], 0.0);
        let aw = bounds.width * pct;
        if aw > 0.0 {
            ctx.primitives.draw_rect(Vec2::new(pos.x + aw * 0.5, cy), Vec2::new(aw * 0.5, *track_height * 0.5), Vec4::new(active_color.0, active_color.1, active_color.2, active_color.3), [*track_height * 0.5; 4], 0.0);
        }
        ctx.primitives.draw_circle(Vec2::new(pos.x + aw, cy), *thumb_radius, Vec4::ONE, 0.0);
    }
  }
}

/// Convenience function to render a widget with the renderer.
pub fn render_ui(
  widget: &Widget,
  renderer: &mut crate::renderer::GloomyRenderer,
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  interaction: Option<&InteractionState>,
) {
  let (primitives, text, images, textures) = renderer.split_mut();
  let mut ctx = RenderContext::new(primitives, text, images, textures, device, queue, interaction);
  render_widget(widget, &mut ctx);
}

/// Performs a hit test on the widget tree.
///
/// Returns the first interactive widget found under the given point.
pub fn hit_test<'a>(
  widget: &'a Widget,
  point: Vec2,
  scroll_offsets: Option<&std::collections::HashMap<String, Vec2>>,
) -> Option<HitTestResult<'a>> {
  match widget {
    Widget::Container { id, scrollable, bounds, children, .. } => {
      // Check if point is inside container bounds first (clipping check)
      // If scrollable, hits outside should be ignored regardless of overflow.
      if *scrollable {
          if point.x < bounds.x || point.x > bounds.x + bounds.width ||
             point.y < bounds.y || point.y > bounds.y + bounds.height {
              return None;
          }
      }

      // Transform point to local space
      let mut local_point = point - Vec2::new(bounds.x, bounds.y);

      // Apply scroll offset
      if *scrollable {
          if let Some(offsets) = scroll_offsets {
              if let Some(wid) = id {
                  if let Some(scroll) = offsets.get(wid) {
                      local_point += *scroll;
                  }
              }
          }
      }

      // Check children in reverse order (top to bottom)
      for child in children.iter().rev() {
        if let Some(result) = hit_test(child, local_point, scroll_offsets) {
          return Some(result);
        }
      }
      None
    }
    Widget::Button { bounds, action, .. } => {
        // Simple bounding box check
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action })
           } else {
             None
           }
    }
    Widget::TextInput { bounds, id, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: id })
           } else {
             None
           }
    }
    Widget::Checkbox { bounds, id, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: id })
        } else {
             None
        }
    }
    Widget::Slider { bounds, id, .. } => {
         if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: id })
        } else {
             None
        }
    }
    _ => None,
  }

}

// Helper to find a widget by ID (or Action) and return mutable access.
// Recursive search.
pub fn find_widget_mut<'a>(root: &'a mut Widget, id: &str) -> Option<&'a mut Widget> {
    match root {
        Widget::TextInput { id: w_id, .. } 
        | Widget::Button { action: w_id, .. } 
        | Widget::Checkbox { id: w_id, .. } 
        | Widget::Slider { id: w_id, .. } => {
            if w_id == id {
                return Some(root);
            }
        },
        Widget::Container { children, .. } => {
            for child in children.iter_mut() {
                if let Some(w) = find_widget_mut(child, id) {
                    return Some(w);
                }
            }
        },
        _ => {}
    }
    None
}

/// Collects all focusable IDs from the widget tree in depth-first order.
pub fn get_focusable_ids(widget: &Widget) -> Vec<String> {
    let mut ids = Vec::new();
    collect_focusable_ids_recursive(widget, &mut ids);
    ids
}

fn collect_focusable_ids_recursive(widget: &Widget, ids: &mut Vec<String>) {
    // If this widget is focusable, add its ID
    if let Some(id) = widget.get_focusable_id() {
        ids.push(id.to_string());
    }

    // If it's a container, recurse into children
    if let Widget::Container { children, .. } = widget {
        for child in children {
            collect_focusable_ids_recursive(child, ids);
        }
    }
}
  
/// Handles widget interactions (toggles, sliders) based on input state.
/// Modifies the widget tree in-place.
pub fn handle_interactions(
  widget: &mut Widget,
  ctx: &crate::interaction::InteractionState,
  offset: Vec2,
) -> bool {
    let mut changed = false;

    match widget {
        Widget::Container { children, bounds, padding, id, scrollable, .. } => {
            let my_pos = offset + Vec2::new(bounds.x, bounds.y);
            
             let scroll_off = if *scrollable {
                id.as_ref().and_then(|i| ctx.scroll_offsets.get(i)).copied().unwrap_or(Vec2::ZERO)
            } else {
                Vec2::ZERO
            };
            
            // Layout engine bounds include padding for children positioning.
            // Render logic: pos - scroll.
            let child_base = my_pos - scroll_off;
            
            for child in children {
                if handle_interactions(child, ctx, child_base) {
                    changed = true;
                }
            }
        }
        
        Widget::Checkbox { id, bounds, checked, .. } => {
             // Check if clicked
             if ctx.clicked_id.as_deref() == Some(id) {
                 *checked = !*checked; // Toggle
                 changed = true;
             }
        }
        
        Widget::Slider { id, bounds, value, min, max, .. } => {
            let my_pos = offset + Vec2::new(bounds.x, bounds.y);
            // Check if active (being dragged) or clicked
             let is_dragging = ctx.is_active(id) && ctx.is_pressed;
             // OR just clicked this frame?
             // hit_test handles initial click.
             
             if is_dragging {
                 let w = bounds.width;
                 if w > 0.0 {
                     let mouse_x = ctx.mouse_pos.x;
                     let local_x = mouse_x - my_pos.x;
                     let pct = (local_x / w).clamp(0.0, 1.0);
                     let new_val = *min + pct * (*max - *min);
                     
                     if (*value - new_val).abs() > 1e-4 {
                         *value = new_val;
                         changed = true;
                     }
                 }
             }
        }
        
        _ => {}
    }
    
    changed
}

/// Handles keyboard events for the UI system.
///
/// Returns true if the event was handled and the UI needs a redraw.
pub fn handle_keyboard_event(
  root: &mut Widget,
  interaction: &mut InteractionState,
  event: &winit::event::KeyEvent,
) -> bool {
    if event.state != ElementState::Pressed {
        return false;
    }

    let mut changed = false;

    // 1. Handle focus cycling (Tab)
    if let Key::Named(NamedKey::Tab) = &event.logical_key {
        let focusable_ids = get_focusable_ids(root);
        // Shift check for winit
        // We'd need modifiers state for true Shift-Tab, but for now just Tab.
        // If we want Shift-Tab we need more info.
        interaction.focus_next(&focusable_ids);
        return true; 
    }

    // 2. Dispatch to focused widget
    if let Some(focused_id) = interaction.focused_id.clone() {
        if let Some(widget) = find_widget_mut(root, &focused_id) {
            changed = handle_text_input_to_widget(widget, event);
        }
    }

    changed
}

fn handle_text_input_to_widget(widget: &mut Widget, event: &winit::event::KeyEvent) -> bool {
    let mut changed = false;
    
    if let Widget::TextInput { value, .. } = widget {
        match &event.logical_key {
            Key::Named(NamedKey::Backspace) => {
                if value.pop().is_some() {
                    changed = true;
                }
            }
            Key::Character(c) => {
                if !c.chars().any(|ch| ch.is_control()) {
                    value.push_str(c);
                    changed = true;
                }
            }
            Key::Named(NamedKey::Space) => {
                value.push(' ');
                changed = true;
            }
            _ => {}
        }
    }
    
    changed
}
