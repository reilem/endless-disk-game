#![cfg(not(target_arch = "wasm32"))]
use async_std::task::sleep;
use instant::Duration;
use std::thread;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use super::graphics;

pub fn run() {
    // Start logger, uses env variable MY_LOG_LEVEL to determine log level
    // E.g. export RUST_LOG=debug
    #[cfg(feature = "env_logger")]
    env_logger::init();

    // TODO: these two lines are duplicated in both clients consider resolving as this is not client-specific
    let event_loop = EventLoop::new();
    // Note: windows launched on mac os from full-screen IDE / terminals will result in a window that is not movable: https://github.com/rust-windowing/winit/issues/1950
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    thread::spawn(|| {
        pollster::block_on(async {
            loop {
                log::info!("Looping!");
                sleep(Duration::from_millis(500)).await;
            }
        })
    });
    pollster::block_on(graphics::run_loop(event_loop, window));
}
