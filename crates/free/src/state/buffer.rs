pub struct Buffer<T> {
    data: Vec<T>,
    gpu_capacity: usize,
    pub buffer: wgpu::Buffer,
}

impl<T> Buffer<T> {
    pub fn new(device: &wgpu::Device, capacity: usize) -> Self {
        let gpu_capacity = capacity.max(1);

        let buffer = Self::create_buffer(device, gpu_capacity, false);

        Self {
            data: Vec::with_capacity(capacity),
            gpu_capacity,
            buffer,
        }
    }

    fn create_buffer(device: &wgpu::Device, capacity: usize, mapped: bool) -> wgpu::Buffer {
        let size = capacity
            .checked_mul(std::mem::size_of::<T>())
            .expect("buffer size overflow") as wgpu::BufferAddress;
        let label = &format!("{} Buffer", std::any::type_name::<T>());

        device.create_buffer(&wgpu::BufferDescriptor {
            // TODO: use `const_format` to labelled it to `std::any::type_name::<T>() Buffer"`
            label: Some(&label),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: mapped,
        })
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn flush(&mut self, device: &wgpu::Device, queue: &wgpu::Queue)
    where
        T: bytemuck::NoUninit,
    {
        if self.data.len() > self.gpu_capacity {
            let new_cap = self.data.capacity();
            let buffer = Self::create_buffer(device, new_cap, true);
            {
                let bytes: &[u8] = bytemuck::cast_slice(&self.data);
                // TODO: replace `unwrap` with `expect`
                let mut view = buffer
                    .slice(..bytes.len() as u64)
                    .get_mapped_range_mut()
                    .expect("");
                view.copy_from_slice(bytes);
            }

            buffer.unmap();
            self.buffer = buffer;
            self.gpu_capacity = new_cap;
            return;
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
