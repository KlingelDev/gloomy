//! UI loading and rendering from RON definitions.

use crate::interaction::HitTestResult;
use crate::interaction::InteractionState;
use crate::primitives::PrimitiveRenderer;
use crate::text::TextRenderer;
use crate::widget::{Widget, TextAlign, WidgetBounds};
use crate::layout::Layout;
use wgpu_text::glyph_brush::HorizontalAlign;
use glam::{Vec2, Vec4};
use crate::style::{BoxStyle, ButtonStyle, TextInputStyle, Border};
use std::fs;
use std::path::Path;
use winit::keyboard::{Key, NamedKey};
use winit::event::ElementState;
use chrono::{NaiveDate, Datelike};

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
  pub scale_factor: f32, // Added scale factor
  pub overlay_queue: Vec<(Widget, Vec2)>, // Widget + Absolute Position
  pub data_provider: Option<&'a dyn crate::data_source::DataProvider>,
  pub widget_tracker: Option<&'a mut crate::widget_state::WidgetStateTracker>,
  pub deferred_draws: Option<&'a mut Vec<Box<dyn FnOnce(&mut crate::renderer::GloomyRenderer, &wgpu::Device, &wgpu::Queue)>>>,
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
    scale_factor: f32, // Added arg
    data_provider: Option<&'a dyn crate::data_source::DataProvider>,
    widget_tracker: Option<&'a mut crate::widget_state::WidgetStateTracker>,
    deferred_draws: Option<&'a mut Vec<Box<dyn FnOnce(&mut crate::renderer::GloomyRenderer, &wgpu::Device, &wgpu::Queue)>>>,
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
      scale_factor, // Init field
      overlay_queue: Vec::new(),
      data_provider,
      widget_tracker,
      deferred_draws,
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
                    style: ButtonStyle {
                        idle: BoxStyle::fill(bg_col),
                        hover: BoxStyle::fill((0.35, 0.35, 0.4, 1.0)),
                        active: BoxStyle::fill((0.4, 0.4, 0.5, 1.0)),
                        text_color: (1.0, 1.0, 1.0, 1.0),
                        ..Default::default()
                    },
                    width: None, height: None, disabled: false, layout: Layout::default(),
                    flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                    font: None,
                };
                list_children.push(btn);
            }
            let list_height = (list_children.len() as f32) * 30.0;
            let dropdown_list = Widget::Container {
                id: Some(format!("{}_list", id)), scrollable: false,
                bounds: WidgetBounds { x: 0.0, y: 0.0, width: w, height: list_height },
                width: Some(w), height: Some(list_height),
                style: BoxStyle {
                    background: Some((0.2, 0.2, 0.25, 1.0)),
                    border: Some(crate::style::Border { width: 1.0, color: (0.1, 0.1, 0.1, 1.0), ..Default::default() }),
                    shadow: Some(crate::style::Shadow { offset: (0.0, 4.0), blur: 8.0, color: (0.0, 0.0, 0.0, 0.5) }),
                    ..Default::default()
                },
                padding: 0.0, layout: crate::layout::Layout { direction: crate::layout::Direction::Column, ..Default::default() },
                flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                children: list_children,
                layout_cache: None,
                render_cache: std::cell::RefCell::new(None),
            };
            let overlay_pos = pos + Vec2::new(0.0, h);
            ctx.overlay_queue.push((dropdown_list, overlay_pos));
        }
    }
    Widget::Container { id, children, bounds, padding: _, style, scrollable, layout_cache, render_cache, .. } => {
      // --- RENDER CACHE DISABLED ---
      // The render cache is causing issues with initial sizing and scroll updates.
      // TODO: Implement proper cache invalidation based on scroll state and layout changes.
      // For now, always re-render containers to ensure correctness.

      let capture_state = ctx.begin_capture();

      let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
      let size = Vec2::new(bounds.width, bounds.height);
      
      draw_box(ctx, pos, size, style);

      let mut child_offset = pos;
      let mut pushed_scissor = false;

      if *scrollable {
          let scroll = if let Some(wid) = id {
              ctx.interaction.and_then(|i| i.scroll_offsets.get(wid)).copied().unwrap_or(Vec2::ZERO)
          } else {
              Vec2::ZERO
          };
          
          child_offset = pos - scroll;

          let s = ctx.scale_factor;
          let x = (pos.x * s).max(0.0).floor() as u32;
          let y = (pos.y * s).max(0.0).floor() as u32;
          let w = (bounds.width * s).max(0.0).ceil() as u32;
          let h = (bounds.height * s).max(0.0).ceil() as u32;
          
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
      let s = ctx.scale_factor;
      let scissor_x = ((ctx.offset.x + x) * s).max(0.0).floor() as u32;
      let scissor_y = ((ctx.offset.y + y) * s).max(0.0).floor() as u32;
      let scissor_w = (*width * s).max(0.0).ceil() as u32;
      let scissor_h = (*height * s).max(0.0).ceil() as u32;

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

    Widget::ListView {
        items,
        selected_index,
        style,
        bounds,
        id,
        ..
    } => {
         let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);

         // 1. Get Scroll Offset
         let scroll_offset = ctx.interaction.as_ref()
              .and_then(|i| i.scroll_offsets.get(id))
              .map(|v| v.y)
              .unwrap_or(0.0);

         // 2. Set Scissor
         let s = ctx.scale_factor;
         let list_scissor = (
              (pos.x * s).floor() as u32,
              (pos.y * s).floor() as u32,
              (bounds.width * s).ceil() as u32,
              (bounds.height * s).ceil() as u32
         );
         ctx.push_scissor(Some(list_scissor));
         
         // 3. Calculate visible range
         let item_h = style.item_height;
         let start_index = (scroll_offset / item_h).floor() as usize;
         let visible_count = (bounds.height / item_h).ceil() as usize;
         // Add buffer
         let buffer = 2;
         let start_index = start_index.saturating_sub(buffer);
         let end_index = (start_index + visible_count + buffer * 2).min(items.len());

         let mouse_pos = ctx.interaction.map(|s| s.mouse_pos).unwrap_or(Vec2::ZERO);
         let local_mouse_y = mouse_pos.y - pos.y + scroll_offset;
         let hover_index = if mouse_pos.x >= pos.x && mouse_pos.x <= pos.x + bounds.width 
             && mouse_pos.y >= pos.y && mouse_pos.y <= pos.y + bounds.height 
         {
             Some((local_mouse_y / item_h) as usize)
         } else {
             None
         };
         
         for i in start_index..end_index {
             let item = &items[i];
             let item_y = pos.y + i as f32 * item_h - scroll_offset;
             let item_rect_pos = Vec2::new(pos.x + bounds.width * 0.5, item_y + item_h * 0.5);
             let item_size = Vec2::new(bounds.width, item_h);
             
             let is_selected = selected_index.map(|si| si == i).unwrap_or(false);
             let is_hovered = hover_index.map(|hi| hi == i).unwrap_or(false);
             
             let (bg_style, text_color) = if is_selected {
                 (&style.selected, style.text_color_selected)
             } else if is_hovered {
                 (&style.hover, style.text_color_idle)
             } else {
                 (&style.idle, style.text_color_idle)
             };
             
             // Draw Background
             if let Some(bg_color) = bg_style.background {
                  ctx.primitives.draw_rect(
                      item_rect_pos,
                      item_size * 0.5, 
                      Vec4::from(bg_color),
                      bg_style.corner_radii,
                      bg_style.border.map(|b| b.width).unwrap_or(0.0)
                  );
             }
             
             // Draw Text
             let text_pos = Vec2::new(pos.x + 12.0, item_y + item_h * 0.5 - 8.0); 
             ctx.text.draw(
                 ctx.device, ctx.queue, item, text_pos, 16.0, 
                 Vec4::from(text_color), HorizontalAlign::Left, None
             );
         }
         
         ctx.pop_scissor();

         // 4. Draw Scrollbar
         let content_height = items.len() as f32 * item_h;
         if content_height > bounds.height {
              let track_w = 10.0;
              let track_h = bounds.height;
              let track_x = pos.x + bounds.width - track_w;
              let track_y = pos.y;
              
              // Track
              ctx.primitives.draw_rect(
                  Vec2::new(track_x + track_w * 0.5, track_y + track_h * 0.5),
                  Vec2::new(track_w * 0.5, track_h * 0.5),
                  Vec4::new(0.0, 0.0, 0.0, 0.2), 
                  [4.0; 4], 0.0
              );
              
              // Thumb
              let thumb_h = (bounds.height / content_height * track_h).max(20.0);
              let max_scroll = content_height - bounds.height;
              let scroll_ratio = if max_scroll > 0.0 { (scroll_offset / max_scroll).clamp(0.0, 1.0) } else { 0.0 };
              let thumb_offset = scroll_ratio * (track_h - thumb_h);
              let thumb_y = track_y + thumb_offset;
              
              ctx.primitives.draw_rect(
                  Vec2::new(track_x + track_w * 0.5, thumb_y + thumb_h * 0.5),
                  Vec2::new(track_w * 0.5 - 2.0, thumb_h * 0.5), 
                  Vec4::new(0.5, 0.5, 0.5, 0.8),
                  [3.0; 4], 0.0
              );
         }
    }

    Widget::Button {
      text,
      action,
      bounds,
      style,
      disabled,
      font,
      ..
    } => {
      let is_disabled = *disabled;
      let is_hovered = ctx.interaction.map(|i| i.is_hovered(action)).unwrap_or(false);
      let is_active = ctx.interaction.map(|i| i.is_active(action)).unwrap_or(false);

      let box_style = if is_disabled {
          &style.disabled
      } else if is_active {
          &style.active
      } else if is_hovered {
          &style.hover
      } else {
          &style.idle
      };
    
      let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
      let size = Vec2::new(bounds.width, bounds.height);
      
      draw_box(ctx, pos, size, box_style);
      
      let text_size = 16.0;
      let text_pos = pos + Vec2::new(bounds.width * 0.5, (bounds.height - text_size) * 0.5);
      // Determine text color based on state if supported, or just base color
      let text_col = style.text_color;

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
        let size = Vec2::new(bounds.width, bounds.height); 

        let is_focused = ctx.interaction.map(|s| s.focused_id.as_deref() == Some(id)).unwrap_or(false);
        
        let box_style = if is_focused {
            &style.focused
        } else {
            &style.idle
        };
        
        draw_box(ctx, pos, size, box_style);
        
        // Draw error border if needed (overlay)
        let has_error = ctx.interaction.as_ref()
           .and_then(|i| i.validation_errors.get(id))
           .map(|e| !e.is_empty())
           .unwrap_or(false);
           
        if has_error {
             let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
             let half_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
             ctx.primitives.draw_rect(center, half_size, Vec4::new(1.0, 0.2, 0.2, 1.0), style.idle.corner_radii, 1.5);
        }

        let text = if value.is_empty() { placeholder } else { value };
        let col_tuple = if value.is_empty() { style.placeholder_color } else { style.text_color };
        let col = Vec4::new(col_tuple.0, col_tuple.1, col_tuple.2, col_tuple.3);
        
        let size_val = if *font_size > 0.0 { *font_size } else { 14.0 };
        
        // Measure and position text
        // (Assuming Left align for now as per original code logic usually hardcoded x=8.0)
        // Original code: text_pos = pos + Vec2::new(8.0, ...)
        
        let text_dims = ctx.text.measure(text, size_val, style.font.as_deref());
        let text_pos = pos + Vec2::new(8.0, (bounds.height - text_dims.y) * 0.5);

        ctx.text.draw(ctx.device, ctx.queue, text, text_pos, size_val, col, HorizontalAlign::Left, style.font.as_deref());
        
        // Draw cursor if focused
        if is_focused {
            let align_x = 8.0;
            let cursor_x = if value.is_empty() {
                align_x
            } else {
                let val_dims = ctx.text.measure(value, size_val, style.font.as_deref());
                align_x + val_dims.x + 2.0
            };
            
            let cursor_pos = pos + Vec2::new(cursor_x + 1.0, bounds.height * 0.5);
            let cursor_col = Vec4::new(style.cursor_color.0, style.cursor_color.1, style.cursor_color.2, style.cursor_color.3);
            ctx.primitives.draw_rect(
                cursor_pos, 
                Vec2::new(1.0, size_val * 0.8 * 0.5), 
                cursor_col, 
                [0.0; 4], 
                0.0
            );
        }
    }

    Widget::Spacer { .. } => {}
    Widget::NumberInput {
        id,
        value,
        min: _,
        max: _,
        step: _,
        precision,
        show_spinner,
        bounds,
        style,
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
        ctx.primitives.draw_rect(center, half_size, Vec4::new(bg_color.0, bg_color.1, bg_color.2, bg_color.3), [style.corner_radius; 4], 0.0);
        
        // Border
        let has_error = ctx.interaction.as_ref()
           .and_then(|i| i.validation_errors.get(id))
           .map(|e| !e.is_empty())
           .unwrap_or(false);

        let border = if is_focused {
            style.border_focused.as_ref().or(style.border.as_ref())
        } else {
            style.border.as_ref()
        };
        
        let error_border_color = (1.0, 0.2, 0.2, 1.0);
        
        if let Some(b) = border {
             let color = if has_error { error_border_color } else { b.color };
             ctx.primitives.draw_rect(center, half_size, Vec4::new(color.0, color.1, color.2, color.3), [style.corner_radius; 4], b.width);
        } else if has_error {
             ctx.primitives.draw_rect(center, half_size, Vec4::new(error_border_color.0, error_border_color.1, error_border_color.2, error_border_color.3), [style.corner_radius; 4], 1.5);
        }

        // Calculate layout
        let spinner_width = if *show_spinner { 20.0 } else { 0.0 };
        let text_area_width = bounds.width - spinner_width - 8.0; // 8px padding

        // Value Formatting
        let text = format!("{:.1$}", value, precision);
        let col = Vec4::new(style.text_color.0, style.text_color.1, style.text_color.2, style.text_color.3);
        let size = 14.0; // Default font size for now
        
        // Text Rendering
        let text_dims = ctx.text.measure(&text, size, style.font.as_deref());
        let text_y = (bounds.height - text_dims.y) * 0.5;
        let text_pos = pos + Vec2::new(8.0, text_y);

        // Clip text if it exceeds area? For now just draw.
        // Helper to clip would be nice, but let's assume it fits or simple scrolling later.
        
        ctx.text.draw(ctx.device, ctx.queue, &text, text_pos, size, col, HorizontalAlign::Left, style.font.as_deref());
        
        // Draw cursor if focused (end of text)
        if is_focused {
             let cursor_x = 8.0 + text_dims.x + 1.0;
             let cursor_pos = pos + Vec2::new(cursor_x, bounds.height * 0.5);
             ctx.primitives.draw_rect(
                 cursor_pos, 
                 Vec2::new(1.0, size * 0.8 * 0.5), 
                 Vec4::new(0.8, 0.8, 0.8, 1.0), 
                 [0.0; 4], 
                 0.0
             );
        }

        // Spinners
        if *show_spinner {
             let spinner_x = pos.x + bounds.width - spinner_width;
             let btn_h = bounds.height * 0.5;
             
             // Top Button (Increment)
             let top_center = Vec2::new(spinner_x + spinner_width * 0.5, pos.y + btn_h * 0.5);
             ctx.primitives.draw_rect(
                 top_center,
                 Vec2::new(spinner_width * 0.5 - 1.0, btn_h * 0.5 - 1.0),
                 Vec4::new(style.spinner_color.0, style.spinner_color.1, style.spinner_color.2, style.spinner_color.3),
                 [0.0, style.corner_radius, 0.0, 0.0], // Top right radius
                 0.0
             );

             // Bottom Button (Decrement)
             let bot_center = Vec2::new(spinner_x + spinner_width * 0.5, pos.y + btn_h * 1.5);
             ctx.primitives.draw_rect(
                 bot_center,
                 Vec2::new(spinner_width * 0.5 - 1.0, btn_h * 0.5 - 1.0),
                 Vec4::new(style.spinner_color.0, style.spinner_color.1, style.spinner_color.2, style.spinner_color.3),
                 [0.0, 0.0, style.corner_radius, 0.0], // Bottom right radius
                 0.0
             );

             // Simple arrow icons using lines
             let arrow_col = Vec4::new(0.8, 0.8, 0.8, 1.0);
             
             // Up Arrow
             let up_cy = top_center.y;
             let up_cx = top_center.x;
             ctx.primitives.draw_line(
                 Vec2::new(up_cx - 3.0, up_cy + 2.0),
                 Vec2::new(up_cx, up_cy - 1.0),
                 1.5,
                 arrow_col
             );
             ctx.primitives.draw_line(
                 Vec2::new(up_cx, up_cy - 1.0),
                 Vec2::new(up_cx + 3.0, up_cy + 2.0),
                 1.5,
                 arrow_col
             );

             // Down Arrow
             let dn_cy = bot_center.y;
             let dn_cx = bot_center.x;
             ctx.primitives.draw_line(
                 Vec2::new(dn_cx - 3.0, dn_cy - 2.0),
                 Vec2::new(dn_cx, dn_cy + 1.0),
                 1.5,
                 arrow_col
             );
             ctx.primitives.draw_line(
                 Vec2::new(dn_cx, dn_cy + 1.0),
                 Vec2::new(dn_cx + 3.0, dn_cy - 2.0),
                 1.5,
                 arrow_col
             );
        }
    }
    Widget::Autocomplete {
        id,
        value,
        placeholder,
        suggestions,
        max_visible,
        bounds,
        style,
        ..
    } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
        let half_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
        
        let is_focused = ctx.interaction.map(|s| s.focused_id.as_deref() == Some(id)).unwrap_or(false);
        
        // Background
        let input_bg = if is_focused {
             style.background_focused.unwrap_or(style.background.unwrap_or((0.15, 0.15, 0.18, 1.0)))
        } else {
             style.background.unwrap_or((0.1, 0.1, 0.12, 1.0))
        };
        ctx.primitives.draw_rect(center, half_size, Vec4::new(input_bg.0, input_bg.1, input_bg.2, input_bg.3), [style.corner_radius; 4], 0.0);
        
        // Validation Error
        let has_error = ctx.interaction.as_ref()
           .and_then(|i| i.validation_errors.get(id))
           .map(|e| !e.is_empty())
           .unwrap_or(false);
           
        // Border
        let border = if is_focused {
            style.border_focused.as_ref().or(style.border.as_ref())
        } else {
            style.border.as_ref()
        };
        
        let error_border_color = (1.0, 0.2, 0.2, 1.0);
        
        if let Some(b) = border {
             let color = if has_error { error_border_color } else { b.color };
             ctx.primitives.draw_rect(center, half_size, Vec4::new(color.0, color.1, color.2, color.3), [style.corner_radius; 4], b.width);
        } else if has_error {
             ctx.primitives.draw_rect(center, half_size, Vec4::new(error_border_color.0, error_border_color.1, error_border_color.2, error_border_color.3), [style.corner_radius; 4], 1.5);
        } else if is_focused && style.border_focused.is_none() {
             ctx.primitives.draw_rect(center, half_size, Vec4::new(0.75, 0.75, 0.75, 1.0), [style.corner_radius; 4], 1.5);
        }

        // Text
        let text_val = if value.is_empty() { placeholder } else { value };
        let col = if value.is_empty() { Vec4::new(0.5, 0.5, 0.6, 1.0) } else { Vec4::new(style.text_color.0, style.text_color.1, style.text_color.2, style.text_color.3) };
        let size = 14.0;
        
        let text_dims = ctx.text.measure(text_val, size, style.font.as_deref());
        let text_y = (bounds.height - text_dims.y) * 0.5;
        let text_pos = pos + Vec2::new(8.0, text_y);
        
        ctx.text.draw(
            ctx.device,
            ctx.queue,
            text_val,
            text_pos,
            size,
            col,
            HorizontalAlign::Left,
            style.font.as_deref()
        );
        
        // Cursor
        if is_focused {
            let cursor_x = if value.is_empty() {
                text_pos.x
            } else {
                text_pos.x + ctx.text.measure(value, size, style.font.as_deref()).x
            };
            
            // Simple Blink
            let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
            if (time / 500) % 2 == 0 {
                let cc = style.cursor_color;
                 ctx.primitives.draw_line(
                     Vec2::new(cursor_x, text_pos.y),
                     Vec2::new(cursor_x, text_pos.y + text_dims.y),
                     1.5,
                     Vec4::new(cc.0, cc.1, cc.2, cc.3)
                 );
            }
        }
        
        // Dropdown
        if is_focused && !suggestions.is_empty() {
             let item_height = 24.0;
             let count = suggestions.len().min(*max_visible);
             let dd_height = count as f32 * item_height;
             let dd_width = bounds.width;
             
             // Position dropdown below input
             let dd_pos = pos + Vec2::new(0.0, bounds.height + 2.0); 
             let dd_center = dd_pos + Vec2::new(dd_width * 0.5, dd_height * 0.5);
             let dd_half = Vec2::new(dd_width * 0.5, dd_height * 0.5);

             // Clone data for closure
             let style = style.clone();
             let suggestions = suggestions.clone();
             let id = id.clone();
             let hovered_action = ctx.interaction.as_ref().and_then(|s| s.hovered_action.clone());

             let draw_dropdown = move |renderer: &mut crate::renderer::GloomyRenderer, _device: &wgpu::Device, _queue: &wgpu::Queue| {
                  // Use overlay renderers for Z-order correctness
                  let (primitives, text) = renderer.split_overlay_mut();
                  
                  // DD Background
                  let dd_bg = style.dropdown_background.unwrap_or((0.12, 0.12, 0.15, 1.0));
                  primitives.draw_rect(
                      dd_center,
                      dd_half,
                      Vec4::new(dd_bg.0, dd_bg.1, dd_bg.2, dd_bg.3),
                      [2.0; 4], 0.0
                  );
                  
                  // DD Border
                  if let Some(b) = &style.dropdown_border {
                       primitives.draw_rect(
                           dd_center,
                           dd_half,
                           Vec4::new(b.color.0, b.color.1, b.color.2, b.color.3),
                           [2.0; 4], b.width
                       );
                  }

                  // Items
                  for (i, item) in suggestions.iter().take(count).enumerate() {
                      let item_y = dd_pos.y + i as f32 * item_height;
                      let item_center = Vec2::new(dd_pos.x + dd_width * 0.5, item_y + item_height * 0.5);
                      let item_half = Vec2::new(dd_width * 0.5, item_height * 0.5);
                      
                      // Hover check
                      let action_id = format!("{}:opt:{}", id, i);
                      let is_hovered = hovered_action.as_deref() == Some(&action_id);
                      
                      if is_hovered {
                          let hl = style.dropdown_highlight_color;
                          primitives.draw_rect(
                              item_center,
                              item_half,
                              Vec4::new(hl.0, hl.1, hl.2, hl.3),
                              [1.0; 4], 0.0
                          );
                      }
                      
                      // Item Text
                      let tc = style.dropdown_text_color;
                      let item_text_pos = Vec2::new(dd_pos.x + 8.0, item_y + 4.0); 
                      text.draw(
                          _device,
                          _queue,
                          item,
                          item_text_pos,
                          14.0,
                          Vec4::new(tc.0, tc.1, tc.2, tc.3),
                          HorizontalAlign::Left,
                          style.font.as_deref()
                      );
                  }
             };

             if let Some(queue) = &mut ctx.deferred_draws {
                 queue.push(Box::new(draw_dropdown));
             } else {
                 // Fallback if no deferred queue available (e.g. tests)
                 // Just skip or log? For now skip to be safe.
             }
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
          .and_then(|grid_id| {
              ctx.interaction.as_ref()
                  .and_then(|i| i.scroll_offsets.get(grid_id.as_str()))
          })
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
          
          let visible_rows_count = (visible_height / row_height).ceil() as usize;
          
          let buffer_size = 5;
          let calculated_start = (scroll_offset / row_height).floor() as usize;
          let start_row = calculated_start.saturating_sub(buffer_size);
          
          let end_row = (calculated_start + visible_rows_count + buffer_size).min(row_count);
          
          let content_y = pos.y + header_height;
          
          // --- SCISSOR START ---
          let s = ctx.scale_factor;
          let my_scissor_rect = (
              (pos.x * s) as u32,
              (content_y * s) as u32,
              (bounds.width * s) as u32,
              ((visible_height + 0.5) * s) as u32 // +0.5 to prevent sub-pixel cutoff
          );
          
          // Helper to intersect scissors
          let intersect_scissor = |old: Option<(u32,u32,u32,u32)>, new: (u32,u32,u32,u32)| -> (u32,u32,u32,u32) {
               if let Some((ox, oy, ow, oh)) = old {
                   let x = ox.max(new.0);
                   let y = oy.max(new.1);
                   let r = (ox + ow).min(new.0 + new.2);
                   let b = (oy + oh).min(new.1 + new.3);
                   let w = r.saturating_sub(x);
                   let h = b.saturating_sub(y);
                   (x, y, w, h)
               } else {
                   new
               }
          };
          
          let old_prim_scissor = ctx.primitives.set_scissor(None);
          let new_prim_scissor = intersect_scissor(old_prim_scissor, my_scissor_rect);
          ctx.primitives.set_scissor(Some(new_prim_scissor));
          
          let old_text_scissor = ctx.text.set_scissor(None);
          let new_text_scissor = intersect_scissor(old_text_scissor, my_scissor_rect);
          ctx.text.set_scissor(Some(new_text_scissor));
          
          // Images might be used in cells? (Future proofing)
          // let old_img_scissor = ctx.images.set_scissor(None); 
          // ...
          
          let mut r = start_row;
          while r < end_row {
               let row_y = pos.y + header_height + ((r as f32) * row_height) - scroll_offset;
               let center_y = row_y + row_height * 0.5;
               
               // Clip check can be loose now since strictly scissoring
               // But keeps it for skipping totally off-screen primitives if buffer is large
               if row_y > content_y + visible_height + (buffer_size as f32 * row_height) {
                   break; 
               }
                if row_y + row_height < content_y - (buffer_size as f32 * row_height) {
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
                   
                   // Check if this cell is being edited
                   let is_editing = id.as_ref().map(|grid_id| {
                       ctx.interaction.as_ref()
                           .map(|s| s.is_editing_cell(grid_id, r, c))
                           .unwrap_or(false)
                   }).unwrap_or(false);
                   
                   if is_editing {
                       // Draw edit input background
                       let cell_center = Vec2::new(x + w * 0.5, center_y);
                       let cell_half = Vec2::new(w * 0.5 - 1.0, row_height * 0.5 - 1.0);
                       ctx.primitives.draw_rect(
                           cell_center,
                           cell_half,
                           Vec4::new(0.2, 0.25, 0.35, 1.0),
                           [2.0; 4],
                           0.0
                       );
                       
                       // Draw edit buffer text
                       let edit_text = ctx.interaction.as_ref()
                           .map(|s| s.grid_edit_buffer.as_str())
                           .unwrap_or("");
                       ctx.text.draw(
                           ctx.device,
                           ctx.queue,
                           edit_text,
                           Vec2::new(x + style.cell_padding, center_y - 6.5),
                           13.0,
                           Vec4::new(1.0, 1.0, 1.0, 1.0),
                           HorizontalAlign::Left,
                           None
                       );
                       
                       // Draw cursor
                       let cursor_x = x + style.cell_padding + (edit_text.len() as f32 * 7.0);
                       ctx.primitives.draw_rect(
                           Vec2::new(cursor_x, center_y),
                           Vec2::new(1.0, 6.0),
                           Vec4::new(1.0, 1.0, 1.0, 0.8),
                           [0.0; 4],
                           0.0
                       );
                   } else {
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
                   }
                   
                   // Check for dirty state (modified)
                   let is_dirty = id.as_ref().map(|grid_id| {
                       ctx.interaction.as_ref()
                           .map(|s| s.is_dirty(grid_id, r, c))
                           .unwrap_or(false)
                   }).unwrap_or(false);
                   
                   if is_dirty {
                       // Draw small red triangle in top-right corner
                       // Using a specialized primitive or just a small rect for now (triangle primitive not readily available?)
                       // Actually, we can just draw a small red rect or use lines. Let's use a small square for clarity.
                       let indicator_size = 6.0;
                       let cell_right = x + w;
                       let cell_top = center_y - row_height * 0.5;
                       
                       ctx.primitives.draw_rect(
                           Vec2::new(cell_right - indicator_size * 0.5 - 2.0, cell_top + indicator_size * 0.5 + 2.0),
                           Vec2::new(indicator_size * 0.5, indicator_size * 0.5),
                           Vec4::new(1.0, 0.2, 0.2, 1.0),
                           [1.0; 4],
                           0.0
                       );
                   }
                   x += w;
               }
               r += 1;
           }
           
           // --- SCISSOR RESTORE ---
           ctx.primitives.set_scissor(old_prim_scissor);
           ctx.text.set_scissor(old_text_scissor);
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
            
            // Debug header text
            if i == 0 {
                log::info!("Header[0] '{}' at ({}, {})", 
                           col.header, x + style.cell_padding, pos.y + header_height * 0.5);
            }

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

    Widget::DatePicker { 
        id, value, placeholder, format, style, bounds, min_date, max_date, .. 
    } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let center = pos + Vec2::new(bounds.width * 0.5, bounds.height * 0.5);
        let half_size = Vec2::new(bounds.width * 0.5, bounds.height * 0.5);

        let is_focused = ctx.interaction.as_ref().map(|s| s.focused_id.as_deref() == Some(id)).unwrap_or(false);
        
        // Draw Input Background
        let bg_color = if is_focused { style.background_focused } else { style.background }.unwrap_or_default();
        let border = if is_focused { style.border_focused } else { style.border };

        if bg_color.3 > 0.0 {
            ctx.primitives.draw_rect(center, half_size, Vec4::from(bg_color), [style.corner_radius; 4], 0.0);
        }
        // Note: Border drawing skipped - draw_border not implemented

        // Draw Text
        let text_str = if let Some(date) = value {
             date.format(format).to_string()
        } else {
             placeholder.clone()
        };
        
        let text_color = if value.is_some() { style.text_color } else { style.placeholder_color };
        
        // Add padding
        let text_pos = Vec2::new(pos.x + 8.0, center.y);
        
        ctx.text.draw(
            ctx.device,
            ctx.queue,
            &text_str,
            text_pos,
            16.0, 
            Vec4::from(text_color),
            HorizontalAlign::Left,
            style.font.as_deref()
        );

        // Draw Overlay (Deferred)
        if is_focused {
             let bounds = *bounds;
             let style = style.clone();
             let id = id.clone();
             let value = *value;
             let min_date = *min_date;
             let max_date = *max_date;
             let offset = ctx.offset;
             
             let view_state = ctx.interaction.and_then(|s| s.calendar_view_state.get(&id).copied());
             let hovered_action = ctx.interaction.and_then(|s| s.hovered_action.clone());
             
             if let Some(deferred) = ctx.deferred_draws.as_mut() {
                 deferred.push(Box::new(move |renderer, device, queue| {
                     // Get Overlay Renderer
                     let (primitives, text) = renderer.split_overlay_mut();
                     
                     let dd_width = bounds.width.max(250.0);
                     let dd_x = offset.x + bounds.x;
                     let dd_y = offset.y + bounds.y + bounds.height + 2.0;

                     let header_height = 30.0;
                     let day_names_height = 28.0;
                     let row_height = 30.0;
                     let padding = 5.0;
                     let dd_height = header_height + day_names_height + 6.0 * row_height + padding * 2.0;
                     
                     let dd_center = Vec2::new(dd_x + dd_width * 0.5, dd_y + dd_height * 0.5);
                     let dd_half = Vec2::new(dd_width * 0.5, dd_height * 0.5);
                     
                     // Background
                     if let Some(bg) = style.calendar_background {
                         primitives.draw_rect(dd_center, dd_half, Vec4::from(bg), [style.corner_radius; 4], 0.0);
                     }
                     // Note: Border drawing skipped - draw_border not implemented
                     
                     // Determine View Date
                     let (view_month, view_year) = view_state
                         .or_else(|| value.map(|d| (d.month(), d.year())))
                         .unwrap_or_else(|| {
                             let now = chrono::Local::now().naive_local().date();
                             (now.month(), now.year())
                         });

                     // Draw Header
                     let header_y = dd_y + padding + header_height * 0.5;
                     
                     // Prev Button (<)
                     let prev_hover = hovered_action.as_deref() == Some(&format!("{}:prev", id));
                     let prev_color = if prev_hover { style.day_hover_color } else { style.month_header_color };
                     text.draw(device, queue, "<", Vec2::new(dd_x + 20.0, header_y), 20.0, Vec4::from(prev_color), HorizontalAlign::Center, None);

                     // Next Button (>)
                     let next_hover = hovered_action.as_deref() == Some(&format!("{}:next", id));
                     let next_color = if next_hover { style.day_hover_color } else { style.month_header_color };
                     text.draw(device, queue, ">", Vec2::new(dd_x + dd_width - 20.0, header_y), 20.0, Vec4::from(next_color), HorizontalAlign::Center, None);

                     // Month Year Text
                     let header_str = format!("{} {}", chrono::Month::try_from(view_month as u8).map(|m| m.name()).unwrap_or(""), view_year);
                     text.draw(device, queue, &header_str, Vec2::new(dd_x + dd_width * 0.5, header_y), 18.0, Vec4::from(style.month_header_color), HorizontalAlign::Center, None);
                     
                     // Day Names
                     let days = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
                     let day_names_y = dd_y + padding + header_height + day_names_height * 0.5;
                     let cell_w = (dd_width - padding * 2.0) / 7.0;
                     
                     for (i, day) in days.iter().enumerate() {
                         let cx = dd_x + padding + cell_w * i as f32 + cell_w * 0.5;
                         text.draw(device, queue, day, Vec2::new(cx, day_names_y), 14.0, Vec4::from(style.month_header_color), HorizontalAlign::Center, None);
                     }
                     
                     // Grid
                     let grid_start_y = dd_y + padding + header_height + day_names_height;
                     
                     if let Some(first_day) = NaiveDate::from_ymd_opt(view_year, view_month, 1) {
                         let start_weekday = first_day.weekday().num_days_from_monday(); // 0=Mon
                         let offset = start_weekday as i64;
                         
                         for row in 0..6 {
                             for col in 0..7 {
                                 let day_idx = (row * 7 + col) as i64;
                                 let date_offset = day_idx - offset;
                                 
                                 if let Some(date) = first_day.checked_add_signed(chrono::Duration::days(date_offset)) {
                                     let is_current_month = date.month() == view_month;
                                     let is_selected = value == Some(date);
                                     let is_today = date == chrono::Local::now().naive_local().date();
                                     let action_id = format!("{}:day:{}", id, date.format("%Y-%m-%d"));
                                     let is_hovered = hovered_action.as_ref() == Some(&action_id);
                                     
                                     let cx = dd_x + padding + cell_w * col as f32 + cell_w * 0.5;
                                     let cy = grid_start_y + row_height * row as f32 + row_height * 0.5;
                                     
                                     // Draw Cell Background
                                     if is_selected {
                                         primitives.draw_rect(Vec2::new(cx, cy), Vec2::new(cell_w * 0.45, row_height * 0.45), Vec4::from(style.selected_day_color), [4.0; 4], 0.0);
                                     } else if is_hovered {
                                         primitives.draw_rect(Vec2::new(cx, cy), Vec2::new(cell_w * 0.45, row_height * 0.45), Vec4::from(style.day_hover_color), [4.0; 4], 0.0);
                                     }
                                     
                                     // Draw Text
                                     let mut color = if is_selected {
                                         Vec4::ONE // White on selection
                                     } else if is_today && is_current_month {
                                         Vec4::from(style.today_color)
                                     } else if !is_current_month {
                                         Vec4::new(0.5, 0.5, 0.5, 1.0) // Gray
                                     } else {
                                         Vec4::from(style.day_text_color)
                                     };
                                     
                                     // Check min/max bounds
                                     if let Some(min) = min_date { if date < min { color.w = 0.3; } }
                                     if let Some(max) = max_date { if date > max { color.w = 0.3; } }

                                     // Offset y by half font size to vertically center
                                     let font_size = 16.0;
                                     let text_y = cy - font_size * 0.5;
                                     text.draw(device, queue, &date.day().to_string(), Vec2::new(cx, text_y), font_size, color, HorizontalAlign::Center, None);
                                 }
                             }
                         }
                     }
                 }));
             }
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
    
    Widget::KpiCard { title, value, trend, style, bounds, .. } => {
         let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
         
         // Background
         ctx.primitives.draw_rect(
             Vec2::new(pos.x + bounds.width * 0.5, pos.y + bounds.height * 0.5),
             Vec2::new(bounds.width * 0.5, bounds.height * 0.5),
             Vec4::from(style.background),
             [style.corner_radius; 4],
             style.border_width
         );
         
         // Title
         let title_pos = pos + Vec2::new(12.0, 12.0);
         ctx.text.draw(
             ctx.device, ctx.queue, title, title_pos, style.label_size,
             Vec4::from(style.label_color), HorizontalAlign::Left, None
         );
         
         // Value
         let val_pos = pos + Vec2::new(12.0, bounds.height * 0.6);
         ctx.text.draw(
              ctx.device, ctx.queue, value, val_pos, style.value_size,
              Vec4::from(style.value_color), HorizontalAlign::Left, None
         );
         
         if let Some(trend) = trend {
             let color = match trend.direction {
                 crate::kpi::TrendDirection::Up => style.trend_up_color,
                 crate::kpi::TrendDirection::Down => style.trend_down_color,
                 crate::kpi::TrendDirection::Neutral => style.trend_neutral_color,
             };
              // Draw trend arrow/text at top right
              let trend_pos = pos + Vec2::new(bounds.width - 12.0, 12.0 + style.label_size * 0.5);
               ctx.text.draw(
                  ctx.device, ctx.queue, &trend.value, trend_pos, style.label_size,
                  Vec4::from(color), HorizontalAlign::Right, None
              );
         }
    }

    Widget::Tab {
        id, tabs, selected, orientation, style, bounds, ..
    } => {
        let pos = ctx.offset + Vec2::new(bounds.x, bounds.y);
        let header_rect = match orientation {
            crate::widget::Orientation::Horizontal => {
                 crate::Rect { x: pos.x, y: pos.y, width: bounds.width, height: 32.0 }
            }
            crate::widget::Orientation::Vertical => {
                 crate::Rect { x: pos.x, y: pos.y, width: 120.0, height: bounds.height }
            }
        };

        // Draw Tab Bar Background
        ctx.primitives.draw_rect(
             Vec2::new(header_rect.x + header_rect.width * 0.5, header_rect.y + header_rect.height * 0.5),
             Vec2::new(header_rect.width * 0.5, header_rect.height * 0.5),
             Vec4::from(style.background),
             [0.0; 4],
             0.0
        );

        // Draw Tabs
        let tab_count = tabs.len();
        if tab_count > 0 {
             let (tab_w, tab_h) = match orientation {
                 crate::widget::Orientation::Horizontal => (header_rect.width / tab_count as f32, header_rect.height),
                 crate::widget::Orientation::Vertical => (header_rect.width, 32.0), // Fixed height per tab in vertical
             };

             for (i, tab) in tabs.iter().enumerate() {
                 let (tx, ty) = match orientation {
                     crate::widget::Orientation::Horizontal => (header_rect.x + i as f32 * tab_w, header_rect.y),
                     crate::widget::Orientation::Vertical => (header_rect.x, header_rect.y + i as f32 * tab_h),
                 };
                 
                 let color = if i == *selected { style.selected_color } else { style.unselected_color };
                 
                 // Draw Tab Rect
                 ctx.primitives.draw_rect(
                      Vec2::new(tx + tab_w * 0.5, ty + tab_h * 0.5),
                      Vec2::new(tab_w * 0.5, tab_h * 0.5),
                      Vec4::from(color),
                      [0.0; 4],
                      1.0 // Border logic can be enhanced
                 );
                 
                 // Draw Title
                 ctx.text.draw(
                      ctx.device, ctx.queue, &tab.title,
                      Vec2::new(tx + tab_w * 0.5, ty + tab_h * 0.5),
                      14.0, Vec4::ONE, HorizontalAlign::Center, None
                 );
             }
        }

        // Render Selected Content
        if let Some(tab) = tabs.get(*selected) {
             render_widget(&tab.content, ctx);
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
    let mut deferred_draws = Vec::new();
    render_ui_with_state(widget, renderer, device, queue, interaction, data_provider, None, &mut deferred_draws);
    
    // Execute deferred draws (overlays like dropdowns)
    for draw_op in deferred_draws {
        draw_op(renderer, device, queue);
    }
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
  deferred_draws: &mut Vec<Box<dyn FnOnce(&mut crate::renderer::GloomyRenderer, &wgpu::Device, &wgpu::Queue)>>,
) {
  let size = renderer.size();
  let surface_width = size.x as u32;
  let surface_height = size.y as u32;
  let scale_factor = renderer.scale_factor;
  
  let (primitives, text, images, textures) = renderer.split_mut();
  
  let mut ctx = RenderContext::new(
      primitives, 
      text, 
      images, 
      textures, 
      device, 
      queue, 
      interaction, 
      surface_width, 
      surface_height, 
      scale_factor, // Use local variable
      data_provider, 
      widget_tracker,
      Some(deferred_draws)
  );
  render_widget(widget, &mut ctx);
}

/// Performs a hit test on the widget tree.
///
/// Returns the first interactive widget found under the given point.
pub fn hit_test<'a>(
  widget: &'a Widget,
  point: Vec2,
  interaction: Option<&InteractionState>,
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
          if let Some(state) = interaction {
              if let Some(wid) = id {
                  if let Some(scroll) = state.scroll_offsets.get(wid) {
                      local_point += *scroll;
                  }
              }
          }
      }

      // Check children in reverse order (top to bottom)
      for child in children.iter().rev() {
        if let Some(result) = hit_test(child, local_point, interaction) {
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
    Widget::NumberInput { bounds, id, show_spinner, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             
             if *show_spinner {
                 let spinner_width = 20.0;
                 let spinner_x = bounds.x + bounds.width - spinner_width;
                 if point.x >= spinner_x {
                     let mid_y = bounds.y + bounds.height * 0.5;
                     if point.y < mid_y {
                         Some(HitTestResult { widget, action: format!("{}:up", id) })
                     } else {
                         Some(HitTestResult { widget, action: format!("{}:down", id) })
                     }
                 } else {
                     Some(HitTestResult { widget, action: id.clone() })
                 }
             } else {
                 Some(HitTestResult { widget, action: id.clone() })
             }
           } else {
             None
           }
    }
    Widget::Autocomplete {
        id, suggestions, max_visible, bounds, ..
    } => {
        let hit_input = point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height;
           
        if let Some(state) = interaction {
             if state.focused_id.as_deref() == Some(id) && !suggestions.is_empty() {
                 let item_height = 24.0;
                 let count = suggestions.len().min(*max_visible);
                 let dd_height = count as f32 * item_height;
                 let dd_width = bounds.width;
                 let dd_y = bounds.y + bounds.height + 2.0; 
                 
                 if point.x >= bounds.x && point.x <= bounds.x + dd_width 
                    && point.y >= dd_y && point.y <= dd_y + dd_height {
                        let local_y = point.y - dd_y;
                        let idx = (local_y / item_height) as usize;
                        if idx < count {
                             return Some(HitTestResult { widget, action: format!("{}:opt:{}", id, idx) });
                        }
                 }
             }
        }
        
        if hit_input {
             Some(HitTestResult { widget, action: id.clone() })
        } else {
             None
        }
    }
    Widget::ListView { id, items, style, bounds, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height 
        {
             let scroll_y = if let Some(state) = interaction {
                  state.scroll_offsets.get(id).map(|v| v.y).unwrap_or(0.0)
             } else { 0.0 };
             
             let local_y = point.y - bounds.y + scroll_y;
             let index = (local_y / style.item_height) as usize;
             
             if index < items.len() {
                 let action = format!("{}:{}", id, index);
                 Some(HitTestResult { widget, action })
             } else {
                 Some(HitTestResult { widget, action: id.clone() })
             }
        } else {
            None
        }
    }
    Widget::DatePicker {
        id, value, bounds, ..
    } => {
        let hit_input = point.x >= bounds.x && point.x <= bounds.x + bounds.width
            && point.y >= bounds.y && point.y <= bounds.y + bounds.height;

        if let Some(state) = interaction {
            if state.focused_id.as_deref() == Some(id) {
                let dd_width = bounds.width.max(250.0);
                let dd_x = bounds.x;
                // Layout constants matching render logic
                let header_height = 30.0;
                let day_names_height = 28.0;
                let row_height = 30.0;
                let padding = 5.0;
                let dd_height = header_height + day_names_height + 6.0 * row_height + padding * 2.0;

                let dd_y = bounds.y + bounds.height + 2.0;

                if point.x >= dd_x && point.x <= dd_x + dd_width 
                   && point.y >= dd_y && point.y <= dd_y + dd_height {
                    
                    let local_x = point.x - dd_x;
                    let local_y = point.y - dd_y;

                    // Header
                    if local_y <= header_height {
                         if local_x < 40.0 { return Some(HitTestResult { widget, action: format!("{}:prev", id) }); }
                         if local_x > dd_width - 40.0 { return Some(HitTestResult { widget, action: format!("{}:next", id) }); }
                         return Some(HitTestResult { widget, action: id.clone() }); 
                    }

                    // Grid
                    if local_y > header_height + day_names_height {
                        let grid_y = local_y - (header_height + day_names_height);
                        let row = (grid_y / row_height) as i32;
                        let cell_w = (dd_width - padding * 2.0) / 7.0;
                        let col = ((local_x - padding) / cell_w) as i32;
                        
                        if row >= 0 && row < 6 && col >= 0 && col < 7 {
                            // Determine View Date
                            let (view_month, view_year) = state.calendar_view_state.get(id)
                                .copied()
                                .or_else(|| value.map(|d| (d.month(), d.year())))
                                .unwrap_or_else(|| {
                                    let now = chrono::Local::now().naive_local().date();
                                    (now.month(), now.year())
                                });
                            
                            if let Some(first_day) = NaiveDate::from_ymd_opt(view_year, view_month, 1) {
                                let start_weekday = first_day.weekday().num_days_from_monday(); // 0=Mon
                                let offset = start_weekday as i64;
                                let day_idx = (row * 7 + col) as i64;
                                let date_offset = day_idx - offset;
                                
                                if let Some(date) = first_day.checked_add_signed(chrono::Duration::days(date_offset)) {
                                     return Some(HitTestResult { widget, action: format!("{}:day:{}", id, date.format("%Y-%m-%d")) });
                                }
                            }
                        }
                    }
                    return Some(HitTestResult { widget, action: id.clone() });
                }
            }
        }
        
        if hit_input {
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
                  
                  let scroll_y = if let Some(state) = interaction {
                       state.scroll_offsets.get(wid).map(|v| v.y).unwrap_or(0.0)
                  } else { 0.0 };
                  
                  let content_y = local_y - header_height + scroll_y;
                  if content_y >= 0.0 {
                      let row = (content_y / row_height).floor() as isize;
                      if row >= 0 {
                           // Calculate column based on local_x
                           let content_width = (bounds.width).max(0.0);
                           let mut total_fixed = 0.0;
                           let mut total_flex = 0.0;
                           
                           for col in columns.iter() {
                               match col.width {
                                   crate::datagrid::ColumnWidth::Fixed(w) => total_fixed += w,
                                   crate::datagrid::ColumnWidth::Flex(f) => total_flex += f,
                                   _ => {}
                               }
                           }
                           
                           let available = (content_width - total_fixed).max(0.0);
                           let mut cx = 0.0;
                           let mut col_idx = 0usize;
                           
                           for (i, col) in columns.iter().enumerate() {
                               let w = match col.width {
                                   crate::datagrid::ColumnWidth::Fixed(w) => w,
                                   crate::datagrid::ColumnWidth::Flex(f) => {
                                       if total_flex > 0.0 {
                                           (f / total_flex) * available
                                       } else { 0.0 }
                                   },
                                   _ => 0.0,
                               };
                               
                               if local_x >= cx && local_x < cx + w {
                                   col_idx = i;
                                   break;
                               }
                               cx += w;
                           }
                           
                           return Some(HitTestResult { widget, action: format!("{}:cell:{}:{}", wid, row, col_idx) });
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
    Widget::KpiCard { bounds, id, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
             Some(HitTestResult { widget, action: id.clone().unwrap_or_default() })
        } else {
             None
        }
    }
    Widget::Tab { id, bounds, tabs, selected, orientation, .. } => {
        if point.x >= bounds.x && point.x <= bounds.x + bounds.width
           && point.y >= bounds.y && point.y <= bounds.y + bounds.height {
            
            // Check Header
             let (header_w, header_h) = match orientation {
                 crate::widget::Orientation::Horizontal => (bounds.width, 32.0),
                 crate::widget::Orientation::Vertical => (120.0, bounds.height),
             };
             
             // Simple hit test for tabs
             let tab_count = tabs.len();
             if tab_count > 0 {
                  match orientation {
                      crate::widget::Orientation::Horizontal => {
                          if point.y >= bounds.y && point.y <= bounds.y + header_h {
                              let local_x = point.x - bounds.x;
                              let tab_w = header_w / tab_count as f32;
                              let idx = (local_x / tab_w) as usize;
                              if idx < tab_count {
                                  if let Some(wid) = id {
                                       return Some(HitTestResult { widget, action: format!("{}:tab:{}", wid, idx) });
                                  }
                              }
                          }
                      }
                      crate::widget::Orientation::Vertical => {
                          if point.x >= bounds.x && point.x <= bounds.x + header_w {
                              let local_y = point.y - bounds.y;
                              let tab_h = 32.0;
                              let idx = (local_y / tab_h) as usize;
                              if idx < tab_count {
                                   if let Some(wid) = id {
                                        return Some(HitTestResult { widget, action: format!("{}:tab:{}", wid, idx) });
                                   }
                              }
                          }
                      }
                  }
             }

             // Check Content
             if let Some(tab) = tabs.get(*selected) {
                 if let Some(res) = hit_test(&tab.content, point, interaction) {
                     return Some(res);
                 }
             }
             
             None
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
        | Widget::Slider { id: w_id, .. } 
        | Widget::NumberInput { id: w_id, .. }
        | Widget::Autocomplete { id: w_id, .. }
        | Widget::DatePicker { id: w_id, .. }
        | Widget::ToggleSwitch { id: w_id, .. } => {
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
        Widget::Tab { tabs, selected, .. } => {
            if let Some(tab) = tabs.get_mut(*selected) {
                 if let Some(w) = find_widget_mut(&mut tab.content, id) {
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
    
    // If Tab, recurse into selected content
    if let Widget::Tab { tabs, selected, .. } = widget {
        if let Some(tab) = tabs.get(*selected) {
            collect_focusable_ids_recursive(&tab.content, ids);
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

    // DEBUG: Log rendering
    if let Widget::Spacer { .. } = widget {
      // skip spacers
    } else {
       match widget {
           Widget::Container { bounds, id, .. } => if bounds.width > 0.0 { println!("Render Container {:?} {:?}", id, bounds); },
           Widget::Label { x, y, width, height, text, .. } => println!("Render Label '{}' at {},{} {}x{}", text, x, y, width, height),
           Widget::KpiCard { bounds, title, .. } => println!("Render KpiCard '{}' {:?}", title, bounds),
           Widget::Button { bounds, text, .. } => println!("Render Button '{}' {:?}", text, bounds),
           Widget::ListView { bounds, items, .. } => println!("Render ListView ({} items) {:?}", items.len(), bounds),
           _ => {},
       }
    }

    match widget {
        Widget::Tab { id: wid, selected, tabs, .. } => {
             if let Some(ref clicked) = ctx.clicked_id {
                 if let Some(wid) = wid {
                     let prefix = format!("{}:tab:", wid);
                     if clicked.starts_with(&prefix) {
                         if let Ok(idx) = clicked[prefix.len()..].parse::<usize>() {
                             if idx < tabs.len() && *selected != idx {
                                 *selected = idx;
                                 changed = true;
                             }
                         }
                     }
                 }
             }
             // Recurse into selected content
             if let Some(tab) = tabs.get_mut(*selected) {
                 let content_offset = Vec2::ZERO; // Layout already positioned it relatively
                 if handle_interactions(&mut tab.content, ctx, offset + content_offset) {
                     changed = true;
                 }
             }
        }

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

/// Helper to render a styled box (shadow, background, border).
fn draw_box(
    ctx: &mut RenderContext,
    pos: Vec2, // Top-left
    size: Vec2, // Width/Height (full)
    style: &BoxStyle,
) {
    let center = Vec2::new(pos.x + size.x * 0.5, pos.y + size.y * 0.5);
    let half_size = size * 0.5;

    // 1. Shadow
    if let Some(shadow) = style.shadow {
        if shadow.color.3 > 0.0 {
            // Shadow is drawn as a blurred rect behind the box.
            ctx.primitives.draw_styled_rect(
                center + Vec2::new(shadow.offset.0, shadow.offset.1),
                half_size,
                Vec4::from(shadow.color),
                Vec4::from(shadow.color),
                style.corner_radii,
                0.0, // Stroke width
                shadow.blur // Softness
            );
        }
    }

    // 2. Background (Gradient or Solid)
    if let Some(grad) = style.gradient {
        ctx.primitives.draw_styled_rect(
            center,
            half_size,
            Vec4::from(grad.start),
            Vec4::from(grad.end),
            style.corner_radii,
            0.0,
            0.0 
        );
    } else if let Some(bg) = style.background {
        if bg.3 > 0.0 {
            // Use draw_styled_rect for consistent radius handling, or basic draw_rect
            ctx.primitives.draw_rect(
                center,
                half_size,
                Vec4::from(bg),
                style.corner_radii,
                0.0
            );
        }
    }

    // 3. Border (Stroke)
    if let Some(border) = style.border {
        if border.width > 0.0 && border.color.3 > 0.0 {
             ctx.primitives.draw_rect(
                center,
                half_size,
                Vec4::from(border.color),
                style.corner_radii,
                border.width
             );
        }
    }
}
