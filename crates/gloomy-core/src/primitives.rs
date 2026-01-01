//! SDF-based primitive rendering with instanced drawing.
//!
//! Renders rectangles, circles, and lines using signed distance fields
//! for crisp edges at any resolution.

use glam::{Vec2, Vec4};

/// Instance data for a single primitive.
///
/// Packed for GPU buffer layout with 48 bytes per instance.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
  /// Position A: center (rect/circle) or start point (line)
  pub pos_a: Vec2,
  /// Position B: size (rect) or end point (line)
  pub pos_b: Vec2,
  /// RGBA color
  pub color: Vec4,
  /// Color End (for gradients). If same as color, solid fill.
  pub color_end: Vec4,
  /// Corner radii: [TopRight, BottomRight, TopLeft, BottomLeft]
  pub radii: [f32; 4],
  /// Primitive type: 0=Rect, 1=Circle, 2=Line
  pub prim_type: u32,
  /// Stroke width
  pub stroke_width: f32,
  /// Blur softness (SDF smoothing edge width)
  pub softness: f32,
  /// Padding
  pub _pad: u32,
}

#[derive(Debug, Clone)]
struct Batch {
  scissor: Option<(u32, u32, u32, u32)>,
  range: std::ops::Range<u32>,
}

/// Batched primitive renderer using instanced SDF shapes.
pub struct PrimitiveRenderer {
  pipeline: wgpu::RenderPipeline,
  bind_group: wgpu::BindGroup,
  uniform_buffer: wgpu::Buffer,
  instance_buffer: wgpu::Buffer,
  instances: Vec<Instance>,
  batches: Vec<Batch>,
  current_scissor: Option<(u32, u32, u32, u32)>,
  screen_size: Vec2,
  capacity: usize,
}

impl PrimitiveRenderer {
  /// Creates a new primitive renderer.
  pub fn new(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
  ) -> Self {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("GloomyPrimitiveShader"),
      source: wgpu::ShaderSource::Wgsl(
        include_str!("shaders/primitives.wgsl").into(),
      ),
    });

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("PrimitiveUniforms"),
      size: 16,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let bind_group_layout =
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("PrimitiveBindGroup"),
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }],
      });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: uniform_buffer.as_entire_binding(),
      }],
      label: None,
    });

    let pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
        label: None,
      });

    let pipeline =
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("GloomyPrimitivePipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
          module: &shader,
          entry_point: "vs_main",
          buffers: &[wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
              0 => Float32x2,
              1 => Float32x2,
              2 => Float32x4, // color
              3 => Float32x4, // color_end
              4 => Float32x4, // radii
              5 => Uint32,    // prim_type
              6 => Float32,   // stroke_width
              7 => Float32,   // softness
              8 => Uint32     // pad
            ],
          }],
          compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
          module: &shader,
          entry_point: "fs_main",
          targets: &[Some(wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
          })],
          compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
      });

    let initial_capacity = 1024;
    let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("InstanceBuffer"),
      size: (initial_capacity * std::mem::size_of::<Instance>()) as u64,
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    Self {
      pipeline,
      bind_group,
      uniform_buffer,
      instance_buffer,
      instances: Vec::with_capacity(initial_capacity),
      batches: Vec::new(),
      current_scissor: None,
      screen_size: Vec2::new(width as f32, height as f32),
      capacity: initial_capacity,
    }
  }

  /// Handles viewport resize.
  pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
    self.screen_size = Vec2::new(width as f32, height as f32);
    queue.write_buffer(
      &self.uniform_buffer,
      0,
      bytemuck::cast_slice(&[self.screen_size]),
    );
  }

  /// Sets the current scissor rect (x, y, width, height).
  /// Pass None to disable scissoring (full screen).
  /// Returns the previous scissor rect.
  pub fn set_scissor(&mut self, rect: Option<(u32, u32, u32, u32)>) -> Option<(u32, u32, u32, u32)> {
      let old = self.current_scissor;
      if self.current_scissor != rect {
          self.current_scissor = rect;
      }
      old
  }

  fn push_instance(&mut self, instance: Instance) {
      let instance_idx = self.instances.len() as u32;
      self.instances.push(instance);

      // Manage batches
      if let Some(last) = self.batches.last_mut() {
          if last.scissor == self.current_scissor && last.range.end == instance_idx {
              // Extend current batch
              last.range.end += 1;
              return;
          }
      }

      // New batch
      self.batches.push(Batch {
          scissor: self.current_scissor,
          range: instance_idx..(instance_idx + 1),
      });
  }

  /// Draws a rectangle with optional rounded corners.
  ///
  /// # Arguments
  /// * `pos` - Center position in screen coordinates
  /// * `size` - Half-extents (width/2, height/2)
  /// * `color` - RGBA color
  /// * `radii` - Corner radii [TR, BR, TL, BL]
  /// * `stroke_width` - Stroke width (0 for filled)
  pub fn draw_rect(
    &mut self,
    pos: Vec2,
    size: Vec2,
    color: Vec4,
    radii: [f32; 4],
    stroke_width: f32,
  ) {
    self.push_instance(Instance {
      pos_a: pos,
      pos_b: size * 2.0,
      color,
      color_end: color, // Solid fill by default
      radii,
      prim_type: 0,
      stroke_width,
      softness: 0.0,
      _pad: 0,
    });
  }

  /// Draws a rectangle with gradient and softness (shadow support).
  pub fn draw_styled_rect(
      &mut self,
      pos: Vec2,
      size: Vec2,
      color_start: Vec4,
      color_end: Vec4,
      radii: [f32; 4],
      stroke_width: f32,
      softness: f32,
  ) {
      self.push_instance(Instance {
          pos_a: pos,
          pos_b: size * 2.0,
          color: color_start,
          color_end,
          radii,
          prim_type: 0,
          stroke_width,
          softness,
          _pad: 0,
      });
  }

  /// Draws a filled rectangle (convenience method).
  pub fn fill_rect(&mut self, pos: Vec2, size: Vec2, color: Vec4) {
    self.draw_rect(pos, size, color, [0.0; 4], 0.0);
  }

  /// Draws a circle.
  ///
  /// # Arguments
  /// * `center` - Center position
  /// * `radius` - Circle radius
  /// * `color` - RGBA color
  /// * `stroke_width` - Stroke width (0 for filled)
  pub fn draw_circle(
    &mut self,
    center: Vec2,
    radius: f32,
    color: Vec4,
    stroke_width: f32,
  ) {
    self.push_instance(Instance {
      pos_a: center,
      pos_b: Vec2::ZERO,
      color,
      color_end: color,
      radii: [radius, 0.0, 0.0, 0.0],
      prim_type: 1,
      stroke_width,
      softness: 0.0,
      _pad: 0,
    });
  }

  /// Draws a line segment.
  ///
  /// # Arguments
  /// * `start` - Start position
  /// * `end` - End position
  /// * `thickness` - Line thickness
  /// * `color` - RGBA color
  pub fn draw_line(
    &mut self,
    start: Vec2,
    end: Vec2,
    thickness: f32,
    color: Vec4,
  ) {
    self.push_instance(Instance {
      pos_a: start,
      pos_b: end,
      color,
      color_end: color,
      radii: [thickness * 0.5, 0.0, 0.0, 0.0],
      prim_type: 2,
      stroke_width: 0.0,
      softness: 0.0,
      _pad: 0,
    });
  }

  /// Prepares instance data for GPU upload.
  pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    if self.instances.is_empty() {
      return;
    }

    // Grow buffer if needed
    if self.instances.len() > self.capacity {
      self.capacity = self.instances.len().next_power_of_two();
      self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("InstanceBuffer"),
        size: (self.capacity * std::mem::size_of::<Instance>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
    }

    queue.write_buffer(
      &self.instance_buffer,
      0,
      bytemuck::cast_slice(&self.instances),
    );
  }

  /// Renders all queued primitives.
  pub fn render<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
    if self.instances.is_empty() {
      return;
    }

    render_pass.set_pipeline(&self.pipeline);
    render_pass.set_bind_group(0, &self.bind_group, &[]);
    render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));

    for batch in &self.batches {
         if let Some((x, y, w, h)) = batch.scissor {
             // Clamp to screen size
             let fw = self.screen_size.x as u32;
             let fh = self.screen_size.y as u32;
             let sx = x.min(fw);
             let sy = y.min(fh);
             let sw = w.min(fw - sx);
             let sh = h.min(fh - sy);
             
             if sw > 0 && sh > 0 {
                 render_pass.set_scissor_rect(sx, sy, sw, sh);
             } else {
                 // Zero area scissor
                 render_pass.set_scissor_rect(0, 0, 0, 0);
             }
         } else {
             // Full screen
             render_pass.set_scissor_rect(0, 0, self.screen_size.x as u32, self.screen_size.y as u32);
         }
         render_pass.draw(0..6, batch.range.clone());
    }

    self.instances.clear();
    self.batches.clear();
    // Keep current_scissor or reset? Reset is safer per frame.
    self.current_scissor = None;
  }
}
