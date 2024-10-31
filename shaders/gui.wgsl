struct VertexInput {
    @location(0) position: vec2<f32>,     // NDC coords (-1 to 1)
    @location(1) tex_coords: vec2<f32>,   // UV coords (0 to 1)
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    screen_size: vec2<f32>,
    drag_start: vec2<f32>,    // Screen coords (0 to width/height)
    drag_end: vec2<f32>,
    selection_start: vec2<f32>,
    selection_end: vec2<f32>,
    time: f32,
    is_dragging: u32,
};

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Pass through texture coordinates unchanged
    out.tex_coords = model.tex_coords;
    // Pass through position unchanged
    out.clip_position = vec4<f32>(model.position.xy, 0.0, 1.0);
    return out;
}

// Add this helper function for generating rainbow colors
fn rainbow(t: f32) -> vec4<f32> {
    let r = sin(t) * 0.5 + 0.5;
    let g = sin(t + 2.094) * 0.5 + 0.5; // 2.094 = 2π/3
    let b = sin(t + 4.189) * 0.5 + 0.5; // 4.189 = 4π/3
    return vec4<f32>(r, g, b, 0.8);
}

// Modified fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let screen_pos = in.tex_coords * uniforms.screen_size;
    
    // Draw drag rectangle with subtle highlight
    if (uniforms.is_dragging == 1u || uniforms.is_dragging == 3u) && point_in_rectangle(screen_pos, uniforms.drag_start, uniforms.drag_end) {
        color = mix(color, vec4<f32>(0.5, 0.5, 1.0, 1.0), 0.2);
    }
    
    // Draw animated selection border
    if uniforms.is_dragging >= 2u {
        // Create outer and inner borders for depth effect
        let outer_border = is_on_border(screen_pos, uniforms.selection_start, uniforms.selection_end, 2.0);
        let inner_border = is_on_border(screen_pos, uniforms.selection_start, uniforms.selection_end, 1.0);
        
        if outer_border {
            // Animated rainbow border with alpha blend
            let border_color = rainbow(uniforms.time * 2.0);
            let glow = 0.8;
            color = mix(color, border_color, glow);
        }
        if inner_border {
            // Brighter inner highlight
            let highlight_color = rainbow(uniforms.time * 2.0 + 0.5);
            color = mix(color, vec4<f32>(highlight_color.rgb, 1.0), 0.9);
        }
    }

    return color;
}

fn point_in_rectangle(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> bool {
    if start.x == 0 && start.y == 0 || end.x == 0 && end.y == 0 {
        return false;
    }
    let min_pos = min(start, end);
    let max_pos = max(start, end);
    return all(point >= min_pos) && all(point <= max_pos);
}

fn is_on_border(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>, thickness: f32) -> bool {
    let min_pos = min(start, end);
    let max_pos = max(start, end);

    let outer = point_in_rounded_rectangle(
        point,
        min_pos - vec2<f32>(thickness),
        max_pos + vec2<f32>(thickness),
        thickness
    );

    let inner = point_in_rounded_rectangle(
        point,
        min_pos + vec2<f32>(thickness),
        max_pos - vec2<f32>(thickness),
        thickness
    );

    return outer && !inner;
}

fn point_in_rounded_rectangle(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>, radius: f32) -> bool {
    let min_pos = min(start, end);
    let max_pos = max(start, end);

    let rounded_min = min_pos + vec2<f32>(radius);
    let rounded_max = max_pos - vec2<f32>(radius);

    let inside_rect = all(point >= rounded_min) && all(point <= rounded_max);

    let corner_radius = vec2<f32>(radius);
    let corner_distance = max(vec2<f32>(0.0), abs(point - (min_pos + max_pos) * 0.5) - corner_radius);
    let inside_rounded = all(corner_distance <= corner_radius);

    return inside_rect || inside_rounded;
}