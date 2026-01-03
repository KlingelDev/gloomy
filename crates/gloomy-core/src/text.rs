use wgpu_text::glyph_brush::ab_glyph::{Font, FontArc, ScaleFont};
use glam::{Vec2, Vec4};
use wgpu_text::{
  glyph_brush::{HorizontalAlign, Section, Text, FontId},
  BrushBuilder,
};
use std::collections::HashMap;

/// A font family with variants (regular, bold, italic, bold-italic).
#[derive(Clone)]
pub struct FontFamily {
  pub name: String,
  pub regular_id: FontId,
  pub bold_id: Option<FontId>,
  pub italic_id: Option<FontId>,
  pub bold_italic_id: Option<FontId>,
}

impl FontFamily {
  /// Gets the appropriate font ID based on style flags.
  pub fn get_variant(&self, bold: bool, italic: bool) -> FontId {
    match (bold, italic) {
      (true, true) => self.bold_italic_id.unwrap_or(self.regular_id),
      (true, false) => self.bold_id.unwrap_or(self.regular_id),
      (false, true) => self.italic_id.unwrap_or(self.regular_id),
      (false, false) => self.regular_id,
    }
  }
}

/// Registry for managing multiple font families.
pub struct FontRegistry {
  families: HashMap<String, FontFamily>,
  default_family: String,
}

impl FontRegistry {
  /// Creates a new empty font registry.
  pub fn new(default_family: String) -> Self {
    Self {
      families: HashMap::new(),
      default_family,
    }
  }
  
  /// Registers a font family.
  pub fn register_family(&mut self, family: FontFamily) {
    self.families.insert(family.name.clone(), family);
  }
  
  /// Gets a font family by name, or the default if not found.
  pub fn get_family(&self, name: Option<&str>) -> Option<&FontFamily> {
    let family_name = name.unwrap_or(&self.default_family);
    self.families.get(family_name)
      .or_else(|| self.families.get(&self.default_family))
  }
  
  /// Gets the appropriate font ID for a family and style.
  pub fn get_font_id(
    &self,
    family_name: Option<&str>,
    bold: bool,
    italic: bool
  ) -> Option<FontId> {
    self.get_family(family_name)
      .map(|family| family.get_variant(bold, italic))
  }
}

/// Text renderer wrapping wgpu_text for TTF rendering.
pub struct TextRenderer {
  brush: wgpu_text::TextBrush<FontArc>,
  fonts: HashMap<String, FontId>,
  font_instances: Vec<FontArc>,
  font_registry: FontRegistry,
  /// Cache for glyph dimensions: (char, size_x10) -> width
  glyph_cache: HashMap<(char, u32), f32>,
  width: u32,
  height: u32,
  pending: Vec<(String, Vec2, f32, Vec4, Option<(u32, u32, u32, u32)>, HorizontalAlign, Option<String>)>,
  current_scissor: Option<(u32, u32, u32, u32)>,
  screen_size: Vec2,
  pub scale_factor: f32,
}

impl TextRenderer {
  /// Creates a new text renderer with font variants.
  pub fn new(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    font_bytes: &[u8],
  ) -> Self {
    let font = FontArc::try_from_vec(font_bytes.to_vec())
      .expect("Failed to load font");
      
    let mut brush =
      BrushBuilder::using_font(font.clone()).build(device, width, height, format);

    let mut fonts = HashMap::new();
    fonts.insert("default".to_string(), FontId(0));
    fonts.insert("regular".to_string(), FontId(0));
    
    // Initialize font registry with single default family
    let mut font_registry = FontRegistry::new("default".to_string());
    let default_family = FontFamily {
      name: "default".to_string(),
      regular_id: FontId(0),
      bold_id: None,
      italic_id: None,
      bold_italic_id: None,
    };
    font_registry.register_family(default_family);
    
    Self { 
        brush, 
        fonts,
        font_instances: vec![font],
        font_registry,
        width, 
        height, 
        pending: Vec::new(), 
        current_scissor: None, 
        screen_size: Vec2::new(width as f32, height as f32),
        scale_factor: 1.0, // Default 1.0, updated via resize
        glyph_cache: HashMap::new(),
    }
  }
  
  /// Creates a new text renderer with full font family variants.
  pub fn new_with_variants(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    regular_bytes: &[u8],
    bold_bytes: Option<&[u8]>,
    italic_bytes: Option<&[u8]>,
    bold_italic_bytes: Option<&[u8]>,
  ) -> Self {
    let regular_font = FontArc::try_from_vec(regular_bytes.to_vec())
      .expect("Failed to load regular font");
      
    let mut all_fonts = vec![regular_font.clone()];
    let mut fonts = HashMap::new();
    fonts.insert("default".to_string(), FontId(0));
    fonts.insert("regular".to_string(), FontId(0));
    
    // Add bold variant
    if let Some(bold) = bold_bytes {
      let bold_font = FontArc::try_from_vec(bold.to_vec())
        .expect("Failed to load bold font");
      fonts.insert("bold".to_string(), FontId(all_fonts.len()));
      all_fonts.push(bold_font);
    }
    
    // Add italic variant
    if let Some(italic) = italic_bytes {
      let italic_font = FontArc::try_from_vec(italic.to_vec())
        .expect("Failed to load italic font");
      fonts.insert("italic".to_string(), FontId(all_fonts.len()));
      all_fonts.push(italic_font);
    }
    
    // Add bold-italic variant
    if let Some(bold_italic) = bold_italic_bytes {
      let bi_font = FontArc::try_from_vec(bold_italic.to_vec())
        .expect("Failed to load bold+italic font");
      fonts.insert("bold-italic".to_string(), FontId(all_fonts.len()));
      all_fonts.push(bi_font);
    }
    
    let mut brush = BrushBuilder::using_fonts(all_fonts.clone())
      .build(device, width, height, format);

    // Initialize font registry with "Roboto" family
    let mut font_registry = FontRegistry::new("Roboto".to_string());
    let roboto_family = FontFamily {
      name: "Roboto".to_string(),
      regular_id: FontId(0),
      bold_id: if bold_bytes.is_some() { Some(FontId(1)) } else { None },
      italic_id: if italic_bytes.is_some() { 
        Some(FontId(if bold_bytes.is_some() { 2 } else { 1 }))
      } else { None },
      bold_italic_id: if bold_italic_bytes.is_some() {
        let idx = 1 + 
          (if bold_bytes.is_some() { 1 } else { 0 }) +
          (if italic_bytes.is_some() { 1 } else { 0 });
        Some(FontId(idx))
      } else { None },
    };
    font_registry.register_family(roboto_family);

    Self { 
        brush, 
        fonts,
        font_instances: all_fonts,
        font_registry,
        width, 
        height, 
        pending: Vec::new(), 
        current_scissor: None, 
        screen_size: Vec2::new(width as f32, height as f32),
        scale_factor: 1.0,
        glyph_cache: HashMap::new(),
    }
  }
  
  /// Creates a new text renderer with all available font families.
  /// Loads: Inter (2), Roboto (4), RobotoCondensed (4) = 10 fonts total.
  pub fn new_with_all_families(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    inter_regular: &[u8],
    inter_italic: &[u8],
    roboto_regular: &[u8],
    roboto_bold: &[u8],
    roboto_italic: &[u8],
    roboto_bold_italic: &[u8],
    roboto_condensed_regular: &[u8],
    roboto_condensed_bold: &[u8],
    roboto_condensed_italic: &[u8],
    roboto_condensed_bold_italic: &[u8],
  ) -> Self {
    let mut all_fonts = Vec::new();
    let mut font_registry = FontRegistry::new("Roboto".to_string());
    
    // Inter family (FontId 0-1)
    all_fonts.push(FontArc::try_from_vec(inter_regular.to_vec())
      .expect("Failed to load Inter Regular"));
    all_fonts.push(FontArc::try_from_vec(inter_italic.to_vec())
      .expect("Failed to load Inter Italic"));
    font_registry.register_family(FontFamily {
      name: "Inter".to_string(),
      regular_id: FontId(0),
      bold_id: None,
      italic_id: Some(FontId(1)),
      bold_italic_id: None,
    });
    
    // Roboto family (FontId 2-5)
    all_fonts.push(FontArc::try_from_vec(roboto_regular.to_vec())
      .expect("Failed to load Roboto Regular"));
    all_fonts.push(FontArc::try_from_vec(roboto_bold.to_vec())
      .expect("Failed to load Roboto Bold"));
    all_fonts.push(FontArc::try_from_vec(roboto_italic.to_vec())
      .expect("Failed to load Roboto Italic"));
    all_fonts.push(FontArc::try_from_vec(roboto_bold_italic.to_vec())
      .expect("Failed to load Roboto BoldItalic"));
    font_registry.register_family(FontFamily {
      name: "Roboto".to_string(),
      regular_id: FontId(2),
      bold_id: Some(FontId(3)),
      italic_id: Some(FontId(4)),
      bold_italic_id: Some(FontId(5)),
    });
    
    // RobotoCondensed family (FontId 6-9)
    all_fonts.push(FontArc::try_from_vec(roboto_condensed_regular.to_vec())
      .expect("Failed to load RobotoCondensed Regular"));
    all_fonts.push(FontArc::try_from_vec(roboto_condensed_bold.to_vec())
      .expect("Failed to load RobotoCondensed Bold"));
    all_fonts.push(FontArc::try_from_vec(roboto_condensed_italic.to_vec())
      .expect("Failed to load RobotoCondensed Italic"));
    all_fonts.push(FontArc::try_from_vec(roboto_condensed_bold_italic.to_vec())
      .expect("Failed to load RobotoCondensed BoldItalic"));
    font_registry.register_family(FontFamily {
      name: "RobotoCondensed".to_string(),
      regular_id: FontId(6),
      bold_id: Some(FontId(7)),
      italic_id: Some(FontId(8)),
      bold_italic_id: Some(FontId(9)),
    });
    
    // Create brush with all fonts
    let brush = BrushBuilder::using_fonts(all_fonts.clone())
      .build(device, width, height, format);
    
    let fonts = HashMap::new();
    
    Self {
      brush,
      fonts,
      font_instances: all_fonts,
      font_registry,
      width,
      height,
      pending: Vec::new(),
      current_scissor: None,
      screen_size: Vec2::new(width as f32, height as f32),
      scale_factor: 1.0,
      glyph_cache: HashMap::new(),
    }
  }

  /// Adds a new font to the renderer.
  /// NOTE: wgpu-text 0.8 doesn't support adding fonts after initialization.
  /// This is a placeholder for future implementation.
  pub fn add_font(&mut self, name: &str, font_bytes: &[u8]) {
    // TODO: wgpu-text 0.8 doesn't have add_font method
    // We would need to rebuild the brush with all fonts
    // For now, just track the font name
    let font = FontArc::try_from_vec(font_bytes.to_vec())
      .expect("Failed to load font");
    let next_id = FontId(self.font_instances.len());
    self.fonts.insert(name.to_string(), next_id);
    self.font_instances.push(font);
  }
  
  /// Gets the appropriate font name based on family and style flags.
  pub fn get_font_for_style<'a>(
    &self,
    family_name: Option<&'a str>,
    bold: bool,
    italic: bool
  ) -> Option<&'a str> {
    // Use font registry to get font ID
    if let Some(font_id) = self.font_registry.get_font_id(family_name, bold, italic) {
      // Map FontId back to font name for compatibility
      // For now, return family name or "Roboto" as default
      family_name.or(Some("Roboto"))
    } else {
      // Fallback to old logic for backwards compatibility
      match (bold, italic) {
        (true, true) => Some("bold-italic"),
        (true, false) => Some("bold"),
        (false, true) => Some("italic"),
        (false, false) => family_name.or(Some("regular")),
      }
    }
  }
  
  /// Gets the font ID directly from the registry.
  pub fn get_font_id_from_registry(
    &self,
    family_name: Option<&str>,
    bold: bool,
    italic: bool
  ) -> Option<FontId> {
    self.font_registry.get_font_id(family_name, bold, italic)
  }

  /// Handles viewport resize.
  pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32, scale_factor: f32) {
    self.width = width;
    self.height = height;
    self.scale_factor = scale_factor;
    // Note: brush view must be PHYSICAL size
    self.screen_size = Vec2::new(width as f32 / scale_factor, height as f32 / scale_factor);
    self.brush.resize_view(width as f32, height as f32, queue);
  }
  
  /// Sets the current scissor rect.
  pub fn set_scissor(&mut self, rect: Option<(u32, u32, u32, u32)>) -> Option<(u32, u32, u32, u32)> {
      let old = self.current_scissor;
      self.current_scissor = rect;
      old
  }

  /// Queues text for rendering.
  pub fn draw(
    &mut self,
    _device: &wgpu::Device,
    _queue: &wgpu::Queue,
    text: &str,
    pos: Vec2,
    size: f32,
    color: Vec4,
    align: HorizontalAlign,
    font_name: Option<&str>,
  ) {
    self.pending.push((
        text.to_string(),
        pos,
        size,
        color,
        self.current_scissor,
        align,
        font_name.map(|s| s.to_string()),
    ));
  }

  /// Renders queued text.
  pub fn render(
    &mut self, 
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
  ) {
      if self.pending.is_empty() { return; }
      
      // Sort by scissor rect - REVERSE order so None renders LAST (on top)
      self.pending.sort_by(|a, b| b.4.cmp(&a.4));
      
      let scale = self.scale_factor;
      let width = self.width;
      let height = self.height;

      let mut current_idx = 0;
      while current_idx < self.pending.len() {
           // Find batch range
           let batch_scissor = self.pending[current_idx].4;
           let mut end_idx = current_idx + 1;
           while end_idx < self.pending.len() && self.pending[end_idx].4 == batch_scissor {
               end_idx += 1;
           }
           
           // Process batch
           let batch_items = &self.pending[current_idx..end_idx];
           let sections: Vec<Section> = batch_items.iter()
             .map(|(text, pos, size, color, _scissor, align, font_name)| {
                 let font_id = font_name.as_deref()
                     .and_then(|name| self.fonts.get(name))
                     .copied()
                     .unwrap_or(FontId(0));
                     
                 // Apply Scale Factor to Position and Size
                 let scaled_x = pos.x * scale;
                 let scaled_y = pos.y * scale;
                 let scaled_size = size * scale;

                 Section::default()
                     .add_text(
                         Text::new(text.as_str())
                           .with_scale(scaled_size)
                           .with_color([color.x, color.y, color.z, color.w])
                           .with_font_id(font_id),
                     )
                     .with_screen_position((scaled_x, scaled_y))
                     .with_layout(
                         wgpu_text::glyph_brush::Layout::default()
                             .h_align(*align)
                     )
             }).collect();
             
           // Queue and Draw
           self.brush.queue(device, queue, sections).unwrap();
           
           {
               let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                   label: Some("GloomyTextPass"),
                   color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                       view,
                       resolve_target: None,
                       ops: wgpu::Operations {
                           load: wgpu::LoadOp::Load,
                           store: wgpu::StoreOp::Store,
                       },
                   })],
                   depth_stencil_attachment: None,
                   timestamp_writes: None,
                   occlusion_query_set: None,
               });
               
               if let Some((x, y, w, h)) = batch_scissor {
                   let sx = x.min(width);
                   let sy = y.min(height);
                   let sw = w.min(width - sx);
                   let sh = h.min(height - sy);
                   
                   log::info!("  Batch scissor: Some({},{},{},{})", sx, sy, sw, sh);
                   rpass.set_scissor_rect(sx, sy, sw, sh);
               } else {
                   log::info!("  Batch scissor: None -> fullscreen ({},{})", width, height);
                   rpass.set_scissor_rect(0, 0, width, height);
               }
               
               self.brush.draw(&mut rpass);
           }
           
           current_idx = end_idx;
      }
      
      self.pending.clear();
      self.current_scissor = None;
  }

  /// Measures the bounds of the given text.
  pub fn measure(&self, text: &str, size: f32, font_name: Option<&str>) -> Vec2 {
      if text.is_empty() {
          return Vec2::new(0.0, size);
      }
      
      let font_id = font_name
          .and_then(|name| self.fonts.get(name))
          .copied()
          .unwrap_or(FontId(0));

      let font = &self.font_instances[font_id.0];
      let scaled_font = font.as_scaled(size);
      let mut width = 0.0;
      
      for c in text.chars() {
          let glyph_id = scaled_font.glyph_id(c);
          let advance = scaled_font.h_advance(glyph_id);
          width += advance;
      }
      
      Vec2::new(width, size)
  }
  
  /// Measures a single character with caching for performance.
  /// Uses cache key of (char, size*10) to avoid float key issues.
  pub fn measure_char_cached(
      &mut self,
      c: char,
      size: f32,
      font_name: Option<&str>
  ) -> f32 {
      // Convert size to integer key (multiply by 10 for precision)
      let size_key = (size * 10.0) as u32;
      let cache_key = (c, size_key);
      
      // Check cache first
      if let Some(&width) = self.glyph_cache.get(&cache_key) {
          return width;
      }
      
      // Measure and cache
      let font_id = font_name
          .and_then(|name| self.fonts.get(name))
          .copied()
          .unwrap_or(FontId(0));

      let font = &self.font_instances[font_id.0];
      let scaled_font = font.as_scaled(size);
      let glyph_id = scaled_font.glyph_id(c);
      let width = scaled_font.h_advance(glyph_id);
      
      // Cache result
      self.glyph_cache.insert(cache_key, width);
      
      width
  }
  
  // --- CAPTURE / REPLAY ---

  pub fn get_count(&self) -> usize {
      self.pending.len()
  }

  pub fn capture(&self, start_index: usize) -> TextSnapshot {
      let mut pending = Vec::new();
      if start_index < self.pending.len() {
          pending.extend_from_slice(&self.pending[start_index..]);
      }
      TextSnapshot { pending }
  }

  pub fn replay(&mut self, snapshot: &TextSnapshot, offset: Vec2) {
      for item in &snapshot.pending {
          let mut new_item = item.clone();
          new_item.1 += offset;
          self.pending.push(new_item);
      }
  }
}

#[derive(Clone, Debug)]
pub struct TextSnapshot {
    pub pending: Vec<(String, Vec2, f32, Vec4, Option<(u32, u32, u32, u32)>, HorizontalAlign, Option<String>)>,
}
