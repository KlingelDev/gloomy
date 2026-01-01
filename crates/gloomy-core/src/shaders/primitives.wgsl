// Gloomy SDF Primitives Shader
// Renders rectangles, circles, and lines using signed distance fields.

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    // Instance Data
    @location(0) pos_a: vec2<f32>,
    @location(1) pos_b: vec2<f32>,
    @location(2) color: vec4<f32>,
    // Radii: TopRight, BottomRight, TopLeft, BottomLeft
    @location(3) radii: vec4<f32>,
    @location(4) prim_type: u32,
    @location(5) stroke_width: f32,
    @location(6) _pad: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) dim: vec2<f32>,
    @location(2) color: vec4<f32>,
    // Params: type, stroke_width, unused, unused
    @location(3) params: vec4<f32>,
    @location(4) radii: vec4<f32>,
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
    
    if (in.prim_type == 1u) {
        // Circle: radius in radii.x
        // Padding for AA and stroke.
        // If stroke is "inner", we don't strictly need extra padding outward for stroke BUT we need it for AA.
        // For line (type 2), stroke is centered.
        
        let r = in.radii.x;
        // Padding
        let padding = stroke + 2.0; 
        size = vec2<f32>((r + padding) * 2.0, (r + padding) * 2.0);
    } else if (in.prim_type == 2u) {
        // Line
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
        out.color = in.color;
        out.params = vec4<f32>(f32(in.prim_type), in.stroke_width, 0.0, 0.0);
        out.radii = in.radii;
        return out;
    }
    
    // Rect
    // Pass local pos in pixels
    let local_pos = vec2<f32>(x, y) * size;
    let screen_pos = center + local_pos;
    let ndc = (screen_pos / globals.screen_size) * 2.0 - 1.0;
    
    out.position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    out.uv = local_pos;
    out.dim = size * 0.5;
    out.color = in.color;
    out.params = vec4<f32>(f32(in.prim_type), in.stroke_width, 0.0, 0.0);
    out.radii = in.radii;
    
    return out;
}

fn sd_rounded_box_varying(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    // r: TopRight, BottomRight, TopLeft, BottomLeft (matching order if desired)
    // p.x > 0 (Right), p.x < 0 (Left)
    // p.y > 0 (Bottom? No, in 2D usually Y up/down depends. Here UV is from center.)
    // In our UI, Y is down? 
    // vs_main sets x,y = -0.5..0.5.
    // ndc y is inverted (-ndc.y). So screen Y increases downwards.
    // vs_main: y = -0.5 is "Top" visually in standard GL if Y up. But we invert Y in NDC.
    // If Y increases downwards in screen space (0 at top, H at bottom):
    // y=-0.5 (local) -> center - H/2 (Top).
    // y=0.5 (local) -> center + H/2 (Bottom).
    
    // So p.y < 0 is TOP, p.y > 0 is BOTTOM.
    // p.x < 0 is LEFT, p.x > 0 is RIGHT.
    
    // r components: x=TR, y=BR, z=TL, w=BL.
    
    var rx: f32;
    if (p.x > 0.0) {
        // Right side
        if (p.y > 0.0) { rx = r.y; } // Bottom Right
        else { rx = r.x; }           // Top Right
    } else {
        // Left side
        if (p.y > 0.0) { rx = r.w; } // Bottom Left
        else { rx = r.z; }           // Top Left
    }
    
    let q = abs(p) - b + rx;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - rx;
}

fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sd_box(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var dist = 0.0;
    let prim_type = u32(in.params.x);
    let stroke_width = in.params.y;
    
    if (prim_type == 0u) {
        // Rounded Rect varying
        dist = sd_rounded_box_varying(in.uv, in.dim, in.radii);
    } else if (prim_type == 1u) {
        // Circle
        dist = sd_circle(in.uv, in.radii.x);
    } else if (prim_type == 2u) {
        // Line
        dist = sd_rounded_box_varying(in.uv, in.dim, vec4<f32>(in.dim.y)); // uniform radius
    }
    
    // Stroke handling
    // If stroke > 0, we treat it as INNER stroke OR CENTERED?
    // User complaint: "thicker in corners".
    // Inner stroke means we only render pixels where dist is between -stroke and 0.
    // This preserves the outer boundary exactly (no corner bulge).
    
    var alpha = 0.0;
    
    if (stroke_width > 0.0) {
        // Inner stroke logic:
        // dist is negative inside. 0 at edge.
        // We want opacity 1.0 inside [-stroke, 0].
        // distance to "stroke band":
        // let center_of_stroke = -stroke_width * 0.5;
        // let d2 = abs(dist - center_of_stroke) - stroke_width * 0.5;
        // alpha = 1.0 - smoothstep(-0.5, 0.5, d2);
        
        // Simpler: 
        // We want to stroke the border.
        // Inner Stroke: matches CSS border-box if we didn't adjust size.
        // Let's implement Inner Stroke.
        
        let d_outer = dist;
        let d_inner = dist + stroke_width; // shifts 0 to -stroke.
        
        // We want d_outer <= 0 AND d_inner >= 0 ? No.
        // We want region where d_outer is <= 0 and d_inner > 0 is NOT true?
        // Wait, "Inner Stroke" of width W.
        // It occupies SDF range [-W, 0].
        // We want alpha=1 in [-W, 0].
        // Use smoothstep for AA on both edges.
        
        let alpha_outer = 1.0 - smoothstep(-0.5, 0.5, d_outer); // 1 inside, 0 outside
        let alpha_inner = 1.0 - smoothstep(-0.5, 0.5, d_inner); // 1 inside inner edge, 0 outside
        
        // Stroke is strictly Outer - Inner (hollow).
        alpha = alpha_outer - alpha_inner;
        
    } else {
        // Fill
        alpha = 1.0 - smoothstep(-0.5, 0.5, dist);
    }
    
    var final_color = in.color;
    final_color.a = final_color.a * alpha;
    
    if (final_color.a <= 0.0) {
        discard;
    }
    
    return final_color;
}


