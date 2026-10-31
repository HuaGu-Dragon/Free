#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Tile {
    pub glyph: u32,
    pub foreground_color: [f32; 4],
    pub background_color: [f32; 4],
}

impl Tile {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![
            0 => Uint32,
            1 => Float32x4,
            2 => Float32x4,
        ],
    };
}
