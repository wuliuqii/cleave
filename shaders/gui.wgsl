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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get the base texture color
    var color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // Convert texture coordinates to screen coordinates
    let screen_pos = in.tex_coords * uniforms.screen_size;
    
    // Draw drag rectangle
    if (uniforms.is_dragging == 1u || uniforms.is_dragging == 3u) && 
       point_in_rectangle(screen_pos, uniforms.drag_start, uniforms.drag_end) {
        color = mix(color, vec4<f32>(0.5, 0.5, 1.0, 1.0), 0.3);
    }
    
    // Draw selection border
    if uniforms.is_dragging >= 2u && 
       is_on_border(screen_pos, uniforms.selection_start, uniforms.selection_end, 2.0) {
        color = vec4<f32>(1.0, 1.0, 1.0, 0.8);
    }
    
    return color;
}

fn point_in_rectangle(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> bool {
    if (start.x == 0 && start.y == 0 || end.x == 0 && end.y == 0) {
        return false;
    }
    let min_pos = min(start, end);
    let max_pos = max(start, end);
    return all(point >= min_pos) && all(point <= max_pos);
}

fn is_on_border(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>, thickness: f32) -> bool {
    let min_pos = min(start, end);
    let max_pos = max(start, end);
    
    let outer = point_in_rectangle(
        point, 
        min_pos - vec2<f32>(thickness),
        max_pos + vec2<f32>(thickness)
    );
    
    let inner = point_in_rectangle(
        point,
        min_pos + vec2<f32>(thickness),
        max_pos - vec2<f32>(thickness)
    );
    
    return outer && !inner;
}