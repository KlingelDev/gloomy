// Gloomy Image Shader

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    // Instance Data
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>, // Full size
    @location(2) color: vec4<f32>, // Tint
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct GlobalUniforms {
    screen_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: GlobalUniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Standard Quad [-0.5, 0.5]
    let idx = in.vertex_index % 6u;
    var x: f32;
    var y: f32;
    // UVs: [0, 1]
    var u: f32;
    var v: f32;
    
    switch idx {
        case 0u: { x = -0.5; y = -0.5; u = 0.0; v = 1.0; } // BL
        case 1u: { x = 0.5;  y = -0.5; u = 1.0; v = 1.0; } // BR
        case 2u: { x = 0.5;  y = 0.5;  u = 1.0; v = 0.0; } // TR
        case 3u: { x = -0.5; y = -0.5; u = 0.0; v = 1.0; } // BL
        case 4u: { x = 0.5;  y = 0.5;  u = 1.0; v = 0.0; } // TR
        case 5u: { x = -0.5; y = 0.5;  u = 0.0; v = 0.0; } // TL
        default: { x = 0.0; y = 0.0; u = 0.0; v = 0.0; }
    }
    
    let center = in.pos;
    let size = in.size;
    let local_pos = vec2<f32>(x, y) * size;
    let screen_pos = center + local_pos;
    
    let ndc = (screen_pos / globals.screen_size) * 2.0 - 1.0;
    out.position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    out.uv = vec2<f32>(u, v);
    out.color = in.color;
    
    return out;
}

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.uv);
    return tex * in.color;
}
