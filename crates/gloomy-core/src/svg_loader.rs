use crate::texture::Texture;
use anyhow::{Context, Result};
use tiny_skia::{Pixmap, Transform};
use resvg::usvg::{self, Tree, Options, TreeParsing};

/// Loads an SVG from bytes, rasterizes it, and creates a wgpu Texture.
pub fn load_svg_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    svg_data: &[u8],
    width: u32,
    height: u32,
) -> Result<Texture> {
    let opt = Options::default();
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    let tree = Tree::from_data(svg_data, &opt).context("Failed to parse SVG")?;

    let mut pixmap = Pixmap::new(width, height).context("Failed to create pixmap")?;
    
    let size = tree.size;
    let target_w = width as f32;
    let target_h = height as f32;
    
    // Scale to fit (contain) and center
    let sx = target_w / size.width();
    let sy = target_h / size.height();
    let scale = sx.min(sy);
    
    let tx = (target_w - size.width() * scale) / 2.0;
    let ty = (target_h - size.height() * scale) / 2.0;
    
    let transform = Transform::from_translate(tx, ty).post_scale(scale, scale);
    
    let rtree = resvg::Tree::from_usvg(&tree);
    rtree.render(transform, &mut pixmap.as_mut());
    
    Texture::from_rgba(device, queue, pixmap.data(), width, height, Some("IconTexture"))
}


