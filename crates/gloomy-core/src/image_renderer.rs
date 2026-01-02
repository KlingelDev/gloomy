use wgpu::util::DeviceExt;
use crate::texture::Texture;
use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ImageInstance {
    pos: Vec2,
    size: Vec2,
    color: Vec4,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GlobalUniforms {
    screen_size: Vec2,
}

#[derive(Clone)]
pub struct Batch {
    pub bind_group: std::sync::Arc<wgpu::BindGroup>,
    pub start_index: u32,
    pub count: u32,
    pub scissor: Option<(u32, u32, u32, u32)>,
}

pub struct ImageRenderer {
    pipeline: wgpu::RenderPipeline,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    instances: Vec<ImageInstance>,
    instance_buffer: wgpu::Buffer,
    batches: Vec<Batch>,
    current_scissor: Option<(u32, u32, u32, u32)>,
    screen_size: Vec2, 
}

impl ImageRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GloomyImageGlobalsLayout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GloomyImageTextureLayout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GloomyImagePipelineLayout"),
            bind_group_layouts: &[&globals_layout, &texture_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/image.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GloomyImagePipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<ImageInstance>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("GloomyImageGlobals"),
            contents: bytemuck::cast_slice(&[GlobalUniforms {
                screen_size: Vec2::new(width as f32, height as f32),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GloomyImageGlobalsBG"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GloomyImageInstances"),
            size: 1024,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            globals_buffer,
            globals_bind_group,
            texture_bind_group_layout: texture_layout,
            instances: Vec::new(),
            instance_buffer,
            batches: Vec::new(),
            current_scissor: None,
            screen_size: Vec2::new(width as f32, height as f32),
        }
    }

    /// Sets the current scissor rect (x, y, width, height).
    pub fn set_scissor(&mut self, rect: Option<(u32, u32, u32, u32)>) -> Option<(u32, u32, u32, u32)> {
        let old = self.current_scissor;
        self.current_scissor = rect;
        old
    }

    pub fn resize(&self, queue: &wgpu::Queue, width: u32, height: u32) {
        queue.write_buffer(
            &self.globals_buffer,
            0,
            bytemuck::cast_slice(&[GlobalUniforms {
                screen_size: Vec2::new(width as f32, height as f32),
            }]),
        );
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        texture: &Texture,
        pos: Vec2,
        size: Vec2,
        color: Vec4,
    ) {
        let instance = ImageInstance { pos, size, color };
        
        // Simple batching: if last batch used SAME texture, append.
        // But we store BindGroups. We can't easily check sameness of BG.
        // So we create BG every time?
        // Or we rely on the caller to group?
        // For simplicity: New batch for every draw call unless optimized later.
        // Actually, we can compare Texture ID if we have it. `texture.texture.global_id()` is not stable or exposed easily.
        // Let's just create a new batch every time for now. It's fine for UI.
        
        // Optimization: Check cache of BGs?
        // Let's create BG on the fly.
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ImageBG"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });
        
        let start_index = self.instances.len() as u32;
        self.instances.push(instance);
        
        self.batches.push(Batch {
            bind_group: std::sync::Arc::new(bg),
            start_index,
            count: 1,
            scissor: self.current_scissor,
        });
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.instances.is_empty() {
            return;
        }

        // Resize buffer if needed
        let needed = (self.instances.len() * std::mem::size_of::<ImageInstance>()) as u64;
        if self.instance_buffer.size() < needed {
            self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("GloomyImageInstances"),
                size: needed.max(1024),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );
        
        // Clear instances for next frame?
        // No, render pass needs them.
        // clear() happens after render.
    }

    pub fn render<'rpass>(&'rpass mut self, rpass: &mut wgpu::RenderPass<'rpass>) {
        if self.batches.is_empty() {
            return;
        }

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.globals_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.instance_buffer.slice(..));

        for batch in &self.batches {
            if let Some((x, y, w, h)) = batch.scissor {
                 let fw = self.screen_size.x as u32;
                 let fh = self.screen_size.y as u32;
                 let sx = x.min(fw);
                 let sy = y.min(fh);
                 let sw = w.min(fw - sx);
                 let sh = h.min(fh - sy);
                 if sw > 0 && sh > 0 {
                     rpass.set_scissor_rect(sx, sy, sw, sh);
                 } else {
                     rpass.set_scissor_rect(0, 0, 0, 0);
                 }
            } else {
                 rpass.set_scissor_rect(0, 0, self.screen_size.x as u32, self.screen_size.y as u32);
            }
            rpass.set_bind_group(1, &batch.bind_group, &[]);
            rpass.draw(0..6, batch.start_index..(batch.start_index + batch.count));
        }
        self.current_scissor = None;
    }
    
    pub fn clear(&mut self) {
        self.instances.clear();
        self.batches.clear();
    }

    // --- CAPTURE / REPLAY ---

    pub fn get_counts(&self) -> (usize, usize) {
        (self.instances.len(), self.batches.len())
    }

    pub fn capture(&self, start_instance: usize, start_batch: usize) -> ImageSnapshot {
        // Clone instances
        let mut instances = Vec::new();
        if start_instance < self.instances.len() {
            instances.extend_from_slice(&self.instances[start_instance..]);
        }

        // Clone batches
        let mut batches = Vec::new();
        if start_batch < self.batches.len() {
            for batch in &self.batches[start_batch..] {
                let mut b = batch.clone();
                b.start_index -= start_instance as u32; // Normalize index relative to snapshot
                batches.push(b);
            }
        }
        
        ImageSnapshot { instances, batches }
    }

    pub fn replay(&mut self, snapshot: &ImageSnapshot, offset: Vec2) {
        let start_instance = self.instances.len() as u32;

        for instance in &snapshot.instances {
            let mut i = *instance;
            i.pos += offset;
            self.instances.push(i);
        }

        for batch in &snapshot.batches {
            let mut b = batch.clone();
            b.start_index += start_instance; // Shift index to current buffer
            self.batches.push(b);
        }
    }
}

#[derive(Clone)]
pub struct ImageSnapshot {
    pub instances: Vec<ImageInstance>,
    pub batches: Vec<Batch>,
}
