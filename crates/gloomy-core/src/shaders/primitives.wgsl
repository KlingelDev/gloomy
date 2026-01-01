// Gloomy SDF Primitives Shader
// Renders rectangles, circles, and lines using signed distance fields.

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    // Instance Data
    @location(0) pos_a: vec2<f32>,
    @location(1) pos_b: vec2<f32>,
    @location(2) color_start: vec4<f32>,
    @location(3) color_end: vec4<f32>,
     // Radii: TopRight, BottomRight, TopLeft, BottomLeft
    @location(4) radii: vec4<f32>,
    @location(5) prim_type: u32,
    @location(6) stroke_width: f32,
    @location(7) softness: f32,
    @location(8) _pad: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) dim: vec2<f32>,
    @location(2) color_start: vec4<f32>,
    @location(3) color_end: vec4<f32>,
    // Params: type, stroke_width, softness, unused
    @location(4) params: vec4<f32>,
    @location(5) radii: vec4<f32>,
};

struct GlobalUniforms {
    screen_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: GlobalUniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Base Quad vertex positions
    let idx = in.vertex_index % 6u;
    var x: f32;
    var y: f32;
    
    switch idx {
        case 0u: { x = -0.5; y = -0.5; }
        case 1u: { x = 0.5; y = -0.5; }
        case 2u: { x = 0.5; y = 0.5; }
        case 3u: { x = -0.5; y = -0.5; }
        case 4u: { x = 0.5; y = 0.5; }
        case 5u: { x = -0.5; y = 0.5; }
        default: { x = 0.0; y = 0.0; }
    }
    
    var center = in.pos_a;
    var size = in.pos_b;
    let stroke = in.stroke_width;
    let soft = in.softness;
    
    if (in.prim_type == 1u) {
        // Circle
        let r = in.radii.x;
        // Padding for AA, stroke and softness
        let padding = stroke + soft + 2.0; 
        size = vec2<f32>((r + padding) * 2.0, (r + padding) * 2.0);
    } else if (in.prim_type == 2u) {
        // Line logic (unchanged mostly, ignore soft for now)
        let delta = in.pos_b - in.pos_a;
        let len = length(delta);
        let thickness = in.radii.x;
        
        center = in.pos_a + delta * 0.5;
        size = vec2<f32>(len + thickness, thickness);
        
        let dir = normalize(delta);
        let cos_a = dir.x;
        let sin_a = dir.y;
        
        let rot_x = x * size.x * cos_a - y * size.y * sin_a;
        let rot_y = x * size.x * sin_a + y * size.y * cos_a;
        
        let screen_pos = center + vec2<f32>(rot_x, rot_y);
        let ndc = (screen_pos / globals.screen_size) * 2.0 - 1.0;
        out.position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
        
        out.uv = vec2<f32>(x, y) * size;
        out.dim = size * 0.5;
        out.color_start = in.color_start;
        out.color_end = in.color_end;
        out.params = vec4<f32>(f32(in.prim_type), in.stroke_width, in.softness, 0.0);
        out.radii = in.radii;
        return out;
    } else {
       // Rect: Expand size to account for softness blur
       // Original size is content size. 
       // We need to render a quad large enough to hold the blurred edge.
       // Padding = softness (blur radius) + 1.0 (AA)
       let padding = soft + 1.0;
       // We don't change 'size' passed to SDF (in.dim), but we scale the quad geometry.
       // Quad is -0.5..0.5
       // Real size is in.pos_b.
       // We want quad to cover real_size + 2*padding.
       let content_size = in.pos_b;
       let quad_size = content_size + vec2<f32>(padding * 2.0);
       
       let local_pos = vec2<f32>(x, y) * quad_size;
       let screen_pos = center + local_pos;
       let ndc = (screen_pos / globals.screen_size) * 2.0 - 1.0;
       
       out.position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
       out.uv = local_pos; // UV in pixels relative to center
       out.dim = content_size * 0.5; // Half-extents of the actual box
       out.color_start = in.color_start;
       out.color_end = in.color_end;
       out.params = vec4<f32>(f32(in.prim_type), in.stroke_width, in.softness, 0.0);
       out.radii = in.radii;
       return out;
    }
    
    // Fallback for Circle/Line paths that returned early? No, circle doesn't return.
    // Wait, Circle block above didn't compute position. 
    // Simplify for Circle logic reuse:
    
    let local_pos = vec2<f32>(x, y) * size;
    let screen_pos = center + local_pos;
    let ndc = (screen_pos / globals.screen_size) * 2.0 - 1.0;
    
    out.position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    out.uv = local_pos;
    out.dim = size * 0.5; // for Circle this is mostly bounding box
    out.color_start = in.color_start;
    out.color_end = in.color_end;
    out.params = vec4<f32>(f32(in.prim_type), in.stroke_width, in.softness, 0.0);
    out.radii = in.radii;
    
    return out;
}


fn sd_rounded_box_varying(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var rx: f32;
    if (p.x > 0.0) {
        if (p.y > 0.0) { rx = r.y; } // Bottom Right
        else { rx = r.x; }           // Top Right
    } else {
        if (p.y > 0.0) { rx = r.w; } // Bottom Left
        else { rx = r.z; }           // Top Left
    }
    let q = abs(p) - b + rx;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - rx;
}

fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var dist = 0.0;
    let prim_type = u32(in.params.x);
    let stroke_width = in.params.y;
    let softness = in.params.z;
    
    if (prim_type == 0u) {
        // Rounded Rect varying
        dist = sd_rounded_box_varying(in.uv, in.dim, in.radii);
    } else if (prim_type == 1u) {
        // Circle - fix radius access from radii[0]
        // For circle, we used size logic in VS, but here dist is simple.
        // Radius was passed in radii.x? VS used it to size quad.
        // We need effective radius. 
        // In VS: size = (r + pad)*2. dim = size/2 = r + pad.
        // But the box SDF for circle is wrong.
        // We need explicit radius.
        dist = sd_circle(in.uv, in.radii.x);
    } else if (prim_type == 2u) {
        // Line
        dist = sd_rounded_box_varying(in.uv, in.dim, vec4<f32>(in.dim.y)); 
    }
    
    var alpha = 0.0;
    
    // AA width: usually 0.5 to 1.0 pixel.
    let aa = 0.5 + max(softness, 0.0);
    
    if (stroke_width > 0.0) {
        // Stroke logic with softness? 
        // For now assume stroke is crisp or matches softness.
        let d_outer = dist;
        let d_inner = dist + stroke_width;
        let alpha_outer = 1.0 - smoothstep(-aa, aa, d_outer);
        let alpha_inner = 1.0 - smoothstep(-aa, aa, d_inner);
        alpha = alpha_outer - alpha_inner;
    } else {
        // Fill
        // Dist is <= 0 inside.
        // smoothstep(edge0, edge1, x)
        // We want 1 when dist is very negative. 0 when dist > aa.
        alpha = 1.0 - smoothstep(-aa, aa, dist);
    }
    
    // Gradient Mixing
    // UV.y is from -height/2 (top) to +height/2 (bottom).
    // Normalize to 0..1
    let grad_t = clamp((in.uv.y / in.dim.y) * 0.5 + 0.5, 0.0, 1.0);
    var final_color = mix(in.color_start, in.color_end, grad_t);
    
    final_color.a = final_color.a * alpha;
    
    if (final_color.a <= 0.0) {
        discard;
    }
    
    return final_color;
}


