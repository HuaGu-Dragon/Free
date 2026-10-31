use crate::tiles::Tile;

pub struct TileBuffer {
    data: Vec<Tile>,
    pub vertex_buffer: wgpu::Buffer,
}

impl TileBuffer {
    pub fn new(device: &wgpu::Device, capacity: usize) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::wgt::BufferDescriptor {
            label: Some("Tile Buffer"),
            size: (capacity * std::mem::size_of::<Tile>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            data: Vec::with_capacity(capacity),
            vertex_buffer,
        }
    }

    pub fn push(&mut self, value: Tile) {
        self.data.push(value);
    }

    pub fn write(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.data));
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
