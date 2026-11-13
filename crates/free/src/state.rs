use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use winit::{dpi::PhysicalSize, event_loop::OwnedDisplayHandle, window::Window};

use crate::state::pipeline::{Program, TilePipeline};

mod pipeline;

pub struct State {
    instance: wgpu::Instance,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    texture_format: wgpu::TextureFormat,

    tile_pipeline: pipeline::TilePipeline,
    // FIXME: should this handle by tile_pipeline?
    last_frame: Instant,
    runtime: Duration,
}

impl State {
    pub async fn new(display: OwnedDisplayHandle, window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(display),
        ));
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let texture_format = cap.formats[0];

        let tile_pipeline = TilePipeline::new(
            &device,
            &queue,
            Program {
                resolution: [size.width as f32, size.height as f32],
                elapsed: 0.0,
                _padding: 0.0,
            },
            texture_format,
        );

        let state = State {
            instance,
            window,
            device,
            queue,
            size,
            surface,
            texture_format,

            tile_pipeline,
            last_frame: Instant::now(),
            runtime: Duration::default(),
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.texture_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.texture_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
        self.tile_pipeline.resize(&self.queue, size);

        self.configure_surface();
    }

    pub fn render(&mut self) {
        let now = Instant::now();
        let detla = now.duration_since(self.last_frame);
        self.last_frame = now;
        self.runtime += detla;

        self.tile_pipeline.update(&self.queue, self.runtime);

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                drop(texture);
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                unreachable!("No error scope registered, so validation errors will panic")
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(self.window.clone()).unwrap();
                self.configure_surface();
                return;
            }
        };
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.texture_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // TODO: refactor this
        renderpass.set_pipeline(&self.tile_pipeline.render_pipeline);
        renderpass.set_bind_group(0, &self.tile_pipeline.diffuse_bind_group, &[]);
        renderpass.set_bind_group(1, &self.tile_pipeline.program_bind_group, &[]);
        renderpass.set_vertex_buffer(0, self.tile_pipeline.tile_buffer.slice());
        renderpass.draw(0..4, 0..self.tile_pipeline.tile_buffer.len() as u32);

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(surface_texture);
    }
}
