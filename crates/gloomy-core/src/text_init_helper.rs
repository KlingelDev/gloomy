// Helper function to create TextRenderer with all available font families
// This loads: Inter, Roboto, RobotoCondensed (10 font files total)
use wgpu_text::glyph_brush::ab_glyph::FontArc;
use wgpu_text::glyph_brush::FontId;
use wgpu_text::BrushBuilder;

pub fn create_text_renderer_with_all_fonts(
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
) -> TextRenderer {
    let mut all_fonts = Vec::new();
    let mut font_registry = FontRegistry::new("Roboto".to_string());
    
    // Load Inter family (variable fonts)
    let inter_regular_id = FontId(all_fonts.len());
    all_fonts.push(FontArc::try_from_vec(inter_regular.to_vec()).expect("Failed to load Inter Regular"));
    let inter_italic_id = FontId(all_fonts.len());
    all_fonts.push(FontArc::try_from_vec(inter_italic.to_vec()).expect("Failed to load Inter Italic"));
    
    font_registry.register_family(FontFamily {
      name: "Inter".to_string(),
      regular_id: inter_regular_id,
      bold_id: None, // Variable font
      italic_id: Some(inter_italic_id),
      bold_italic_id: None,
    });
    
    // Load Roboto family
    let roboto_regular_id = FontId(all_fonts.len());
    all_fonts.push(FontArc::try_from_vec(roboto_regular.to_vec()).expect("Failed to load Roboto Regular"));
    let roboto_bold_id = FontId(all_fonts.len());
    all_fonts.push(FontArc::try_from_vec(roboto_bold.to_vec()).expect("Failed to load Roboto Bold"));
    let roboto_italic_id = FontId(all_fonts.len());
    all_fonts.push(FontArc::try_from_vec(roboto_italic.to_vec()).expect("Failed to load Roboto Italic"));
    let roboto_bold_italic_id = FontId(all_fonts.len());
    all_fonts.push(FontArc::try_from_vec(roboto_bold_italic.to_vec()).expect("Failed to load Roboto BoldItalic"));
    
    font_registry.register_family(FontFamily {
      name: "Roboto".to_string(),
      regular_id: roboto_regular_id,
      bold_id: Some(roboto_bold_id),
      italic_id: Some(roboto_italic_id),
      bold_italic_id: Some(roboto_bold_italic_id),
    });
    
    // Load RobotoCondensed family
    let rc_regular_id = FontId(all_fonts.len());
    all_fonts.push(FontArc::try_from_vec(roboto_condensed_regular.to_vec()).expect("Failed to load RobotoCondensed Regular"));
    let rc_bold_id = FontId(all_fonts.len());
    all_fonts.push(FontArc::try_from_vec(roboto_condensed_bold.to_vec()).expect("Failed to load RobotoCondensed Bold"));
    let rc_italic_id = FontId(all_fonts.len());
    all_fonts.push(FontArc::try_from_vec(roboto_condensed_italic.to_vec()).expect("Failed to load RobotoCondensed Italic"));
    let rc_bold_italic_id = FontId(all_fonts.len());
    all_fonts.push(FontArc::try_from_vec(roboto_condensed_bold_italic.to_vec()).expect("Failed to load RobotoCondensed BoldItalic"));
    
    font_registry.register_family(FontFamily {
      name: "RobotoCondensed".to_string(),
      regular_id: rc_regular_id,
      bold_id: Some(rc_bold_id),
      italic_id: Some(rc_italic_id),
      bold_italic_id: Some(rc_bold_italic_id),
    });
    
    // Create brush with all fonts
    let brush = BrushBuilder::using_fonts(all_fonts.clone())
      .build(device, width, height, format);
    
    // Create TextRenderer
    use std::collections::HashMap;
    let fonts = HashMap::new(); // Legacy font name mapping
    
    TextRenderer {
      brush,
      fonts,
      font_instances: all_fonts,
      font_registry,
      width,
      height,
      pending: Vec::new(),
      current_scissor: None,
      screen_size: glam::Vec2::new(width as f32, height as f32),
    }
}
