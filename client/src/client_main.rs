#![cfg(not(target_arch = "wasm32"))]
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use super::graphics;

pub fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    pollster::block_on(graphics::run_loop(event_loop, window));
}
