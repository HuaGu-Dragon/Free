use std::mem::MaybeUninit;

pub struct Buffer<T> {
    data: Box<[MaybeUninit<T>]>,
    len: usize,
    capacity: usize,
    pub vertex_buffer: wgpu::Buffer,
}

impl<T> Buffer<T> {
    pub fn new(device: &wgpu::Device, capacity: usize) -> Self {
        let capacity = capacity.max(1);

        let vertex_buffer = Self::create_buffer(device, capacity);

        Self {
            data: Self::create_data(capacity),
            len: 0,
            capacity,
            vertex_buffer,
        }
    }

    fn create_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
        let size = (capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress;

        device.create_buffer(&wgpu::BufferDescriptor {
            // TODO: use `const_format` to labelled it to `std::any::type_name::<T>() Buffer"`
            label: Some(std::any::type_name::<T>()),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_data(capacity: usize) -> Box<[MaybeUninit<T>]> {
        Box::new_uninit_slice(capacity)
    }

    pub fn push(&mut self, device: &wgpu::Device, value: T) {
        if self.len == self.capacity {
            let new_capa = self
                .capacity
                .checked_mul(2)
                .expect("buffer capacity overflow");
            self.resize(device, new_capa);
        }

        self.data[self.len].write(value);
        self.len += 1;
    }

    pub fn write(&self, queue: &wgpu::Queue)
    where
        T: bytemuck::NoUninit,
    {
        if self.len == 0 {
            return;
        }

        // SAFETY: `push` only init [0..self.len)，resize would keep this area
        let init = unsafe { self.data[..self.len].assume_init_ref() };

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(init));
    }

    fn resize(&mut self, device: &wgpu::Device, new_capa: usize) {
        if new_capa <= self.capacity {
            return;
        }

        let mut new_data = Self::create_data(new_capa);

        unsafe {
            std::ptr::copy_nonoverlapping(self.data.as_ptr(), new_data.as_mut_ptr(), self.len);
        }

        self.data = new_data;
        self.vertex_buffer = Self::create_buffer(device, new_capa);
        self.capacity = new_capa;
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        // SAFETY: `push` only init [0..self.len)，resize would keep this area
        unsafe { self.data[..self.len].assume_init_drop() };
    }
}
