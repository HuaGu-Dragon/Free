const GRID_SIZE = vec2<f32>(60, 60);

const FONT_WIDTH: u32 = 128;
const FONT_HEIGHT: u32 = 64;
const FONT_COLS: u32 = 18u;
const FONT_ROWS: u32 = 7u;
const FONT_CHAR_WIDTH: u32 = FONT_WIDTH / FONT_COLS;
const FONT_CHAR_HEIGHT: u32 = FONT_HEIGHT / FONT_ROWS;
const FONT_CHAR_WIDTH_UV: f32 = f32(FONT_CHAR_WIDTH) / f32(FONT_WIDTH);
const FONT_CHAR_HEIGHT_UV: f32 = f32(FONT_CHAR_HEIGHT) / f32(FONT_HEIGHT);

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

struct Program {
    resolution: vec2<f32>,
    elapsed: f32,
    _padding: f32,
};

@group(1) @binding(0)
var<uniform> program: Program;

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

    let ndc = pixel_pos / program.resolution
        * vec2<f32>(2.0, -2.0)
        + vec2<f32>(-1.0, 1.0);

    out.clip_position = vec4<f32>(ndc, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let glyph = input.glyph - 32u;
    let cell = vec2<f32>(
        f32(glyph % FONT_COLS) * FONT_CHAR_WIDTH_UV,
        f32(glyph / FONT_COLS) * FONT_CHAR_HEIGHT_UV,
    );
    var pos = vec2f(cell.x, cell.y);
    var size = vec2f(FONT_CHAR_WIDTH_UV, FONT_CHAR_HEIGHT_UV);

    var t = pos + size * input.uv;

    let mask = textureSample(t_diffuse, s_diffuse, t).r;

    let screen_uv = input.clip_position.xy / program.resolution;

    let gradient_t = fract(screen_uv.x + program.elapsed);
    let phase = gradient_t * 6.2831853;
    let gradient_rgb = 0.5 + 0.5 * cos(
        phase + vec3<f32>(0.0, 2.094395, 4.188790)
    );
    let foreground = vec4<f32>(gradient_rgb, input.foreground_color.a);

    return mix(input.background_color, foreground, mask);
}
