const GRID_SIZE = vec2<f32>(60, 60);
const view_point = vec2<f32>(680, 480);

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,

    @location(0) glyph: u32,
    @location(1) foreground_color: vec4<f32>,
    @location(2) background_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) glyph: u32,
    @location(2) foreground_color: vec4<f32>,
    @location(3) background_color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let vertex_index = input.vertex_index;
    let uv = vec2<f32>(
        f32(input.vertex_index & 1u),
        f32((input.vertex_index >> 1u) & 1u),
    );

    var out: VertexOutput;
    out.uv = uv;
    out.glyph = input.glyph;
    out.foreground_color = input.foreground_color;
    out.background_color = input.background_color;

    let origin_px = vec2<f32>(
        f32(input.instance_index) * GRID_SIZE.x,
        0.0,
    );

    let pixel_pos = origin_px + uv * GRID_SIZE;

    let ndc = pixel_pos / view_point
        * vec2<f32>(2.0, -2.0)
        + vec2<f32>(-1.0, 1.0);

    out.clip_position = vec4<f32>(ndc, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.background_color);
}
