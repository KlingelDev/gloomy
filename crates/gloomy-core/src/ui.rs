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
  pub data_provider: Option<&'a dyn crate::data_source::DataProvider>,
  pub widget_tracker: Option<&'a mut crate::widget_state::WidgetStateTracker>,
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
    data_provider: Option<&'a dyn crate::data_source::DataProvider>,
    widget_tracker: Option<&'a mut crate::widget_state::WidgetStateTracker>,
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
      data_provider,
      widget_tracker,
    }
  }

    pub fn push_scissor(&mut self, rect: Option<(u32, u32, u32, u32)>) {
          self.scissor_stack.push(self.current_scissor);
          
          if let Some(r) = rect {
              if let Some(current) = self.current_scissor {
                  let x = r.0.max(current.0);
                  let y = r.1.max(current.1);
                  let w = (r.0 + r.2).min(current.0 + current.2) - x;
                  let h = (r.1 + r.3).min(current.1 + current.3) - y;
                  if w > 0 && h > 0 {
                      self.current_scissor = Some((x, y, w, h));
                  } else {
                      self.current_scissor = Some((0, 0, 0, 0));
                  }
              } else {
                  self.current_scissor = Some(r);
              }
          }
          
          self.primitives.set_scissor(self.current_scissor);
          self.text.set_scissor(self.current_scissor);
          self.images.set_scissor(self.current_scissor);
    }
    
    pub fn pop_scissor(&mut self) {
        self.current_scissor = self.scissor_stack.pop().flatten();
        self.primitives.set_scissor(self.current_scissor);
        self.text.set_scissor(self.current_scissor);
        self.images.set_scissor(self.current_scissor);
    }

    // --- RENDER CACHING API ---

    pub fn begin_capture(&self) -> CaptureState {
        CaptureState {
            prim_counts: self.primitives.get_counts(),
            text_count: self.text.get_count(),
            img_counts: self.images.get_counts(),
            base_offset: self.offset,
        }
    }

    pub fn end_capture(&self, start: &CaptureState) -> crate::widget::RenderCache {
        let (p_inst, p_batch) = start.prim_counts;
        let t_count = start.text_count;
        let (i_inst, i_batch) = start.img_counts;

        let primitives = self.primitives.capture(p_inst, p_batch);
        let text = self.text.capture(t_count);
        let images = self.images.capture(i_inst, i_batch);

        // Optimization: return None if empty? 
        // For now, store fully.
        
        crate::widget::RenderCache {
            primitives: Some(primitives),
            text: Some(text),
            images: Some(images),
            base_offset: start.base_offset,
        }
    }

    pub fn replay_cache(&mut self, cache: &crate::widget::RenderCache) {
        let delta = self.offset - cache.base_offset;
        
        if let Some(p) = &cache.primitives {
            self.primitives.replay(p, delta);
        }
        if let Some(t) = &cache.text {
            self.text.replay(t, delta);
        }
        if let Some(i) = &cache.images {
            self.images.replay(i, delta);
        }
    }
}

pub struct CaptureState {
    pub prim_counts: (usize, usize),
    pub text_count: usize,
    pub img_counts: (usize, usize),
    pub base_offset: Vec2,
}

/// Renders text with optional rich text markup support.
/// Automatically detects and parses HTML-like markup.
/// Uses cached parsing to avoid re-parsing unchanged text.
fn render_text_field(
    ctx: &mut RenderContext,
    text: &str,
    pos: Vec2,
    default_size: f32,
    default_color: (f32, f32, f32, f32),
    default_font: Option<&str>,
    align: TextAlign,
    max_width: Option<f32>,
) {
    use crate::rich_text::{RichText, TextStyle};
    use std::collections::HashMap;
    use std::cell::RefCell;
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    
    // Quick check for markup
    if RichText::has_markup(text) {
        // Compute hash for cache lookup (content-addressable)
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        // Include style in hash for correct invalidation
        (default_size as u32).hash(&mut hasher);
        // Hash color too to ensure correctness if same text used with diff colors
        ((default_color.0 * 255.0) as u32).hash(&mut hasher);
        let cache_key = hasher.finish();
        
        // Try to get from tracker first, then fall back to parse
        let rich_text = if let Some(tracker) = &mut ctx.widget_tracker {
            if let Some(cached) = tracker.get_by_hash(cache_key) {
                cached
            } else {
                let base_style = TextStyle {
                    color: default_color,
                    font_size: Some(default_size),
                    font_family: default_font.map(|s| s.to_string()),
                    bold: false,
                    italic: false,
                    underline: false,
                };
                let parsed = RichText::parse(text, base_style);
                tracker.store_by_hash(cache_key, parsed.clone());
                parsed
            }
        } else {
             // Fallback to local parsing (or could keep thread_local as Level 2 cache)
             // For now just parse to keep it simple, as tracker is expected
                let base_style = TextStyle {
                    color: default_color,
                    font_size: Some(default_size),
                    font_family: default_font.map(|s| s.to_string()),
                    bold: false,
                    italic: false,
                    underline: false,
                };
                RichText::parse(text, base_style)
        };
        
        render_rich_text(ctx, &rich_text, pos, default_size, align, max_width);
    } else {
        // Render as plain text (fast path)
        ctx.text.draw(
            ctx.device,
            ctx.queue,
            text,
            pos,
            default_size,
            Vec4::from(default_color),
            map_text_align(align),
            default_font
        );
    }
}

/// Renders parsed rich text with per-span styling.
/// Uses per-character rendering with glyph caching for optimal performance.
fn render_rich_text(
    ctx: &mut RenderContext,
    rich_text: &crate::rich_text::RichText,
    base_pos: Vec2,
    default_size: f32,
    align: TextAlign,
    _max_width: Option<f32>,
) {
    // Calculate total width for alignment
    let (total_width, _) = rich_text.measure(default_size);
    
    // Calculate starting X offset based on alignment
    let mut x_offset = match align {
        TextAlign::Left => 0.0,
        TextAlign::Center => -total_width / 2.0,
        TextAlign::Right => -total_width,
    };
    
    // Render each span with per-character positioning (uses glyph cache)
    for span in &rich_text.spans {
        let size = span.style.font_size.unwrap_or(default_size);
        let base_font = span.style.font_family.as_deref();
        
        // Select appropriate font variant based on bold/italic flags
        let font_name = ctx.text.get_font_for_style(
            base_font,
            span.style.bold,
            span.style.italic
        );
        
        // Render each character with cached measurements
        for ch in span.text.chars() {
            let ch_str = ch.to_string();
            let char_pos = Vec2::new(base_pos.x + x_offset, base_pos.y);
            
            // Draw the character
            ctx.text.draw(
                ctx.device,
                ctx.queue,
                &ch_str,
                char_pos,
                size,
                Vec4::from(span.style.color),
                HorizontalAlign::Left,
                font_name
            );
            
            // Use cached character measurement
            let char_width = ctx.text.measure_char_cached(ch, size, font_name);
            
            // Draw underline if needed
            if span.style.underline {
                let underline_y = char_pos.y + size * 0.85;
                ctx.primitives.draw_line(
                    Vec2::new(char_pos.x, underline_y),
                    Vec2::new(char_pos.x + char_width, underline_y),
                    1.0,
                    Vec4::from(span.style.color),
                );
            }
            
            x_offset += char_width;
        }
    }
}

/// Maps TextAlign to glyph_brush HorizontalAlign.
fn map_text_align(align: TextAlign) -> HorizontalAlign {
    match align {
        TextAlign::Left => HorizontalAlign::Left,
        TextAlign::Center => HorizontalAlign::Center,
        TextAlign::Right => HorizontalAlign::Right,
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
                layout_cache: None,
                render_cache: std::cell::RefCell::new(None),
            };
            let overlay_pos = pos + Vec2::new(0.0, h);
            ctx.overlay_queue.push((dropdown_list, overlay_pos));
        }
    }
    Widget::Container { id, children, bounds, padding: _, background, corner_radius, scrollable, shadow, gradient, border, layout_cache, render_cache, .. } => {
      // --- RENDER CACHE CHECK ---
      if let Some(l_cache) = layout_cache {
          if l_cache.valid {
              if let Some(r_cache) = render_cache.borrow().as_ref() {
                   // Check if bounds match? Layout cache validity implies bounds are consistent with constraints.
                   // So we can safely replay.
                   ctx.replay_cache(r_cache);
                   return;
              }
          }
      }

      let capture_state = ctx.begin_capture();

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

      // --- RENDER CACHE UPDATE ---
      let new_cache = ctx.end_capture(&capture_state);
      *render_cache.borrow_mut() = Some(Box::new(new_cache));
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
      
      let mut text_pos = ctx.offset + Vec2::new(*x, *y);
      if *text_align == TextAlign::Center {
          text_pos.x += width * 0.5;
      } else if *text_align == TextAlign::Right {
          text_pos.x += width;
      }

      // Use rich text rendering (automatically handles markup)
      render_text_field(
        ctx,
        text,
        text_pos,
        *size,
        *color,
        font.as_deref(),
        *text_align,
        Some(*width),
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
      // Calculate center position for the button
      let text_pos = pos + Vec2::new(bounds.width * 0.5, (bounds.height - text_size) * 0.5);
      let text_col = (1.0, 1.0, 1.0, 1.0);

      // Use rich text rendering with center alignment
      // The render_text_field will handle centering the text around text_pos
      render_text_field(
        ctx,
        text,
        text_pos,
        text_size,
        text_col,
        font.as_deref(),
        TextAlign::Center,
        Some(bounds.width),
      );
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
      data_source_id,
      header_height,
      row_height,
      striped,
      show_vertical_lines,
      show_horizontal_lines,
      style,
      id,
      selected_rows,
      sort_column,
      sort_direction,
      ..
    } => {
      let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
      
      // Calculate Scroll Offset
      let scroll_offset = id.as_ref()
           .and_then(|i| ctx.interaction.as_ref().map(|s| s.scroll_offsets.get(i)))
           .flatten()
           .map(|v| v.y)
           .unwrap_or(0.0);

      // 1. Resolve Data Source
      let source = data_source_id.as_ref()
           .and_then(|id| ctx.data_provider.and_then(|dp| dp.get_source(id)));
      
      // 2. Calculate column widths
      let available_width = bounds.width;
      let mut col_widths = Vec::with_capacity(columns.len());
      let mut total_fixed = 0.0;
      let mut total_flex = 0.0;
      
      for col in columns {
          match col.width {
              crate::datagrid::ColumnWidth::Fixed(w) => {
                  total_fixed += w;
              }
              crate::datagrid::ColumnWidth::Flex(f) => {
                  total_flex += f;
              }
              _ => total_fixed += col.min_width,
          }
      }
      
      let remaining = (available_width - total_fixed).max(0.0);
      
      for col in columns {
          let w = match col.width {
              crate::datagrid::ColumnWidth::Fixed(w) => w,
              crate::datagrid::ColumnWidth::Flex(f) => {
                  if total_flex > 0.0 {
                      (remaining * f / total_flex).max(col.min_width)
                  } else {
                      col.min_width
                  }
              }
              crate::datagrid::ColumnWidth::Auto => col.min_width, 
          };
          col_widths.push(w);
      }
      
      // Background
      ctx.primitives.draw_rect(
        pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5),
        Vec2::new(bounds.width * 0.5, bounds.height * 0.5),
        Vec4::from(style.row_background),
        [0.0; 4],
        0.0,
      );

      // Render Rows
      if let Some(ds) = source {
          let row_count = ds.row_count();
          let visible_height = bounds.height - header_height;
          
          let start_row = (scroll_offset / row_height).floor().max(0.0) as usize;
          let visible_rows_count = (visible_height / row_height).ceil() as usize + 1;
          let end_row = (start_row + visible_rows_count).min(row_count);
          
          let content_y = pos.y + header_height;
          
          let mut r = start_row;
          while r < end_row {
               let row_y = pos.y + header_height + ((r as f32) * row_height) - scroll_offset;
               let center_y = row_y + row_height * 0.5;
               
               // Clip check
               if row_y + row_height < content_y || row_y > content_y + visible_height {
                   r += 1;
                   continue;
               }
               
               // Selection & Striping
               if selected_rows.contains(&r) {
                   ctx.primitives.draw_rect(
                       Vec2::new(pos.x + bounds.width * 0.5, center_y),
                       Vec2::new(bounds.width * 0.5, row_height * 0.5),
                       Vec4::from(style.selected_background),
                       [0.0; 4],
                       0.0
                   );
               } else if *striped && r % 2 == 1 {
                   ctx.primitives.draw_rect(
                       Vec2::new(pos.x + bounds.width * 0.5, center_y),
                       Vec2::new(bounds.width * 0.5, row_height * 0.5),
                       Vec4::from(style.alt_row_background),
                       [0.0; 4],
                       0.0
                   );
               }
               
               // Horizontal Lines
               if *show_horizontal_lines {
                    let line_y = row_y + row_height;
                    ctx.primitives.draw_rect(
                         Vec2::new(pos.x + bounds.width * 0.5, line_y - style.grid_line_width * 0.5),
                         Vec2::new(bounds.width * 0.5, style.grid_line_width * 0.5),
                         Vec4::from(style.grid_line_color),
                         [0.0; 4],
                         0.0
                    );
               }

               // Cells
               let mut x = pos.x;
               for (c, col) in columns.iter().enumerate() {
                   let w = col_widths[c];
                   // Column Virtualization / Culling
                   let col_end = x + w;
                   
                   // Check if column is within visible bounds
                   // Note: x is absolute screen position. 
                   // visible range is [pos.x, pos.x + bounds.width]
                   if col_end < pos.x {
                       x += w;
                       continue;
                   }
                   if x > pos.x + bounds.width {
                       break; // All subsequent columns are also to the right
                   }
                   
                   let text = ds.cell_text(r, c);
                   
                   let (text_align_enum, text_x) = match col.align {
                       crate::widget::TextAlign::Left => (crate::widget::TextAlign::Left, x + style.cell_padding),
                       crate::widget::TextAlign::Center => (crate::widget::TextAlign::Center, x + w * 0.5),
                       crate::widget::TextAlign::Right => (crate::widget::TextAlign::Right, x + w - style.cell_padding),
                   };
                   
                   // Use rich text rendering for cells
                   render_text_field(
                       ctx,
                       &text,
                       Vec2::new(text_x, center_y),
                       13.0,
                       style.row_text_color,
                       None,
                       text_align_enum,
                       Some(w),
                   );
                   x += w;
               }
               r += 1;
           }
       }

       // Scrollbar
       if let Some(ds) = source {
           let row_count = ds.row_count();
           let total_height = row_count as f32 * row_height;
           let visible_height = (bounds.height - header_height).max(0.0);
           
           if total_height > visible_height {
                let track_w = 12.0;
                let track_h = visible_height;
                let track_x = pos.x + bounds.width - track_w - 2.0;
                let track_y = pos.y + header_height;
                
                // Track
                let track_center = Vec2::new(track_x + track_w * 0.5, track_y + track_h * 0.5);
                ctx.primitives.draw_rect(
                    track_center,
                    Vec2::new(track_w * 0.5, track_h * 0.5),
                    Vec4::new(0.0, 0.0, 0.0, 0.2), 
                    [4.0; 4],
                    0.0
                );
                
                // Thumb
                let thumb_h = (visible_height / total_height * track_h).max(20.0);
                // Clamp ratio 0..1 to be safe
                let max_scroll = total_height - visible_height;
                let scroll_ratio = (scroll_offset / max_scroll).clamp(0.0, 1.0);
                
                let available_travel = track_h - thumb_h;
                let thumb_offset = scroll_ratio * available_travel;
                let thumb_y = track_y + thumb_offset;
                
                let thumb_center = Vec2::new(track_x + track_w * 0.5, thumb_y + thumb_h * 0.5);
                ctx.primitives.draw_rect(
                    thumb_center,
                    Vec2::new(track_w * 0.5 - 2.0, thumb_h * 0.5), 
                    Vec4::new(0.5, 0.5, 0.5, 0.8),
                    [3.0; 4],
                    0.0
                );
           }
       }

      // Header
       ctx.primitives.draw_rect(
         pos + Vec2::new(bounds.width * 0.5, header_height * 0.5),
         Vec2::new(bounds.width * 0.5, header_height * 0.5),
         Vec4::from(style.header_background),
         [0.0; 4],
         0.0,
       );
       
       let mut x = pos.x;
       for (i, col) in columns.iter().enumerate() {
           let w = col_widths[i];
           
           ctx.text.draw(
             ctx.device,
             ctx.queue,
             &col.header,
             Vec2::new(x + style.cell_padding, pos.y + header_height * 0.5),
             14.0,
             Vec4::from(style.header_text_color),
             HorizontalAlign::Left,
             None,
           );

            // Check for Hovered Resize
            let mut is_resize_hover = false;
            if let Some(interaction) = ctx.interaction {
                if let Some(ref action) = interaction.hovered_action {
                    if action == &format!("{}:header_resize:{}", id.as_ref().unwrap_or(&"".to_string()), i) {
                        is_resize_hover = true;
                    }
                }
            }
            
            if is_resize_hover {
                 // Draw Highlight Line
                 let line_x = x + w;
                 ctx.primitives.draw_rect(
                     Vec2::new(line_x, pos.y + header_height * 0.5),
                     Vec2::new(2.0, header_height * 0.8), // 4px wide (2.0 radius), slightly shorter than header
                     Vec4::new(1.0, 1.0, 1.0, 0.5),
                     [1.0; 4],
                     0.0
                 );
            }

            // Sort Indicator
            if let Some(sc) = sort_column {
                if *sc == i {
                     if let Some(dir) = sort_direction {
                         let arrow = match dir {
                             crate::data_source::SortDirection::Ascending => "▲",
                             crate::data_source::SortDirection::Descending => "▼",
                         };
                         
                         // Position at right edge of column
                         // But we don't have exact text width of header label here easily.
                         // Just put it at right - padding.
                         let arrow_x = x + w - style.cell_padding;
                         
                         ctx.text.draw(
                              ctx.device,
                              ctx.queue,
                              arrow,
                              Vec2::new(arrow_x, pos.y + header_height * 0.5),
                              10.0,
                              Vec4::from(style.header_text_color),
                              HorizontalAlign::Right,
                              None
                         );
                     }
                }
            }
           
           if *show_vertical_lines && i > 0 {
                // Draw line at x
                ctx.primitives.draw_rect(
                    Vec2::new(x, pos.y + bounds.height * 0.5),
                    Vec2::new(style.grid_line_width * 0.5, bounds.height * 0.5),
                    Vec4::from(style.grid_line_color),
                    [0.0; 4],
                    0.0
                );
           }
           
           x += w;
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
    Widget::Tree { id, bounds, root_nodes, selected_id, expanded_ids, style, .. } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        
        // 1. Flatten visible nodes
        let mut visible_rows = Vec::new();
        flatten_tree(root_nodes, expanded_ids, 0, &mut visible_rows);
        
        let mut y = pos.y;
        
        for (node, depth) in visible_rows {
             // Row Background
             let row_rect = crate::Rect { 
                 x: pos.x, 
                 y, 
                 width: bounds.width, 
                 height: style.row_height 
             };
             
             // Selection
             if Some(&node.id) == selected_id.as_ref() {
                 ctx.primitives.draw_rect(
                     Vec2::new(row_rect.x + row_rect.width * 0.5, row_rect.y + row_rect.height * 0.5),
                     Vec2::new(row_rect.width * 0.5, row_rect.height * 0.5),
                     Vec4::from(style.selected_background),
                     [0.0; 4],
                     0.0
                 );
             }
             
             // Hover (Basic, from InteractionState)
             // Need full ID for hit test correlation: format!("{}:node:{}", tree_id, node.id)
             
             let indent_x = pos.x + (depth as f32 * style.indent_size);
             
             // Expander
             if !node.leaf && !node.children.is_empty() {
                 let expanded = expanded_ids.contains(&node.id);
                 let symbol = if expanded { "▼" } else { "►" };
                 
                 ctx.text.draw(
                    ctx.device,
                    ctx.queue,
                    symbol,
                    Vec2::new(indent_x + style.indent_size * 0.5, y + style.row_height * 0.5),
                    style.font_size * 0.8,
                    Vec4::from(style.icon_color),
                    HorizontalAlign::Center,
                    None
                 );
             }
             
             // Label
             let label_x = indent_x + style.indent_size + 4.0;
             render_text_field(
                ctx,
                &node.label,
                Vec2::new(label_x, y + style.row_height * 0.5),
                style.font_size,
                style.text_color,
                None,
                TextAlign::Left,
                Some(bounds.width - label_x),
             );
             
             y += style.row_height;
        }
    }
  }
}

/// Convenience function to render a widget with the renderer.
/// Convenience function to render a widget with the renderer.
pub fn render_ui(
  widget: &Widget,
  renderer: &mut crate::renderer::GloomyRenderer,
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  interaction: Option<&InteractionState>,
  data_provider: Option<&dyn crate::data_source::DataProvider>,
) {
    render_ui_with_state(widget, renderer, device, queue, interaction, data_provider, None);
}

/// Renders a widget with optional state tracking for performance.
pub fn render_ui_with_state(
  widget: &Widget,
  renderer: &mut crate::renderer::GloomyRenderer,
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  interaction: Option<&InteractionState>,
  data_provider: Option<&dyn crate::data_source::DataProvider>,
  widget_tracker: Option<&mut crate::widget_state::WidgetStateTracker>,
) {
  let size = renderer.size();
  let surface_width = size.x as u32;
  let surface_height = size.y as u32;
  
  let (primitives, text, images, textures) = renderer.split_mut();
  
  let mut ctx = RenderContext::new(primitives, text, images, textures, device, queue, interaction, surface_width, surface_height, data_provider, widget_tracker);
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
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: action.clone() })
           } else {
             None
           }
    }
    Widget::TextInput { bounds, id, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: id.clone() })
           } else {
             None
           }
    }
    Widget::Checkbox { bounds, id, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: id.clone() })
        } else {
             None
        }
    }
    Widget::Slider { bounds, id, .. } => {
         if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: id.clone() })
        } else {
             None
        }
    }
    Widget::ToggleSwitch { bounds, id, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: id.clone() })
        } else {
             None
        }
    }
    Widget::RadioButton { bounds, value, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: value.clone() })
        } else {
             None
        }
    }
     Widget::Tree { id, bounds, root_nodes, expanded_ids, style, .. } => {
          if point.x >= bounds.x && point.x <= bounds.x + bounds.width
             && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
               
               let local_y = point.y - bounds.y;
               if local_y < 0.0 { return None; }
               
               let row_index = (local_y / style.row_height) as usize;
               
               // Re-flatten to find the node at this index
               let mut visible_rows = Vec::new();
               flatten_tree(root_nodes, expanded_ids, 0, &mut visible_rows);
               
               if let Some((node, depth)) = visible_rows.get(row_index) {
                   let wid = id.as_deref().unwrap_or("tree");
                   
                   // Check for X position (Expanded toggle vs Select)
                   let indent_x = bounds.x + (*depth as f32 * style.indent_size);
                   
                   if point.x >= indent_x && point.x < indent_x + style.indent_size {
                       // Toggle area
                       return Some(HitTestResult { widget, action: format!("{}:toggle:{}", wid, node.id) });
                   } else {
                       // Select area (row)
                       return Some(HitTestResult { widget, action: format!("{}:select:{}", wid, node.id) });
                   }
               }
               None
          } else {
              None
          }
    }
    Widget::DataGrid { bounds, id, header_height, row_height, columns, .. } => {
         if point.x >= bounds.x && point.x <= bounds.x + bounds.width
            && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
              if let Some(wid) = id {
                  let local_y = point.y - bounds.y;
                  let local_x = point.x - bounds.x;
                  
                  // Header check
                  if local_y < *header_height {
                       // Determine column
                       let content_width = (bounds.width).max(0.0);
                       let mut col_widths = Vec::new();
                       let mut total_fixed = 0.0;
                       let mut total_flex = 0.0;
                       
                       for col in columns {
                           match col.width {
                               crate::datagrid::ColumnWidth::Fixed(w) => total_fixed += w,
                               crate::datagrid::ColumnWidth::Flex(f) => total_flex += f,
                               _ => {}
                           }
                       }
                       
                       let available = (content_width - total_fixed).max(0.0);
                       
                       for col in columns {
                           let w = match col.width {
                               crate::datagrid::ColumnWidth::Fixed(w) => w,
                               crate::datagrid::ColumnWidth::Flex(f) => {
                                   if total_flex > 0.0 {
                                       (f / total_flex) * available
                                   } else {
                                       0.0
                                   }
                               },
                               _ => 0.0,
                           };
                           col_widths.push(w);
                       }
                       
                       let mut cx = 0.0;
                       for (i, w) in col_widths.iter().enumerate() {
                           let right_edge = cx + w;
                           // Check for resize (Right edge) - 8px tolerance
                           if (local_x - right_edge).abs() <= 8.0 && i < columns.len() {
                               if columns[i].resizable {
                                   return Some(HitTestResult { widget, action: format!("{}:header_resize:{}", wid, i) });
                               }
                           }
                           
                           if local_x >= cx && local_x < cx + w {
                               return Some(HitTestResult { widget, action: format!("{}:header:{}", wid, i) });
                           }
                           cx += w;
                       }
                       
                       return Some(HitTestResult { widget, action: wid.clone() });
                  }
                  
                  let scroll_y = if let Some(offsets) = scroll_offsets {
                       offsets.get(wid).map(|v| v.y).unwrap_or(0.0)
                  } else { 0.0 };
                  
                  let content_y = local_y - header_height + scroll_y;
                  if content_y >= 0.0 {
                      let row = (content_y / row_height).floor() as isize;
                      if row >= 0 {
                           return Some(HitTestResult { widget, action: format!("{}:row:{}", wid, row) });
                      }
                  }
                  
                  Some(HitTestResult { widget, action: wid.clone() })
              } else {
                  None
              }
         } else {
              None
         }
    }
    Widget::Dropdown { bounds, id, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: id.clone() })
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
        
        Widget::Checkbox { id, checked, .. } => {
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

fn flatten_tree<'a>(
    nodes: &'a [crate::tree::TreeNode],
    expanded: &std::collections::HashSet<String>,
    depth: usize,
    out: &mut Vec<(&'a crate::tree::TreeNode, usize)>
) {
    for node in nodes {
        out.push((node, depth));
        if expanded.contains(&node.id) {
            flatten_tree(&node.children, expanded, depth + 1, out);
        }
    }
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
