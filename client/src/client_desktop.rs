#![cfg(not(target_arch = "wasm32"))]
use winit::{event_loop::EventLoop, window::WindowBuilder};

use super::graphics;

pub fn run() {
    // Start logger, uses env variable MY_LOG_LEVEL to determine log level
    // E.g. export RUST_LOG=debug
    #[cfg(feature = "env_logger")]
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    pollster::block_on(graphics::run_loop(event_loop, window));
}
