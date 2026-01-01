//! UI loading and rendering from RON definitions.

use crate::interaction::HitTestResult;
use crate::interaction::InteractionState;
use crate::primitives::PrimitiveRenderer;
use crate::text::TextRenderer;
use crate::widget::{Widget, TextAlign, WidgetBounds};
use crate::layout::Layout;
use wgpu_text::glyph_brush::HorizontalAlign;
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
  pub surface_width: u32,
  pub surface_height: u32,
  pub overlay_queue: Vec<(Widget, Vec2)>, // Widget + Absolute Position
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
    surface_width: u32,
    surface_height: u32,
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
      surface_width,
      surface_height,
      overlay_queue: Vec::new(),
    }
  }

  /// Pushes a new scissor rect, intersecting with the current one.
  pub fn push_scissor(&mut self, rect: Option<(u32, u32, u32, u32)>) {
      self.scissor_stack.push(self.current_scissor);
      
      if let Some(r) = rect {
          if let Some(current) = self.current_scissor {
              let x = r.0.max(current.0);
              let y = r.1.max(current.1);
              let right = (r.0 + r.2).min(current.0 + current.2).min(self.surface_width);
              let bottom = (r.1 + r.3).min(current.1 + current.3).min(self.surface_height);
              
              let w = if right > x { right - x } else { 0 };
              let h = if bottom > y { bottom - y } else { 0 };
              self.current_scissor = Some((x, y, w, h));
          } else {
              // Clamp to surface
              let x = r.0.min(self.surface_width);
              let y = r.1.min(self.surface_height);
              let w = r.2.min(self.surface_width - x);
              let h = r.3.min(self.surface_height - y);
              self.current_scissor = Some((x, y, w, h));
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
    Widget::ToggleSwitch { id, checked, style, bounds, .. } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
        let track_h = if style.track_height > 0.0 { style.track_height } else { 20.0 };
        let thumb_r = if style.thumb_radius > 0.0 { style.thumb_radius } else { 8.0 };
        let track_col = if *checked { style.track_color_on.unwrap_or((0.2, 0.8, 0.4, 1.0)) } else { style.track_color_off.unwrap_or((0.2, 0.2, 0.25, 1.0)) };
        let thumb_col = style.thumb_color.unwrap_or((0.9, 0.9, 0.95, 1.0));
        ctx.primitives.draw_rect(center, Vec2::new(bounds.width * 0.5, track_h * 0.5), Vec4::new(track_col.0, track_col.1, track_col.2, track_col.3), [track_h * 0.5; 4], 0.0);
        let pad = 2.0;
        let travel = bounds.width - (thumb_r * 2.0) - (pad * 2.0);
        let offset_x = if *checked { travel } else { 0.0 };
        let thumb_x = pad + thumb_r + offset_x - (bounds.width * 0.5); // Relative to center?
        // Primitive renderer draw_circle takes center pos.
        let thumb_pos = pos + Vec2::new(pad + thumb_r + offset_x, bounds.height * 0.5); 
        ctx.primitives.draw_circle(thumb_pos, thumb_r, Vec4::new(thumb_col.0, thumb_col.1, thumb_col.2, thumb_col.3), 0.0);
    }
    Widget::ProgressBar { value, min, max, style, bounds, .. } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
        let bg = style.background_color.unwrap_or((0.15, 0.15, 0.18, 1.0));
        let cr = style.corner_radius;
        ctx.primitives.draw_rect(center, Vec2::new(bounds.width * 0.5, bounds.height * 0.5), Vec4::new(bg.0, bg.1, bg.2, bg.3), [cr; 4], 0.0);
        let range = max - min;
        let pct = if range > 0.0 { ((*value - min) / range).clamp(0.0, 1.0) } else { 0.0 };
        if pct > 0.0 {
            let fill_w = bounds.width * pct;
            let fill_center = pos + Vec2::new(fill_w * 0.5, bounds.height * 0.5);
            let fill_col = style.fill_color.unwrap_or((0.3, 0.5, 0.9, 1.0));
            ctx.primitives.draw_rect(fill_center, Vec2::new(fill_w * 0.5, bounds.height * 0.5), Vec4::new(fill_col.0, fill_col.1, fill_col.2, fill_col.3), [cr; 4], 0.0);
        }
    }
    Widget::RadioButton { selected, style, bounds, .. } => {
         let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
         let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
         let size = if style.size > 0.0 { style.size } else { 20.0 };
         let radius = size * 0.5;
         let outer_col = style.outer_color.unwrap_or((0.5, 0.5, 0.5, 1.0));
         ctx.primitives.draw_circle(center, radius, Vec4::new(outer_col.0, outer_col.1, outer_col.2, outer_col.3), 0.0);
         let inner_bg = (0.1, 0.1, 0.12, 1.0);
         ctx.primitives.draw_circle(center, radius - 2.0, Vec4::new(inner_bg.0, inner_bg.1, inner_bg.2, inner_bg.3), 0.0);
         if *selected {
             let inner_col = style.inner_color.unwrap_or((0.2, 0.8, 0.4, 1.0));
             ctx.primitives.draw_circle(center, radius - 6.0, Vec4::new(inner_col.0, inner_col.1, inner_col.2, inner_col.3), 0.0);
         }
    }
    Widget::Dropdown { id, options, expanded, style, bounds, selected_index, width, height, .. } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let w = if let Some(w) = width { *w } else { bounds.width };
        let h = if let Some(h) = height { *h } else { bounds.height };
        let center = pos + Vec2::new(w * 0.5, h * 0.5);
        let bg = style.background.unwrap_or((0.2, 0.2, 0.25, 1.0));
        let cr = style.corner_radius;
        ctx.primitives.draw_rect(center, Vec2::new(w * 0.5, h * 0.5), Vec4::new(bg.0, bg.1, bg.2, bg.3), [cr; 4], 0.0);
        let text_idx = selected_index.unwrap_or(0);
        let label = if !options.is_empty() && text_idx < options.len() { &options[text_idx] } else { "" };
        let text_col = style.text_color.unwrap_or((0.9, 0.9, 0.9, 1.0));
        let text_vec = Vec4::new(text_col.0, text_col.1, text_col.2, text_col.3);
        let text_pos = pos + Vec2::new(10.0, (h - 16.0) * 0.5);
        ctx.text.draw(ctx.device, ctx.queue, label, text_pos, 16.0, text_vec, HorizontalAlign::Left, None);
        let arrow_pos = ctx.offset + Vec2::new(bounds.x + bounds.width - 15.0, bounds.y + 15.0);
        ctx.text.draw(ctx.device, ctx.queue, "v", arrow_pos, 12.0, text_vec, HorizontalAlign::Center, None);
        
        if *expanded {
            let mut list_children = Vec::new();
            for (i, opt) in options.iter().enumerate() {
                let action = format!("select_{}_{}", id, i);
                let is_sel = Some(i) == *selected_index;
                let bg_col = if is_sel { (0.3, 0.3, 0.4, 1.0) } else { (0.25, 0.25, 0.3, 1.0) };
                let item_h = 30.0;
                let btn = Widget::Button {
                    text: opt.clone(), action, bounds: WidgetBounds { x: 0.0, y: 0.0, width: w, height: item_h },
                    background: bg_col, hover_color: (0.35, 0.35, 0.4, 1.0), active_color: (0.4, 0.4, 0.5, 1.0),
                    border: None, corner_radius: 0.0, shadow: None, gradient: None, layout: Layout::default(),
                    flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, corner_radii: None,
                    font: None,
                };
                list_children.push(btn);
            }
            let list_height = (list_children.len() as f32) * 30.0;
            let dropdown_list = Widget::Container {
                id: Some(format!("{}_list", id)), scrollable: false,
                bounds: WidgetBounds { x: 0.0, y: 0.0, width: w, height: list_height },
                width: Some(w), height: Some(list_height), background: Some((0.2, 0.2, 0.25, 1.0)),
                border: Some(crate::widget::Border { width: 1.0, color: (0.1, 0.1, 0.1, 1.0), ..Default::default() }),
                corner_radius: 0.0, shadow: Some(crate::widget::Shadow { offset: (0.0, 4.0), blur: 8.0, color: (0.0, 0.0, 0.0, 0.5) }),
                gradient: None, padding: 0.0, layout: crate::layout::Layout { direction: crate::layout::Direction::Column, ..Default::default() },
                flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, corner_radii: None,
                children: list_children,
            };
            let overlay_pos = pos + Vec2::new(0.0, h);
            ctx.overlay_queue.push((dropdown_list, overlay_pos));
        }
    }
    Widget::Container { id, children, bounds, padding: _, background, corner_radius, scrollable, shadow, gradient, border, .. } => {
      let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);

      // 1. Draw Shadow
      if let Some(shadow) = shadow {
          let shadow_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
          let shadow_pos = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5) + Vec2::new(shadow.offset.0, shadow.offset.1);
          
          ctx.primitives.draw_styled_rect(
              shadow_pos,
              shadow_size,
              Vec4::new(shadow.color.0, shadow.color.1, shadow.color.2, shadow.color.3),
              Vec4::new(shadow.color.0, shadow.color.1, shadow.color.2, shadow.color.3),
              [*corner_radius; 4],
              0.0,
              shadow.blur,
          );
      }

      // 2. Draw Background
      if let Some(bc) = background {
          let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
          let half_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
          
          let (col_start, col_end) = if let Some(grad) = gradient {
              (Vec4::new(grad.start.0, grad.start.1, grad.start.2, grad.start.3),
               Vec4::new(grad.end.0, grad.end.1, grad.end.2, grad.end.3))
          } else {
              let c = Vec4::new(bc.0, bc.1, bc.2, bc.3);
              (c, c)
          };

          ctx.primitives.draw_styled_rect(
            center,
            half_size,
            col_start,
            col_end,
            [*corner_radius; 4],
            0.0,
            0.0
          );
      }
      
      // 3. Draw Border
      if let Some(border) = border {
          let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
          let half_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
          
          let (col_start, col_end) = if let Some(grad) = border.gradient {
              (Vec4::new(grad.start.0, grad.start.1, grad.start.2, grad.start.3),
               Vec4::new(grad.end.0, grad.end.1, grad.end.2, grad.end.3))
          } else {
              let c = Vec4::new(border.color.0, border.color.1, border.color.2, border.color.3);
              (c, c)
          };

          ctx.primitives.draw_styled_rect(
            center,
            half_size,
            col_start,
            col_end,
            [*corner_radius; 4],
            border.width,
            0.0
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

    Widget::Label { text, x, y, size, color, text_align, width, height, font, .. } => {
      // Set scissor to clip text within label bounds
      let scissor_x = (ctx.offset.x + x).max(0.0) as u32;
      let scissor_y = (ctx.offset.y + y).max(0.0) as u32;
      let scissor_w = (*width).max(0.0) as u32;
      let scissor_h = (*height).max(0.0) as u32;
      
      let old_scissor = if scissor_w > 0 && scissor_h > 0 {
        Some(ctx.text.set_scissor(Some((scissor_x, scissor_y, scissor_w, scissor_h))))
      } else {
        None
      };
      
      let align = match text_align {
          TextAlign::Left => HorizontalAlign::Left,
          TextAlign::Center => HorizontalAlign::Center,
          TextAlign::Right => HorizontalAlign::Right,
      };
      
      let mut text_pos = ctx.offset + Vec2::new(*x, *y);
      if *text_align == TextAlign::Center {
          text_pos.x += width * 0.5;
      } else if *text_align == TextAlign::Right {
          text_pos.x += width;
      }

      ctx.text.draw(
        ctx.device,
        ctx.queue,
        text,
        text_pos,
        *size,
        Vec4::new(color.0, color.1, color.2, color.3),
        align,
        font.as_deref(),
      );
      
      // Restore scissor
      if let Some(prev) = old_scissor {
        ctx.text.set_scissor(prev);
      }
    }

    Widget::Button {
      text,
      action,
      bounds,
      background,
      hover_color,
      active_color,
      border,
      corner_radius,
      shadow,
      gradient,
      font,
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

      // Button Shadow
      if let Some(shadow) = shadow {
          let shadow_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
          let map_center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
          let shadow_pos = map_center + Vec2::new(shadow.offset.0, shadow.offset.1);
          
          ctx.primitives.draw_styled_rect(
              shadow_pos,
              shadow_size,
              Vec4::new(shadow.color.0, shadow.color.1, shadow.color.2, shadow.color.3),
              Vec4::new(shadow.color.0, shadow.color.1, shadow.color.2, shadow.color.3),
              [*corner_radius; 4],
              0.0,
              shadow.blur,
          );
      }

      let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
      let half_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
      
      // Button Gradient or Solid
      let (mut c_start, mut c_end) = if let Some(grad) = gradient {
           (Vec4::new(grad.start.0, grad.start.1, grad.start.2, grad.start.3),
            Vec4::new(grad.end.0, grad.end.1, grad.end.2, grad.end.3))
      } else {
           let c = Vec4::new(color_to_use.0, color_to_use.1, color_to_use.2, color_to_use.3);
           (c, c)
      };
      
      // Hover overlay (if gradient)
      if let Some(state) = ctx.interaction {
           if state.is_hovered(action) && gradient.is_some() && !state.is_active(action) {
               // Lighten slightly
               c_start += Vec4::splat(0.1);
               c_end += Vec4::splat(0.1);
           }
      }

      ctx.primitives.draw_styled_rect(
        center,
        half_size,
        c_start,
        c_end,
        [*corner_radius; 4],
        0.0,
        0.0
      );

      // Draw border (configured or active/hover effect)
      if let Some(bc) = active_border {
           ctx.primitives.draw_rect(center, half_size, Vec4::new(bc.0, bc.1, bc.2, bc.3), [*corner_radius; 4], 2.0);
      } else if let Some(border) = border {
          // Gradient border support for button
          let (col_start, col_end) = if let Some(grad) = border.gradient {
              (Vec4::new(grad.start.0, grad.start.1, grad.start.2, grad.start.3),
               Vec4::new(grad.end.0, grad.end.1, grad.end.2, grad.end.3))
          } else {
              let c = Vec4::new(border.color.0, border.color.1, border.color.2, border.color.3);
              (c, c)
          };
          
          ctx.primitives.draw_styled_rect(
            center,
            half_size,
            col_start,
            col_end,
            [*corner_radius; 4],
            border.width,
            0.0
          );
      }

      let text_size = 16.0;
      let text_dims = ctx.text.measure(text, text_size, font.as_deref());
      let text_pos = pos + Vec2::new((bounds.width - text_dims.x) * 0.5, (bounds.height - text_dims.y) * 0.5);
      // Bright text
      let text_col = Vec4::new(1.0, 1.0, 1.0, 1.0);

      ctx.text.draw(ctx.device, ctx.queue, text, text_pos, text_size, text_col, HorizontalAlign::Left, font.as_deref());
    }

    Widget::TextInput {
      value,
      placeholder,
      id,
      font_size,
      text_align,
      style,
      bounds,
      ..
    } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
        let half_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);

        let is_focused = ctx.interaction.map(|s| s.focused_id.as_deref() == Some(id)).unwrap_or(false);
        
        // Background
        let bg_color = if is_focused {
             style.background_focused.unwrap_or(style.background.unwrap_or((0.15, 0.15, 0.18, 1.0)))
        } else {
             style.background.unwrap_or((0.1, 0.1, 0.12, 1.0))
        };
        ctx.primitives.draw_rect(center, half_size, Vec4::new(bg_color.0, bg_color.1, bg_color.2, bg_color.3), [4.0; 4], 0.0);
        
        // Border
        let border = if is_focused {
            style.border_focused.as_ref().or(style.border.as_ref())
        } else {
            style.border.as_ref()
        };
        
        if let Some(b) = border {
             ctx.primitives.draw_rect(center, half_size, Vec4::new(b.color.0, b.color.1, b.color.2, b.color.3), [4.0; 4], b.width);
        } else if is_focused && style.border_focused.is_none() {
             // Default focus border if none specified
             ctx.primitives.draw_rect(center, half_size, Vec4::new(0.75, 0.75, 0.75, 1.0), [4.0; 4], 1.5);
        }

        let text = if value.is_empty() { placeholder } else { value };
        let col = if value.is_empty() { Vec4::new(0.5, 0.5, 0.6, 1.0) } else { Vec4::new(0.9, 0.9, 0.95, 1.0) };
        let size = if *font_size > 0.0 { *font_size } else { 14.0 };
        
        let text_dims = ctx.text.measure(text, size, style.font.as_deref());
        let text_pos = pos + Vec2::new(8.0, (bounds.height - text_dims.y) * 0.5);

        ctx.text.draw(ctx.device, ctx.queue, text, text_pos, size, col, HorizontalAlign::Left, style.font.as_deref());
        
        // Draw cursor if focused - White
        if is_focused {
            let align_x = 8.0;
            let cursor_x = if value.is_empty() {
                align_x
            } else {
                let val_dims = ctx.text.measure(value, size, style.font.as_deref());
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
    Widget::Divider { bounds, orientation, thickness, color, margin, .. } => {
      use crate::widget::Orientation;
      let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
      
      // Draw divider based on orientation
      match orientation {
        Orientation::Horizontal => {
          let line_pos = Vec2::new(pos.x, pos.y + margin);
          let line_size = Vec2::new(bounds.width, *thickness);
          ctx.primitives.draw_rect(
            line_pos,
            line_size,
            Vec4::new(color.0, color.1, color.2, color.3),
            [0.0; 4],
            0.0
          );
        }
        Orientation::Vertical => {
          let line_pos = Vec2::new(pos.x + margin, pos.y);
          let line_size = Vec2::new(*thickness, bounds.height);
          ctx.primitives.draw_rect(
            line_pos,
            line_size,
            Vec4::new(color.0, color.1, color.2, color.3),
            [0.0; 4],
            0.0
          );
        }
      };
    }

    Widget::Scrollbar {
      bounds,
      content_size,
      viewport_size,
      scroll_offset,
      orientation,
      style,
      ..
    } => {
      use crate::widget::Orientation;
      let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
      
      // Draw track
      ctx.primitives.draw_rect(
        pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5),
        Vec2::new(bounds.width * 0.5, bounds.height * 0.5),
        Vec4::new(style.track_color.0, style.track_color.1, style.track_color.2, style.track_color.3),
        [style.corner_radius; 4],
        0.0
      );
      
      // Calculate thumb size and position
      let (thumb_pos, thumb_size) = match orientation {
        Orientation::Vertical => {
          let thumb_height = (viewport_size / content_size) * bounds.height;
          let scroll_ratio = scroll_offset / (content_size - viewport_size).max(1.0);
          let thumb_y = scroll_ratio * (bounds.height - thumb_height);
          (
            Vec2::new(pos.x, pos.y + thumb_y),
            Vec2::new(bounds.width, thumb_height)
          )
        }
        Orientation::Horizontal => {
          let thumb_width = (viewport_size / content_size) * bounds.width;
          let scroll_ratio = scroll_offset / (content_size - viewport_size).max(1.0);
          let thumb_x = scroll_ratio * (bounds.width - thumb_width);
          (
            Vec2::new(pos.x + thumb_x, pos.y),
            Vec2::new(thumb_width, bounds.height)
          )
        }
      };
      
      // Draw thumb
      ctx.primitives.draw_rect(
        thumb_pos + thumb_size * 0.5,
        thumb_size * 0.5,
        Vec4::new(style.thumb_color.0, style.thumb_color.1, style.thumb_color.2, style.thumb_color.3),
        [style.corner_radius; 4],
        0.0
      );
    }

    Widget::DataGrid {
      bounds,
      columns,
      header_height,
      style,
      ..
    } => {
      let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
      
      // Placeholder rendering - full implementation in next phase
      
      // Background
      ctx.primitives.draw_rect(
        pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5),
        Vec2::new(bounds.width * 0.5, bounds.height * 0.5),
        Vec4::from(style.row_background),
        [0.0; 4],
        0.0,
      );
      
      // Header background  
      ctx.primitives.draw_rect(
        pos + Vec2::new(bounds.width * 0.5, header_height * 0.5),
        Vec2::new(bounds.width * 0.5, header_height * 0.5),
        Vec4::from(style.header_background),
        [0.0; 4],
        0.0,
      );
      
      // Header text
      let mut x_offset = style.cell_padding;
      for column in columns.iter() {
        ctx.text.draw(
          ctx.device,
          ctx.queue,
          &column.header,
          pos + Vec2::new(x_offset, header_height * 0.5),
          14.0,
          Vec4::from(style.header_text_color),
          HorizontalAlign::Left,
          None,
        );
        x_offset += 120.0;
      }
    }

    Widget::Checkbox { checked, style, bounds, size, .. } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
        let hs = *size * 0.5;
        
        ctx.primitives.draw_rect(center, Vec2::new(hs, hs), Vec4::new(style.background.0, style.background.1, style.background.2, style.background.3), [4.0; 4], 0.0);
        
        if *checked {
            let cs = *size * 0.3;
            ctx.primitives.draw_rect(center, Vec2::new(cs, cs), Vec4::new(style.checkmark_color.0, style.checkmark_color.1, style.checkmark_color.2, style.checkmark_color.3), [1.0; 4], 0.0);
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

    Widget::Slider { value, min, max, style, bounds, .. } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let cy = pos.y + bounds.height * 0.5;
        let pct = ((*value - *min) / (*max - *min)).clamp(0.0, 1.0);
        
        ctx.primitives.draw_rect(
            Vec2::new(pos.x + bounds.width * 0.5, cy), 
            Vec2::new(bounds.width * 0.5, style.track_height * 0.5), 
            Vec4::new(style.track_color.0, style.track_color.1, style.track_color.2, style.track_color.3), 
            [style.track_height * 0.5; 4], 
            0.0
        );
        let aw = bounds.width * pct;
        if aw > 0.0 {
            ctx.primitives.draw_rect(
                Vec2::new(pos.x + aw * 0.5, cy), 
                Vec2::new(aw * 0.5, style.track_height * 0.5), 
                Vec4::new(style.active_track_color.0, style.active_track_color.1, style.active_track_color.2, style.active_track_color.3), 
                [style.track_height * 0.5; 4], 
                0.0
            );
        }
        ctx.primitives.draw_circle(Vec2::new(pos.x + aw, cy), style.thumb_radius, Vec4::ONE, 0.0);
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
  let size = renderer.size();
  let surface_width = size.x as u32;
  let surface_height = size.y as u32;
  
  let (primitives, text, images, textures) = renderer.split_mut();
  
  let mut ctx = RenderContext::new(primitives, text, images, textures, device, queue, interaction, surface_width, surface_height);
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
    Widget::ToggleSwitch { bounds, id, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: id })
        } else {
             None
        }
    }
    Widget::RadioButton { bounds, value, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: value }) // Use value as action ID
        } else {
             None
        }
    }
    Widget::Dropdown { bounds, id, .. } => {
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
