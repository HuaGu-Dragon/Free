// TODO: move this to a seperate crate

use std::ops::Range;

pub struct Buffer<T> {
    data: Vec<T>,
    gpu_capacity: usize,
    dirty: Option<Range<usize>>,
    buffer: wgpu::Buffer,
}

impl<T> Buffer<T> {
    pub fn new(device: &wgpu::Device, capacity: usize) -> Self {
        const {
            assert!(std::mem::size_of::<T>() > 0);
            assert!(
                std::mem::size_of::<T>().is_multiple_of(4),
                "size_of::<T>() must be a multiple of 4"
            );
        }

        let gpu_capacity = capacity.max(1);

        Self {
            data: Vec::with_capacity(capacity),
            gpu_capacity,
            dirty: None,
            buffer: Self::create_buffer(device, gpu_capacity, false),
        }
    }

    fn create_buffer(device: &wgpu::Device, capacity: usize, mapped: bool) -> wgpu::Buffer {
        let size = capacity
            .checked_mul(std::mem::size_of::<T>())
            .expect("buffer size overflow") as wgpu::BufferAddress;
        let label = format!("{} Buffer", std::any::type_name::<T>());

        device.create_buffer(&wgpu::BufferDescriptor {
            // TODO: use `const_format` to labelled it to `std::any::type_name::<T>() Buffer"`
            label: Some(&label),
            size,
            // TODO: custom buffer_usages
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: mapped,
        })
    }

    fn mark_dirty(&mut self, r: Range<usize>) {
        self.dirty = Some(match self.dirty.take() {
            Some(d) => d.start.min(r.start)..d.end.max(r.end),
            None => r,
        });
    }

    pub fn push(&mut self, value: T) {
        let i = self.data.len();
        self.data.push(value);
        self.mark_dirty(i..i + 1);
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.dirty = None;
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
                let mut view = buffer
                    .slice(..bytes.len() as u64)
                    .get_mapped_range_mut()
                    .expect(
                        "the beginning of this slice is 0 which is aliged to `MAP_ALIGNMENT`, the length of slice is checked in compile time, the buffer is currently mapped, there is not others `BufferView`",
                    );
                view.copy_from_slice(bytes);
            }

            buffer.unmap();
            self.buffer = buffer;
            self.gpu_capacity = new_cap;
            return;
        }

        if let Some(r) = self.dirty.take() {
            let r = r.start..r.end.min(self.data.len());
            if r.is_empty() {
                return;
            }

            let offset = (r.start * std::mem::size_of::<T>()) as wgpu::BufferAddress;
            queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(&self.data[r]));
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        let bytes = (self.data.len() * std::mem::size_of::<T>()) as u64;
        self.buffer.slice(..bytes)
    }
}
