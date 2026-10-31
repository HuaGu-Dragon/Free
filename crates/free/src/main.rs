use winit::event_loop::EventLoop;

use crate::app::App;

mod app;

fn main() {
    let event_loop = EventLoop::new().expect("failed to create event loop");

    event_loop
        .run_app(&mut App::default())
        .expect("event loop failed");
}
